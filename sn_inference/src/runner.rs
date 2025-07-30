use crate::error::Result;
use crate::model::model_runtime::ModelRuntime;
use tracing::info;
use sn_core::conversation::conversation::Conversation;
#[derive(Debug)]
pub struct Runner {
    pub models: Vec<ModelRuntime>,
}
const BASE_PATH: &str = "/Volumes/EXT1_SSD/Users/user1/Projects/ML/lm-sanaga/_MODEL";

impl Runner {
    pub fn new() -> Self {
        Runner { models: Vec::new() }
    }

    fn generate_unique_id(salt: &String) -> String {
        let id = hex::encode(salt.as_bytes());
        String::from(&id[..8])
    }
    pub fn load_model_name(&mut self, name: &String) -> Result<()> {
        let path = BASE_PATH.to_string() + name;
        let id = Self::generate_unique_id(&path);
        let mut model_runtime = ModelRuntime::load_with_path(path.as_str(), id)?;
        let _ = &model_runtime.routine_model()?;
        info!(
            "Model {} loaded in container {}",
            model_runtime.name, model_runtime.id
        );
        self.models.push(model_runtime);
        Ok(())
    }

    pub fn unload_model(&self, model_id: &str) {}

    pub fn generate_text(&self, model_id: &str, conversation: &Conversation) -> Result<()> {
        let model_runtime = self.models.first().unwrap();
        model_runtime.generate_text(conversation)
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
