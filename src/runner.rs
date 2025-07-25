use log::info;
use crate::conversation::{Conversation};
use crate::error::{Result};
use crate::model::model_runtime::ModelRuntime;

pub struct Runner {
    models: Vec<(String, ModelRuntime)>,
}

impl Runner {
    pub fn new() -> Self {
        Runner{
            models: Vec::new(),
        }
    }

    fn generate_unique_id(salt: &String) -> String {
        let id = hex::encode(salt.as_bytes());
        String::from(&id[..8])
    }
    pub fn load_model_by_path(&mut self, path: &String) -> Result<()>{
        let id = Self::generate_unique_id(path);
        let mut model_runtime = ModelRuntime::load_with_path(path)?;
        let _ = &model_runtime.routine_model()?;
        info!("Model {} loaded in container {}", &model_runtime.name, id);
        self.models.push((id, model_runtime));
        Ok(())
    }

    pub fn unload_model(&mut self, model_id: &str) {

    }

    pub fn generate_text(&mut self, model_id:&str, conversation: &Conversation) -> Result<()> {
        let (_, m) = self.models.first_mut().unwrap();
        m.generate_text(conversation)
    }

}

