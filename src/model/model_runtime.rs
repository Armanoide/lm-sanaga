use std::rc::Rc;
use std::path::Path;
use std::sync::{Arc, RwLock};
use walkdir::WalkDir;
use crate::config::config::Config;
use crate::config::config_model::{ConfigModel, ConfigModelCommon};
use crate::model::weight::Weight;
use crate::conversation::Conversation;
use crate::factory::model::create_model_instance;
use crate::generator::stream_response::StreamResponse;
use crate::tokenizer::tokenizer::Tokenizer;
use crate::model::model::{Model};
use crate::error::{Error, Result};
use crate::model::model_kind::ModelKind;
use crate::quantized::Quantize;
use crate::utils::rw_lock::RwLockExt;
pub struct ModelRuntime {
    pub name: String,
    pub model_path: String,
    pub config: Rc<Config>,
    pub model: Option<Arc<RwLock<ModelKind>>>,
    pub tokenizer: Option<Rc<Tokenizer>>,
    pub weight: Option<Weight>,
}

impl ModelRuntime {
    pub fn load_with_path(root_path: &str) -> Result<ModelRuntime> {
        let path = Path::new(&root_path);
        if !path.exists() {
            return Err(Error::ModelPathNotFound);
        }

        let model_path = Self::find_model_path_from_root(&root_path)?;
        let config = Rc::new(Config::new(&model_path)?);
        let name = Self::set_name(&config.model);
        let weight = Weight::new(&config)?;
        let model = create_model_instance(config.clone())?;
        let tokenizer = Rc::new(Tokenizer::new(config.clone())?);

        Ok(ModelRuntime {
            name,
            model_path,
            config,
            model: Some(model),
            tokenizer: Some(tokenizer),
            weight: Some(weight),
        })
    }

    fn set_name(config_model: &ConfigModel) -> String{
        match config_model {
            ConfigModel::LLaMA(llama_config) => {
                llama_config.get_name()
            }
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
                            break
                        }
                    }
                }
            }
        }
        Err(Error::RootModelPathNotFound)
    }

    pub fn routine_model(&mut self) -> Result<()> {
        if let (Some(box_model), Some(weight))= (&self.model, &mut self.weight) {
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
            // free
            self.weight = None;
        }
        Ok(())
    }

    pub fn generate_text(&mut self, conversation: &Conversation) -> Result<()> {
        if let (Some( tokenizer), Some(model)) = (&self.tokenizer, &self.model){
            let (inputs, _) = tokenizer.apply_chat_template(conversation)?;
            let prompt = tokenizer.encode_prompt(inputs)?;
            // Arc<RwLock<ModelKind>>, tokenizer: Rc<Tokenizer>
            let mut sr = StreamResponse::new(model.clone(), tokenizer.clone());
            let _ = sr.generate_text(prompt);
            //let test = generator.generate2()?;
            //println!("{:?}", test);
        }
        Ok(())
    }
}

