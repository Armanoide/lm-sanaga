use crate::config::config::Config;
use crate::config::config_model::ConfigModel;
use crate::error::Result;
use crate::model::model_kind::ModelKind;
use crate::model::models::llama::llama::ModelLLama;
use crate::model::models::qwen3::qwen3::ModelQwen3;
use std::rc::Rc;
use std::sync::{Arc, RwLock};
//use crate::models::model_mistral::ModelMistral;

pub fn create_model_instance(config: Rc<Config>) -> Result<Arc<RwLock<ModelKind>>> {
    let model = &config.model;
    match &**model {
        ConfigModel::LLaMA(llama_config) => {
            let instance = ModelLLama::new(llama_config.clone())?;
            Ok(Arc::new(RwLock::new(ModelKind::LLaMA(instance))))
        }
        ConfigModel::Qwen3(qwen_config) => {
            let instance = ModelQwen3::new(qwen_config.clone())?;
            Ok(Arc::new(RwLock::new(ModelKind::Qwen3(instance))))
        }
    }
}
