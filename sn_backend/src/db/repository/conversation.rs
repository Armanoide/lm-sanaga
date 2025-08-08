use crate::db;
use crate::error::{Error, Result};
use sea_orm::ColumnTrait;
use sea_orm::QueryFilter;
use sea_orm::{ActiveModelTrait, DatabaseConnection, EntityTrait, ModelTrait, QueryOrder, Set};

/// Creates a new conversation associated with the given session ID.
///
/// # Arguments
/// * `db` - A reference to the database connection.
/// * `session_id` - The session ID to associate the new conversation with.
///
/// # Returns
/// * `Ok(Model)` - The newly created conversation model.
/// * `Err` - If an error occurs during insertion.
pub async fn create_conversation(
    db: &DatabaseConnection,
    session_id: &i32,
) -> Result<db::entities::conversation::Model> {
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
pub async fn get_conversation_by_id(
    db: &DatabaseConnection,
    id: &i32,
) -> Result<Option<db::entities::conversation::Model>> {
    let conversation = db::entities::conversation::Entity::find_by_id(id.clone())
        .one(db)
        .await?;
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
pub async fn get_conversations_by_session_id(
    db: &DatabaseConnection,
    session_id: i32,
) -> Result<Vec<db::entities::conversation::Model>> {
    let conversations = db::entities::conversation::Entity::find()
        .order_by_asc(db::entities::conversation::Column::CreatedAt)
        .filter(db::entities::conversation::Column::SessionId.eq(session_id))
        .all(db)
        .await?;
    Ok(conversations)
}

/// Deletes a conversation from the database by its ID, if it exists.
///
/// This function performs the following:
/// - Attempts to find the conversation with the given `id`.
/// - If found, deletes it from the database.
/// - If not found, it silently does nothing (no error is returned).
///
/// # Parameters
/// - `db`: Reference to the database connection.
/// - `id`: ID of the conversation to delete.
///
/// # Returns
/// - `Ok(())` if the operation succeeded or the conversation was not found.
/// - `Err` if there was a database error during lookup or deletion.
///
/// # Errors
/// Returns a `DbErr` (or your crate's error type via `Result`) if:
/// - The conversation lookup fails.
/// - The deletion fails.
///
/// ```
pub async fn delete_conversation_by_id(db: &DatabaseConnection, id: &i32) -> Result<()> {
    let conversation = db::entities::conversation::Entity::find_by_id(*id)
        .one(db)
        .await?;
    if let Some(conversation) = conversation {
        conversation.delete(db).await?;
    }
    Ok(())
}

/// Updates the name of a conversation in the database by its ID.
///
/// This function:
/// - Looks up the conversation by ID.
/// - If found, updates its `name` field with the provided `name`.
/// - Trims whitespace and removes all newline (`\n`, `\r`) characters from the name before saving.
/// - Returns the updated conversation model.
///
/// # Parameters
/// - `db`: Reference to the database connection.
/// - `id`: ID of the conversation to update.
/// - `name`: New name for the conversation (will be cleaned of newlines).
///
/// # Returns
/// - `Ok(Model)` with the updated conversation on success.
/// - `Err(Error::ConversationNotFound)` if the conversation does not exist.
/// - `Err(DbErr)` if the database update fails.
///
/// # Errors
/// This function returns an error if:
/// - The conversation is not found.
/// - There is a database failure during the update.
///
/// ```
pub async fn update_conversation_name(
    db: &DatabaseConnection,
    id: &i32,
    name: String,
) -> Result<db::entities::conversation::Model> {
    let mut conversation = db::entities::conversation::Entity::find_by_id(*id)
        .one(db)
        .await?
        .ok_or_else(|| Error::ConversationNotFound)?;

    let mut conversation: db::entities::conversation::ActiveModel = conversation.into();
    conversation.name = Set(Some(name.trim().replace('\n', "").replace('\r', "")));
    let updated_conversation = conversation.update(db).await?;
    Ok(updated_conversation)
}
