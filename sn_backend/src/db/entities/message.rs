use rayon::prelude::*;
use sea_orm::entity::prelude::*;
use sea_orm::sqlx::types::chrono::NaiveDateTime;
use serde::Serialize;
use sn_core::types::message::MessageRole;

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
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::conversation::Entity",
        from = "Column::ConversationId",
        to = "super::conversation::Column::Id"
    )]
    Conversation,
    #[sea_orm(has_one = "super::embedding::Entity")]
    Embedding,
}

impl Related<super::conversation::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Conversation.def()
    }
}

impl Related<super::embedding::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Embedding.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

pub trait Convert {
    fn into_conversation(self) -> sn_core::types::conversation::Conversation;
}
impl Convert for Vec<Model> {
    fn into_conversation(self) -> sn_core::types::conversation::Conversation {
        sn_core::types::conversation::Conversation {
            name: None,
            id: None,
            messages: self
                .par_iter()
                .map(|m| {
                    sn_core::types::message::MessageBuilder::default()
                        .content(m.content.clone())
                        .role(MessageRole::try_from(m.role.as_str()).unwrap_or(MessageRole::User))
                        .build()
                        .unwrap_or_default()
                })
                .collect(),
        }
    }
}
