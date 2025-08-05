use sea_orm::{ActiveModelTrait, DatabaseConnection, EntityTrait, ModelTrait, QueryOrder, Set};
use crate::db;
use crate::error::{Result};
use sea_orm::QueryFilter;
use sea_orm::ColumnTrait;

/// Creates a new conversation associated with the given session ID.
///
/// # Arguments
/// * `db` - A reference to the database connection.
/// * `session_id` - The session ID to associate the new conversation with.
///
/// # Returns
/// * `Ok(Model)` - The newly created conversation model.
/// * `Err` - If an error occurs during insertion.
pub async fn create_conversation(db: &DatabaseConnection, session_id: &i32) -> Result<db::entities::conversation::Model> {
    let conversation = db::entities::conversation::ActiveModel {
        id: Default::default(),
        name: Default::default(),
        session_id: Set(session_id.clone()),
        created_at: Default::default(),
    };
    let conversation = conversation.insert(db).await?;
    Ok(conversation)
}

/// Retrieves a conversation by its ID.
///
/// # Arguments
/// * `db` - A reference to the database connection.
/// * `id` - The ID of the conversation to retrieve.
///
/// # Returns
/// * `Ok(Some(Model))` - If the conversation is found.
/// * `Ok(None)` - If no conversation with the given ID exists.
/// * `Err` - If a database error occurs.
pub async fn get_conversation_by_id(db: &DatabaseConnection, id: &i32) -> Result<Option<db::entities::conversation::Model>> {
    let conversation = db::entities::conversation::Entity::find_by_id(id.clone()).one(db).await?;
    Ok(conversation)
}

/// Retrieves all conversations associated with a specific session, ordered by creation time.
///
/// # Arguments
/// * `db` - A reference to the database connection.
/// * `session_id` - The session ID to filter conversations by.
///
/// # Returns
/// * `Ok(Vec<Model>)` - A list of conversations sorted by creation time (ascending).
/// * `Err` - If a database error occurs.
pub async fn get_conversations_by_session_id(db: &DatabaseConnection, session_id: i32) -> Result<Vec<db::entities::conversation::Model>> {
    let conversations = db::entities::conversation::Entity::find()
        .order_by_asc(db::entities::conversation::Column::CreatedAt)
        .filter(db::entities::conversation::Column::SessionId.eq(session_id))
        .all(db)
        .await?;
    Ok(conversations)
}