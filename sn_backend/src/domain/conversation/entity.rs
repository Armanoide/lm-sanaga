use rayon::iter::*;
use sea_orm::entity::prelude::*;
use sea_orm::sqlx::types::chrono::NaiveDateTime;
use serde::Serialize;

use crate::domain;
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize)]
#[sea_orm(table_name = "conversation")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub name: Option<String>,
    pub session_id: i32,
    pub created_at: NaiveDateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "domain::message::entity::Entity")]
    Messages,
    #[sea_orm(has_many = "domain::embedding::entity::Entity")]
    Embeddings,
    #[sea_orm(
        belongs_to = "domain::session::entity::Entity",
        from = "Column::SessionId",
        to = "domain::session::entity::Column::Id"
    )]
    Session,
}

impl Related<domain::message::entity::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Messages.def()
    }
}

impl Related<domain::session::entity::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Session.def()
    }
}

impl Related<domain::embedding::entity::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Embeddings.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

pub trait IntoConversations {
    fn into_conversations(self) -> Vec<sn_core::types::conversation::Conversation>;
}

impl IntoConversations for Vec<crate::domain::conversation::entity::Model> {
    fn into_conversations(self) -> Vec<sn_core::types::conversation::Conversation> {
        self.into_par_iter()
            .map(IntoConversation::into_conversation)
            .collect()
    }
}
pub trait IntoConversation {
    fn into_conversation(self) -> sn_core::types::conversation::Conversation;
}
impl IntoConversation for crate::domain::conversation::entity::Model {
    fn into_conversation(self) -> sn_core::types::conversation::Conversation {
        sn_core::types::conversation::Conversation {
            name: self.name,
            id: Some(self.id),
            messages: vec![],
        }
    }
}
