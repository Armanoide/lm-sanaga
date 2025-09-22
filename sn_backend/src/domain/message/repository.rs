use std::sync::Arc;

use crate::domain;
use crate::error::Result;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder, Set,
};
use sn_core::types::message::MessageRole;
use sn_core::types::message_stats::MessageStats;

#[derive(Clone, Debug)]
pub struct MessageRepository {
    db: Arc<DatabaseConnection>,
}

impl MessageRepository {
    pub fn new(db: Arc<DatabaseConnection>) -> MessageRepository {
        MessageRepository { db }
    }
    pub async fn find_by_id(&self, id: &i32) -> Result<Option<domain::message::entity::Model>> {
        let message = domain::message::entity::Entity::find_by_id(*id)
            .one(self.db.as_ref())
            .await?;
        Ok(message)
    }

    pub async fn create(
        &self,
        conversation_id: &i32,
        assistant_content: String,
        assistant_stats: Option<MessageStats>,
        user_content: String,
    ) -> Result<(
        domain::message::entity::Model,
        domain::message::entity::Model,
    )> {
        let new_message_user = domain::message::entity::ActiveModel {
            conversation_id: Set(*conversation_id),
            content: Set(user_content),
            role: Set(MessageRole::User.to_string()),
            ..Default::default()
        };
        let new_message_user = new_message_user.insert(self.db.as_ref()).await?;

        let stats = &assistant_stats.unwrap_or_default();

        let new_message_assistant = domain::message::entity::ActiveModel {
            conversation_id: Set(*conversation_id),
            content: Set(assistant_content),
            prompt_tps: Set(Some(stats.prompt_tps)),
            generation_tps: Set(Some(stats.generation_tps)),
            generation_duration: Set(Some(stats.generation_duration)),
            role: Set(MessageRole::Assistant.to_string()),
            ..Default::default()
        };
        let new_message_assistant = new_message_assistant.insert(self.db.as_ref()).await?;
        Ok((new_message_user, new_message_assistant))
    }

    pub async fn find_all_by_conversation_id(
        &self,
        conversation_id: &i32,
    ) -> Result<Vec<domain::message::entity::Model>> {
        let messages = domain::message::entity::Entity::find()
            .filter(domain::message::entity::Column::ConversationId.eq(*conversation_id))
            .order_by_asc(domain::message::entity::Column::CreatedAt)
            .all(self.db.as_ref())
            .await?;
        Ok(messages)
    }

    // pub async fn find_last_by_conversation_id(
    //     &self,
    //     conversation_id: &i32,
    // ) -> Result<Option<domain::message::entity::Model>> {
    //     let message = domain::message::entity::Entity::find()
    //         .filter(domain::message::entity::Column::ConversationId.eq(*conversation_id))
    //         .order_by_asc(domain::message::entity::Column::CreatedAt)
    //         .one(self.db.as_ref())
    //         .await?;
    //     Ok(message)
    // }

    pub async fn find_first_of_conversation(
        &self,
        conversation_id: &i32,
    ) -> Result<Option<domain::message::entity::Model>> {
        let message = domain::message::entity::Entity::find()
            .filter(domain::message::entity::Column::ConversationId.eq(*conversation_id))
            .order_by_asc(domain::message::entity::Column::CreatedAt)
            .one(self.db.as_ref())
            .await?;
        Ok(message)
    }
}
