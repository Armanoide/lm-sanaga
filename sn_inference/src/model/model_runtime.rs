use crate::cache::k_v_cache::k_v_cache::ArcCacheList;
use crate::chat_template::chat_template::ChatTemplate;
use crate::config::config::Config;
use crate::config::config_model::{ConfigModel, ConfigModelCommon};
use crate::error::{Error, Result};
use crate::factory::model::create_model_instance;
use crate::model::model::Model;
use crate::model::model_kind::ModelKind;
use crate::model::weight::Weight;
use crate::quantized::Quantize;
use crate::token::token_embedding_generator::TokenEmbeddingGenerator;
use crate::token::token_stream_manager::{PromptStreamCallback, TokenStreamManager};
use crate::tokenizer::tokenizer::Tokenizer;
use crate::utils::mlx::similarity::similarity_cos;
use crate::utils::tokenizer::pad_encode_batch::pad_encode_batch;
use mlx_rs::Array;
use mlx_rs::ops::indexing::IndexOp;
use mlx_rs::ops::stack;
use serde::{Deserialize, Serialize};
use sn_core::types::conversation::Conversation;
use sn_core::types::message_stats::MessageStats;
use sn_core::utils::rw_lock::RwLockExt;
use std::path::Path;
use std::rc::Rc;
use std::sync::{Arc, RwLock};
use walkdir::WalkDir;

pub type GenerateTextResult = (String, Option<MessageStats>);

#[derive(Debug, Serialize, Deserialize)]
pub struct ModelRuntime {
    pub id: String,
    pub name: String,
    pub model_path: String,
    pub config: Rc<Config>,
    #[serde(skip_serializing, skip_deserializing)]
    pub model: Option<Arc<RwLock<ModelKind>>>,
    #[serde(skip_serializing, skip_deserializing)]
    pub tokenizer: Option<Rc<Tokenizer>>,
    #[serde(skip_serializing, skip_deserializing)]
    pub weight: Option<Weight>,
    #[serde(skip_serializing, skip_deserializing)]
    pub chat_template: Option<Rc<ChatTemplate>>,
}

//todo :// - Add support for multiple models in the same runtime
const EMB_TASK_QUERY: &str =
    "Instruct: Given a web search query, retrieve relevant passages that answer the query\nQuery: ";
impl ModelRuntime {
    pub fn load_with_path(
        root_path: &str,
        id: &String,
        callback: Option<PromptStreamCallback>,
    ) -> Result<ModelRuntime> {
        let path = Path::new(&root_path);
        if !path.exists() {
            return Err(Error::ModelPathNotFound(path.display().to_string()));
        }

        let model_path = Self::find_model_path_from_root(&root_path)?;
        let config = Rc::new(Config::new(&model_path)?);
        let name = Self::get_name(&config.model);
        let weight = Weight::new(&config, callback)?;
        let model = create_model_instance(config.clone())?;
        let tokenizer = Rc::new(Tokenizer::new(config.clone())?);
        let chat_template = Rc::new(ChatTemplate::new(&config)?);

        Ok(ModelRuntime {
            id: id.clone(),
            name,
            model_path,
            config,
            model: Some(model),
            tokenizer: Some(tokenizer),
            weight: Some(weight),
            chat_template: Some(chat_template),
        })
    }

    fn get_name(config_model: &ConfigModel) -> String {
        match config_model {
            ConfigModel::LLaMA(llama_config) => llama_config.get_name(),
            ConfigModel::Qwen3(qwen3_config) => qwen3_config.get_name(),
        }
    }

    fn find_model_path_from_root(root_path: &str) -> Result<String> {
        for entry in WalkDir::new(root_path).into_iter().flatten() {
            let path = entry.path();
            if path.is_file() {
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    if name.to_lowercase().starts_with("readme") {
                        if let Some(parent) = path.parent() {
                            return Ok(parent.display().to_string());
                        } else {
                            break;
                        }
                    }
                }
            }
        }
        Err(Error::RootModelPathNotFound(root_path.to_string()))
    }

    pub fn routine_model(&mut self) -> Result<()> {
        let model = self
            .model
            .as_ref()
            .ok_or(Error::RoutineMissingModel(self.name.clone()))?;
        let weight = self
            .weight
            .as_mut()
            .ok_or(Error::RoutineMissingWeight(self.name.clone()))?;

        model.write_lock("reading_model:sanitize")?.sanitize(weight);
        if model
            .read_lock("routine_model:supports_quantization")?
            .supports_quantization()
        {
            // Passing 0,0 as no effect the model, because the model will automatically
            // quantize compute from his config file.
            model.write_lock("routine_model")?.quantize(0, 0)?;
        }
        model
            .write_lock("routine_model:load_weights")?
            .load_weights(weight)?;
        Ok(())
    }

    pub fn generate_similarity(
        &self,
        queries: &Vec<String>,
        documents: &Vec<String>,
    ) -> Result<Array> {
        // Build inputs based on condition
        let queries: Vec<_> = queries
            .clone()
            .iter()
            .map(|query| format!("{}{}", EMB_TASK_QUERY, query))
            .collect();

        let mut all_texts = Vec::new();
        all_texts.extend(queries.iter().cloned());
        all_texts.extend(documents.iter().cloned());
        let embeddings = self.generate_embeddings(&all_texts)?;
        let query_embeddings = embeddings.index(0..queries.len() as i32);
        let doc_embeddings = embeddings.index(queries.len() as i32..);
        let scores = similarity_cos(&query_embeddings, &doc_embeddings)?;
        Ok(scores)
    }

    pub fn generate_embeddings(&self, texts: &Vec<String>) -> Result<Array> {
        let tokenizer = self.tokenizer.as_ref().ok_or(Error::MissingTokenizer)?;
        let model = self.model.as_ref().ok_or(Error::MissingModel)?;

        let to_array = |ids: &[u32]| -> Array { Array::from_slice(ids, &[ids.len() as i32]) };

        // get Pad token
        let pad_token = tokenizer.get_pad_token_id().ok_or(Error::MissingPadToken)?;

        let batch = tokenizer.encode_batch(texts.clone(), true)?;
        let batch = pad_encode_batch(&batch, pad_token.pad_id)?;
        let inputs: Vec<Array> = batch.iter().map(|i| to_array(i.get_ids())).collect();
        let masks: Vec<Array> = batch
            .iter()
            .map(|i| to_array(i.get_attention_mask()))
            .collect();
        let inputs = stack(inputs.as_ref())?;
        let masks = stack(masks.as_ref())?;
        let token_embedding_gen = TokenEmbeddingGenerator::new(model.clone());
        Ok(token_embedding_gen.generate(&inputs, &masks)?)
    }

    pub fn generate_text(
        &self,
        conversation: &Conversation,
        cache: ArcCacheList,
        callback: Option<PromptStreamCallback>,
    ) -> Result<GenerateTextResult> {
        let tokenizer = self.tokenizer.as_ref().ok_or(Error::MissingTokenizer)?;
        let model = self.model.as_ref().ok_or(Error::MissingModel)?;
        let chat_template = self
            .chat_template
            .as_ref()
            .ok_or(Error::MissingChatTemplate)?;

        // Render prompt from conversation
        let inputs = chat_template.apply_chat_template(conversation, None, None)?;
        let prompt_ids = tokenizer.encode_prompt(vec![inputs])?;

        if prompt_ids.is_empty() {
            return Err(Error::EmptyPrompt);
        }

        let mut stream = TokenStreamManager::new(model.clone(), tokenizer.clone());
        let generated_text = stream.generate_text(prompt_ids, cache, callback)?;
        let stats = stream.get_average_stats()?;

        Ok((generated_text, stats))
    }

    pub fn get_num_layer(&self) -> Result<usize> {
        if let Some(model) = &self.model {
            let guard = model.read_lock("get_num_layer")?;
            Ok(guard.get_num_layer())
        } else {
            Ok(1)
        }
    }
}
