use rayon::iter::IntoParallelRefIterator;
use sea_orm::entity::prelude::*;
use sea_orm::sqlx::types::chrono::NaiveDateTime;
use serde::Serialize;
use rayon::iter::*;
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
    #[sea_orm(has_many="super::message::Entity")]
    Messages,
    #[sea_orm(belongs_to="super::session::Entity", from = "Column::SessionId", to = "super::session::Column::Id")]
    Session,
}

impl Related<super::message::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Messages.def()
    }
}

impl Related<super::session::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Session.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

pub trait Convert {
    fn into_conversations(self) -> Vec<sn_core::types::conversation::Conversation>;
}

impl Convert for Vec<crate::db::entities::conversation::Model> {
    fn into_conversations(self) -> Vec<sn_core::types::conversation::Conversation>{
        self.par_iter().map(|c| sn_core::types::conversation::Conversation {
            name: c.name.clone(),
            id: Some(c.id),
            messages: vec![],
        }).collect()
    }
}