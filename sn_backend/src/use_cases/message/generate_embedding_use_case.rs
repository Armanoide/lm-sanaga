use crate::error::{ErrorBackend, Result};
use sn_core::{types::message::Message, utils::rw_lock::RwLockExt};
use sn_inference::runner::Runner;
use std::sync::{Arc, RwLock};

pub struct GenerateEmbeddingUseCase {
    runner: Arc<RwLock<Runner>>,
}

impl GenerateEmbeddingUseCase {
    pub fn new(runner: Arc<RwLock<Runner>>) -> Self {
        GenerateEmbeddingUseCase { runner }
    }
    pub async fn generate_message_embeddings(&self, message: &Message) -> Result<Vec<f32>> {
        let embeddings = self
            .runner
            .read_lock("read runner for generate embeddings")?
            .generate_embeddings(&vec![message.content.clone()])
            .map_err(|e| ErrorBackend::Inference(e))?;
        let embeddings = embeddings.as_slice::<f32>().to_vec();
        Ok(embeddings)
    }
}
