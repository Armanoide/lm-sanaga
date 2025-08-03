use sea_orm::{ActiveModelTrait, DatabaseConnection, EntityTrait, ModelTrait, Set};
use crate::db;
use crate::error::{Result};

pub async fn create_conversation(db: &DatabaseConnection, session_id: &i32) -> Result<db::entities::conversation::Model> {
    let conversation = db::entities::conversation::ActiveModel {
        id: Default::default(),
        name: Default::default(),
        session_id: Set(session_id.clone()),
    };
    let conversation = conversation.insert(db).await?;
    Ok(conversation)
}

pub async fn get_conversation_by_id(db: &DatabaseConnection, id: &i32) -> Result<Option<db::entities::conversation::Model>> {
    let conversation = db::entities::conversation::Entity::find_by_id(id.clone()).one(db).await?;
    Ok(conversation)
}
