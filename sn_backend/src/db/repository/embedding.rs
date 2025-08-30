use crate::db::entities;
use crate::db::repository::message::get_message_by_id;
use crate::error::{ErrorBackend, Result};
use sea_orm::ActiveValue::Set;
use sea_orm::{ActiveModelTrait, DatabaseConnection};
use serde_json::json;

pub async fn create_embedding(
    db: &DatabaseConnection,
    message_id: i32,
    emb: &[f32],
) -> Result<Option<entities::embedding::Model>> {
    let message = match get_message_by_id(&db, message_id).await? {
        Some(message) => message,
        _ => return Err(ErrorBackend::MessageNotFound(message_id)),
    };
    let data = json!(emb);
    let new_embedding = entities::embedding::ActiveModel {
        conversation_id: Set(message.conversation_id),
        message_id: Set(message.id),
        data: Set(data),
        ..Default::default()
    };
    Ok(Some(new_embedding.insert(db).await?))
}
