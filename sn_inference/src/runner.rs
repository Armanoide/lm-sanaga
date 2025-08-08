use crate::cache::k_v_cache::k_v_cache::ArcCacheList;
use crate::cache::k_v_cache::k_v_cache_session::KvCacheSession;
use crate::error::{Error, Result};
use crate::factory::k_v_cache::create_cache_from_model_runtime;
use crate::model::model_kind::ModelKind;
use crate::model::model_runtime::{GenerateTextResult, ModelRuntime};
use crate::token::token_stream_manager::PromptStreamCallback;
use minijinja::ErrorKind::BadSerialization;
use serde::{Deserialize, Serialize};
use sn_core::types::conversation::Conversation;
use sn_core::utils::rw_lock::RwLockExt;
use std::env::VarError;
use std::ops::Add;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};
use tracing::{error, info};

const BASE_PATH_DEFAULT: &str = "~/.sanaga";
#[derive(Debug)]
pub struct Runner {
    pub models: Arc<RwLock<Vec<Arc<ModelRuntime>>>>,
    pub session_caches: Arc<RwLock<Vec<KvCacheSession>>>,
}

fn expand_tilde(path: &str) -> String {
    if path.starts_with("~") {
        if let Some(home) = std::env::var_os("HOME") {
            return PathBuf::from(path.replacen("~", &home.to_string_lossy(), 1))
                .display()
                .to_string();
        }
    }
    PathBuf::from(path).display().to_string()
}

fn get_base_path() -> String {
    expand_tilde(
        std::env::var("PATH_SANAGA")
            .unwrap_or_else(|_| {
                info!("Using default base path: {}", BASE_PATH_DEFAULT);
                String::from(BASE_PATH_DEFAULT)
            })
            .as_str(),
    )
}

fn get_base_path_models() -> String {
    get_base_path().add("/models/")
}

fn create_cache(model_runtime: Arc<ModelRuntime>) -> Result<ArcCacheList> {
    Ok(create_cache_from_model_runtime(model_runtime)?)
}

impl Runner {
    pub fn new() -> Self {
        Runner {
            models: Arc::new(RwLock::new(Vec::new())),
            session_caches: Arc::new(RwLock::new(Vec::new())),
        }
    }

    fn generate_path_id(salt: &String) -> String {
        let id = hex::encode(salt.as_bytes());
        String::from(&id[(id.len() - 10)..])
    }
    pub fn load_model_name(
        &self,
        name: &str,
        callback: Option<PromptStreamCallback>,
    ) -> Result<(String)> {
        let path = get_base_path_models().add(name);
        let id = Self::generate_path_id(&path);

        if let Some(model_runtime) = self.get_model_by_id(&id) {
            info!(
                "Model {} already loaded in container {}",
                model_runtime.name, model_runtime.id
            );
            return Ok(id);
        }

        let mut model_runtime = ModelRuntime::load_with_path(path.as_str(), &id, callback)?;
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
            Ok(guard) => guard
                .iter()
                .find(|m| m.id == model_id)
                .map(|model| model.clone()),
            Err(_) => None,
        }
    }
    pub fn unload_model(&mut self, model_id: &str) {
        let result = self.models.read_lock("unload_model");
        match result {
            Ok(guard_models) => {
                guard_models
                    .iter()
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

    pub fn get_session_cache(
        &self,
        session_id: Option<i32>,
        model_id: &str,
    ) -> Result<ArcCacheList> {
        let model = match self.get_model_by_id(model_id) {
            Some(m) => m,
            None => return Err(Error::ModelRuntimeNotFoundWithId(model_id.to_string())),
        };

        if let Some(id) = session_id {
            // Try to find existing session cache
            {
                let guard = self
                    .session_caches
                    .read_lock("check existing session cache")?;
                if let Some(existing) = guard.iter().find(|c| c.session_id == id) {
                    return Ok(existing.cache.clone());
                }
            }

            // Create new session cache and store it
            let new_cache = create_cache(model.clone())?;
            let new_session = KvCacheSession {
                session_id: id,
                cache: new_cache.clone(),
            };
            self.session_caches
                .write_lock("insert new session cache")?
                .push(new_session);

            Ok(new_cache)
        } else {
            // Anonymous session: return a fresh cache, not stored
            create_cache(model.clone())
        }
    }

    pub fn generate_text(
        &self,
        model_id: &str,
        conversation: &Conversation,
        session_id: Option<i32>,
        callback: Option<PromptStreamCallback>,
    ) -> Result<GenerateTextResult> {
        if let Some(model_runtime) = self.get_model_by_id(model_id) {
            let cache = self.get_session_cache(session_id, model_id)?;
            model_runtime.generate_text(conversation, cache, callback)
        } else {
            Err(Error::ModelRuntimeNotFoundWithId(model_id.to_string()))
        }
    }

    pub fn scan_model_installed(&self) -> Result<Vec<String>> {
        let path_models = get_base_path_models();
        let paths = match std::fs::read_dir(&path_models) {
            Ok(paths) => paths,
            Err(e) => {
                error!("Failed to read directory: {}", path_models);
                return Err(Error::IOError(e));
            }
        };
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
