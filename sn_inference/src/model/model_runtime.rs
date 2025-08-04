use crate::config::config::Config;
use crate::config::config_model::{ConfigModel, ConfigModelCommon};
use crate::error::{Error, Result};
use crate::factory::model::create_model_instance;
use crate::model::model::Model;
use crate::model::model_kind::ModelKind;
use crate::model::weight::Weight;
use crate::quantized::Quantize;
use crate::token::token_stream_manager::{PromptStreamCallback, TokenStreamManager};
use crate::tokenizer::tokenizer::Tokenizer;
use serde::{Deserialize, Serialize};
use sn_core::utils::rw_lock::RwLockExt;
use std::path::Path;
use std::rc::Rc;
use std::sync::{Arc, RwLock};
use walkdir::WalkDir;
use sn_core::types::conversation::Conversation;
use sn_core::types::message_stats::MessageStats;

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
}

impl ModelRuntime {
    pub fn load_with_path(root_path: &str, id: &String) -> Result<ModelRuntime> {
        let path = Path::new(&root_path);
        if !path.exists() {
            return Err(Error::ModelPathNotFound(path.display().to_string()));
        }

        let model_path = Self::find_model_path_from_root(&root_path)?;
        let config = Rc::new(Config::new(&model_path)?);
        let name = Self::set_name(&config.model);
        let weight = Weight::new(&config)?;
        let model = create_model_instance(config.clone())?;
        let tokenizer = Rc::new(Tokenizer::new(config.clone())?);

        Ok(ModelRuntime {
            id: id.clone(),
            name,
            model_path,
            config,
            model: Some(model),
            tokenizer: Some(tokenizer),
            weight: Some(weight),
        })
    }

    fn set_name(config_model: &ConfigModel) -> String {
        match config_model {
            ConfigModel::LLaMA(llama_config) => llama_config.get_name(),
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
        if let (Some(box_model), Some(weight)) = (&self.model, &mut self.weight) {
            let context = "reading model to sanitize";
            box_model.write_lock(context)?.sanitize(weight);
            match &*self.config.model {
                ConfigModel::LLaMA(llama_config) => {
                    if llama_config.quantization.is_some() {
                        // Passing 0,0 as no effect the model, because the model will automatically
                        // quantize compute from his config file.
                        let context = "reading model to quantize";
                        box_model.write_lock(context)?.quantize(0, 0)?;
                    }
                }
            }

            let context = "reading to load weights";
            box_model.write_lock(context)?.load_weights(weight)?;
        }
        Ok(())
    }

    pub fn generate_text(
        &self,
        conversation: &Conversation,
        callback: Option<PromptStreamCallback>,
    ) -> Result<GenerateTextResult> {
        if let (Some(tokenizer), Some(model)) = (&self.tokenizer, &self.model) {
            let (inputs, _) = tokenizer.apply_chat_template(conversation)?;
            let prompt = tokenizer.encode_prompt(vec!(inputs))?;
            let mut sr = TokenStreamManager::new(model.clone(), tokenizer.clone());
            let generated_text = sr.generate_text(prompt, callback)?;
            let stats = sr.get_average_stats()?;
            Ok((generated_text, stats))
        } else {
            Ok((String::default(), None))
        }
    }
}
