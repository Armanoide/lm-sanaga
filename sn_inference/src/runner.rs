use sn_core::error::Result;
use crate::model::model_runtime::ModelRuntime;
use log::info;
use sn_core::conversation::conversation::Conversation;

pub struct Runner {
    models: Vec<(String, ModelRuntime)>,
}

impl Runner {
    pub fn new() -> Self {
        Runner { models: Vec::new() }
    }

    fn generate_unique_id(salt: &String) -> String {
        let id = hex::encode(salt.as_bytes());
        String::from(&id[..8])
    }
    pub fn load_model_by_path(&mut self, path: &String) -> Result<()> {
        let id = Self::generate_unique_id(path);
        let mut model_runtime = ModelRuntime::load_with_path(path)?;
        let _ = &model_runtime.routine_model()?;
        info!("Model {} loaded in container {}", &model_runtime.name, id);
        self.models.push((id, model_runtime));
        Ok(())
    }

    pub fn unload_model(&self, model_id: &str) {}

    pub fn generate_text(&self, model_id: &str, conversation: &Conversation) -> Result<()> {
        let (_, m) = self.models.first().unwrap();
        m.generate_text(conversation)
    }
}
