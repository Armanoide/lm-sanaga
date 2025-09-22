use std::sync::Arc;

use crate::domain;
use crate::error::Result;
use sea_orm::sqlx::types::chrono::NaiveDateTime;
use sea_orm::ActiveValue::Set;
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use serde_json::json;

#[derive(Clone, Debug)]
pub struct EmbeddingRepository {
    db: Arc<DatabaseConnection>,
}

impl EmbeddingRepository {
    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        EmbeddingRepository { db }
    }

    pub async fn create(
        &self,
        conversation_id: &i32,
        message_id: &i32,
        emb: &[f32],
    ) -> Result<Option<domain::embedding::entity::Model>> {
        let data = json!(emb);
        let new_embedding = domain::embedding::entity::ActiveModel {
            conversation_id: Set(*conversation_id),
            message_id: Set(*message_id),
            data: Set(data),
            ..Default::default()
        };
        Ok(Some(new_embedding.insert(self.db.as_ref()).await?))
    }

    pub async fn find_all_embeddings_after_message_id(
        &self,
        message_id: &i32,
        created_at: NaiveDateTime,
    ) -> Result<Vec<domain::embedding::entity::Model>> {
        let embeddings = domain::embedding::entity::Entity::find()
            .filter(domain::embedding::entity::Column::MessageId.gt(*message_id))
            .filter(domain::embedding::entity::Column::CreatedAt.gt(created_at))
            .all(self.db.as_ref())
            .await?;
        Ok(embeddings)
    }
}
