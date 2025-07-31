use crate::config::config::Config;
use crate::config::config_model::ConfigModel;
use crate::error::Result;
use crate::model::model_kind::ModelKind;
use crate::model::models::llama::llama::ModelLLama;
use std::rc::Rc;
use std::sync::{Arc, RwLock};
use mlx_rs::Stream;
//use crate::models::model_mistral::ModelMistral;

pub fn create_model_instance(config: Rc<Config>, stream: Option<Arc<Stream>>) -> Result<Arc<RwLock<ModelKind>>> {
    let model = &config.model;
    match &**model {
        ConfigModel::LLaMA(llama_config) => {
            let instance = ModelLLama::new(llama_config.clone(), stream)?;
            Ok(Arc::new(RwLock::new(ModelKind::LLaMA(instance))))
        }
    }
}
