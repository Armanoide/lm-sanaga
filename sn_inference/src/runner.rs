use std::sync::{Arc, RwLock};
use crate::error::{Error, Result};
use crate::model::model_runtime::{GenerateTextResult, ModelRuntime};
use crate::token::token_stream_manager::PromptStreamCallback;
use serde::{Deserialize, Serialize};
use tracing::{error, info};
use sn_core::types::conversation::Conversation;
use sn_core::utils::rw_lock::RwLockExt;

#[derive(Debug, Serialize, Deserialize)]
pub struct Runner {
    pub models: Arc<RwLock<Vec<Arc<ModelRuntime>>>>,
}
const BASE_PATH: &str = "/Volumes/EXT1_SSD/Users/user1/Projects/ML/lm-sanaga/_MODEL";

impl Runner {
    pub fn new() -> Self {
        Runner { models: Arc::new(RwLock::new(Vec::new())) }
    }

    fn generate_path_id(salt: &String) -> String {
        let id = hex::encode(salt.as_bytes());
        String::from(&id[(id.len() - 10)..])
    }
    pub fn load_model_name(&self, name: &str) -> Result<(String)> {
        let path = format!("{BASE_PATH}/{name}");
        let id = Self::generate_path_id(&path);

        if let Some(model_runtime) = self.get_model_by_id(&id) {
            info!(
                "Model {} already loaded in container {}",
                model_runtime.name, model_runtime.id
            );
            return Ok(id);
        }

        let mut model_runtime = ModelRuntime::load_with_path(path.as_str(), &id)?;
        let _ = &model_runtime.routine_model()?;
        info!(
            "Model {} loaded in container {}",
            model_runtime.name, model_runtime.id
        );
        {
            let context = "adding model to container";
            let mut guard = self.models.write_lock_mut(context)?;
            guard.push(Arc::new(model_runtime))
        }
        Ok(id)
    }

    fn get_model_by_id(&self, model_id: &str) -> Option<Arc<ModelRuntime>> {
        let context = "get_model_by_id";
        let result = self.models.read_lock(context);
        match result {
            Ok(guard) => {
                guard.iter().find(|m| m.id == model_id)
                    .map(|model| model.clone())
            }
            Err(_) => {
                None
            }
        }
    }
    pub fn unload_model(&mut self, model_id: &str) {
        let result = self.models.read_lock("unload_model");
        match result {
            Ok(guard_models) => {
                guard_models.iter()
                    .position(|model| model.id == model_id)
                    .map(|index| {
                        info!("Unloading model: {}", guard_models[index].name);
                        let guard = self.models.write_lock("unload_model");
                        match guard {
                            Ok(mut models) => {
                                models.remove(index);
                            }
                            Err(_) => {
                                error!("Failed to acquire write lock for unloading model");
                            }
                        }

                    });
            }
            Err(_) => {
                error!("Failed to acquire read lock for unloading model");
            }
        }

    }

    pub fn generate_text(
        &self,
        model_id: &str,
        conversation: &Conversation,
        callback: Option<PromptStreamCallback>,
    ) -> Result<GenerateTextResult>  {
        if let Some(model_runtime) = self.get_model_by_id(model_id) {
            model_runtime.generate_text(conversation, callback)
        } else {
            Err(Error::ModelRuntimeNotFoundWithId(model_id.to_string()))
        }
    }

    pub fn scan_model_installed(&self) -> Result<Vec<String>> {
        let paths = std::fs::read_dir(BASE_PATH)?;
        let list = paths
            .map(|res| res.map(|dir| dir.path()))
            .map(|path| path.ok())
            .map(|path| {
                if let Some(path) = path {
                    if let Some(name) = path.file_name() {
                        return String::from(name.display().to_string());
                    }
                }
                String::default()
            })
            .collect::<Vec<_>>();

        Ok(list)
    }
}

unsafe impl Sync for Runner {}
unsafe impl Send for Runner {}
