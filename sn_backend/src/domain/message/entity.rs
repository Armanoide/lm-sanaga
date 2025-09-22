use rayon::prelude::*;
use sea_orm::entity::prelude::*;
use sea_orm::sqlx::types::chrono::NaiveDateTime;
use serde::Serialize;
use sn_core::types::message::{Message, MessageRole};

use crate::domain;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize)]
#[sea_orm(table_name = "message")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub role: String,
    pub content: String,
    pub generation_duration: Option<f64>,
    pub prompt_tps: Option<f64>,
    pub generation_tps: Option<f64>,
    pub conversation_id: i32,
    pub created_at: NaiveDateTime,
    pub model_id: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "domain::conversation::entity::Entity",
        from = "Column::ConversationId",
        to = "domain::conversation::entity::Column::Id"
    )]
    Conversation,
    #[sea_orm(has_one = "domain::embedding::entity::Entity")]
    Embedding,
}

impl Related<domain::conversation::entity::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Conversation.def()
    }
}

impl Related<domain::embedding::entity::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Embedding.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

pub trait IntoMessage {
    fn into_message(self) -> Message;
}

impl IntoMessage for Model {
    fn into_message(self) -> Message {
        sn_core::types::message::MessageBuilder::default()
            .content(self.content.clone())
            .role(MessageRole::try_from(self.role.as_str()).unwrap_or(MessageRole::User))
            .conversation_id(Some(self.conversation_id))
            .build()
            .unwrap_or_default()
    }
}

pub trait IntoConversation {
    fn into_conversation(self) -> sn_core::types::conversation::Conversation;
}

impl IntoConversation for Vec<Model> {
    fn into_conversation(self) -> sn_core::types::conversation::Conversation {
        sn_core::types::conversation::Conversation {
            name: None,
            id: None,
            messages: self
                .into_par_iter()
                .map(IntoMessage::into_message)
                .collect(),
        }
    }
}
