use sea_orm::entity::prelude::*;
use serde::Serialize;
use sn_core::types::session::Session;

use crate::domain;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize)]
#[sea_orm(table_name = "session")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub name: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "domain::conversation::entity::Entity")]
    Conversation,
}

impl Related<domain::conversation::entity::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Conversation.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

pub trait IntoSession {
    fn into_session(self) -> Session;
}

impl IntoSession for Model {
    fn into_session(self) -> Session {
        Session {
            id: self.id,
            name: self.name,
        }
    }
}
