use std::sync::Arc;

use crate::domain;
use crate::error::{ErrorBackend, Result};
use sea_orm::ColumnTrait;
use sea_orm::QueryFilter;
use sea_orm::{ActiveModelTrait, DatabaseConnection, EntityTrait, ModelTrait, QueryOrder, Set};

#[derive(Clone, Debug)]
pub struct ConversationRepository {
    db: Arc<DatabaseConnection>,
}

impl ConversationRepository {
    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        Self { db }
    }
    pub async fn create(&self, session_id: &i32) -> Result<domain::conversation::entity::Model> {
        let conversation = domain::conversation::entity::ActiveModel {
            id: Default::default(),
            name: Default::default(),
            session_id: Set(*session_id),
            created_at: Default::default(),
        };
        let conversation = conversation.insert(self.db.as_ref()).await?;
        Ok(conversation)
    }

    pub async fn find_by_id(
        &self,
        id: &i32,
    ) -> Result<Option<domain::conversation::entity::Model>> {
        let conversation = domain::conversation::entity::Entity::find_by_id(*id)
            .one(self.db.as_ref())
            .await?;
        Ok(conversation)
    }

    pub async fn find_all_by_session_id(
        &self,
        session_id: &i32,
    ) -> Result<Vec<domain::conversation::entity::Model>> {
        let conversations = domain::conversation::entity::Entity::find()
            .order_by_asc(domain::conversation::entity::Column::CreatedAt)
            .filter(domain::conversation::entity::Column::SessionId.eq(*session_id))
            .all(self.db.as_ref())
            .await?;
        Ok(conversations)
    }

    pub async fn delete_by_id(&self, id: &i32) -> Result<()> {
        let conversation = domain::conversation::entity::Entity::find_by_id(*id)
            .one(self.db.as_ref())
            .await?;
        if let Some(conversation) = conversation {
            conversation.delete(self.db.as_ref()).await?;
        }
        Ok(())
    }

    pub async fn update_name(
        &self,
        id: &i32,
        name: String,
    ) -> Result<domain::conversation::entity::Model> {
        let conversation = domain::conversation::entity::Entity::find_by_id(*id)
            .one(self.db.as_ref())
            .await?
            .ok_or_else(|| ErrorBackend::ConversationNotFound)?;

        let mut conversation: domain::conversation::entity::ActiveModel = conversation.into();
        conversation.name = Set(Some(name));
        let updated_conversation = conversation.update(self.db.as_ref()).await?;
        Ok(updated_conversation)
    }

    pub async fn has_name(&self, id: &i32) -> Result<bool> {
        let conversation = domain::conversation::entity::Entity::find_by_id(*id)
            .one(self.db.as_ref())
            .await?
            .ok_or_else(|| ErrorBackend::ConversationNotFound)?;
        Ok(conversation.name.is_some())
    }
}
