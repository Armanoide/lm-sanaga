use crate::config::config::Config;
use crate::config::config_model::ConfigModel;
use sn_core::error::Result;
use crate::model::model_kind::ModelKind;
use crate::model::models::llama::llama::ModelLLama;
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
    }
}
