use std::sync::{Arc, RwLock};

use crate::clients::ann::AnnClient;
use crate::domain::embedding::repository::EmbeddingRepository;
use crate::error::{ErrorBackend, Result};
use crate::use_cases::message::generate_embedding_use_case::GenerateEmbeddingUseCase;
use sn_core::types::message::Message;
use sn_inference::runner::Runner;

#[derive(Clone, Debug)]
pub struct EmbeddingService {
    repo_embedding: Arc<EmbeddingRepository>,
    runner: Arc<RwLock<Runner>>,
    ann_client: Arc<AnnClient>,
}

impl EmbeddingService {
    pub fn new(
        repo_embedding: Arc<EmbeddingRepository>,
        runner: Arc<RwLock<Runner>>,
        ann_client: Arc<AnnClient>,
    ) -> Self {
        EmbeddingService {
            runner,
            ann_client,
            repo_embedding,
        }
    }

    pub async fn generate_embedding(&self, message: Message) -> Result<()> {
        let conversation_id = message
            .conversation_id
            .ok_or_else(|| ErrorBackend::ConversationNotFound)?;
        let use_case = GenerateEmbeddingUseCase::new(self.runner.clone());
        let embeddings = use_case.generate_message_embeddings(&message).await?;
        let _ = self
            .repo_embedding
            .create(&conversation_id, &message.id, &embeddings)
            .await?;
        Ok(())
    }
    async fn sync_embeddings(conversation_id: i32, client_ann: &AnnClient) -> Result<()> {
        // let last_ann_message = client_ann.get_partition_status(conversation_id).await?;
        // let last_ann_message_id = last_ann_message.last_vector_id;
        //
        // let last_db_message = get_last_message_from_conversation(db, last_ann_message_id).await?;
        // let last_db_message = match last_db_message {
        //     Some(msg) => msg,
        //     None => return Err(ErrorBackend::MessageNotFound(last_ann_message_id)),
        // };
        // let last_db_message_id = last_db_message.id;
        // if last_ann_message_id != last_db_message_id {
        //     let embeddings = get_all_embeddings_after_message_id(db, last_ann_message_id).await?;
        //     let embeddings = embeddings.into_vec_ann()?;
        //     let _ = client_ann.embedding_insert_bulk(embeddings).await;
        // }
        Ok(())
    }
}
