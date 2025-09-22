use sea_orm::entity::prelude::*;
use sea_orm::sqlx::types::chrono::NaiveDateTime;
use serde::Serialize;
use sn_core::types::ann_item::AnnItem;

use crate::domain;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize)]
#[sea_orm(table_name = "embedding")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub message_id: i32,
    pub conversation_id: i32,
    pub created_at: NaiveDateTime,
    pub data: Json,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "domain::message::entity::Entity",
        from = "Column::MessageId",
        to = "domain::message::entity::Column::Id"
    )]
    Message,
    #[sea_orm(
        belongs_to = "domain::conversation::entity::Entity",
        from = "Column::ConversationId",
        to = "domain::conversation::entity::Column::Id"
    )]
    Conversation,
}

impl Related<domain::message::entity::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Message.def()
    }
}

impl Related<domain::conversation::entity::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Conversation.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

pub trait Convert {
    fn into_vec_ann(&self) -> crate::error::Result<Vec<AnnItem>>;
}

impl Convert for Vec<Model> {
    fn into_vec_ann(&self) -> crate::error::Result<Vec<AnnItem>> {
        let items = self
            .iter()
            .map(|emb| {
                let vectors = serde_json::from_value(emb.data.clone())?;
                let item = AnnItem {
                    primary_key: emb.message_id,
                    partition_id: emb.conversation_id,
                    vectors,
                };
                Ok(item)
            })
            .collect::<crate::error::Result<Vec<AnnItem>>>()?;
        Ok(items)
    }
}
