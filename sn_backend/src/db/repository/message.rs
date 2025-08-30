use crate::db::entities;
use crate::db::repository::conversation::{create_conversation, get_conversation_by_id};
use crate::db::repository::session::get_session;
use crate::error::{ErrorBackend, Result};
use axum::Json;
use sea_orm::{ActiveModelTrait, DatabaseConnection, EntityTrait, ModelTrait, QueryOrder, Set};
use sn_core::server::payload::generate_text_request::GenerateTextRequest;
use sn_core::types::message::MessageRole;
use sn_inference::model::model_runtime::GenerateTextResult;

/// Creates two messages (user and assistant) for a given session and conversation.
///
/// - Validates and retrieves or creates the session and conversation.
/// - Inserts the user's prompt as a message.
/// - Inserts the assistant's generated response as another message.
///
/// # Arguments
/// * `db` - A reference to the database connection.
/// * `payload` - The request payload containing prompt details and metadata.
/// * `generate_text_result` - The generated text and related stats from the model.
///
/// # Returns
/// * `Ok(Some(Model))` - The assistant's message if all went well.
/// * `Ok(None)` - If session ID was missing or invalid.
/// * `Err` - If any database operation fails.
pub async fn create_message(
    db: &DatabaseConnection,
    payload: &Json<GenerateTextRequest>,
    generate_text_result: &GenerateTextResult,
) -> Result<Option<(entities::message::Model, entities::message::Model)>> {
    let GenerateTextRequest {
        session_id,
        conversation_id,
        prompt,
        ..
    } = &payload.0;

    let session_id = match session_id {
        Some(id) => id.clone(),
        None => return Ok(None),
    };

    let session = match get_session(db, session_id.clone()).await? {
        Some(s) => s,
        // It's possible that the session_id is not provided.
        None => return Ok(None),
    };

    let conversation_id = match conversation_id {
        Some(id) => id,
        None => &0,
    };

    // Attempt to retrieve the conversation, or create a new one if it doesn't exist
    let conversation = match get_conversation_by_id(&db, &conversation_id).await? {
        Some(conv) => conv,
        None => create_conversation(&db, &session.id).await?,
    };

    let new_message_user = entities::message::ActiveModel {
        conversation_id: Set(conversation.id),
        content: Set(prompt.clone()),
        role: Set(MessageRole::User.to_string()),
        ..Default::default()
    };
    let new_message_user = new_message_user.insert(db).await?;

    let stats = &generate_text_result.1.clone().unwrap_or_default();

    let new_message_assistant = entities::message::ActiveModel {
        conversation_id: Set(conversation.id),
        content: Set(generate_text_result.0.clone()),
        prompt_tps: Set(Some(stats.prompt_tps)),
        generation_tps: Set(Some(stats.generation_tps)),
        generation_duration: Set(Some(stats.generation_duration)),
        role: Set(MessageRole::Assistant.to_string()),
        ..Default::default()
    };
    let new_message_assistant = new_message_assistant.insert(db).await?;
    Ok(Some((new_message_user, new_message_assistant)))
}

/// Retrieves a single message by its ID from the database.
///
/// # Arguments
/// * `db` - A reference to the database connection.
/// * `message_id` - The ID of the message to retrieve.
///
/// # Returns
/// * `Ok(Some(Model))` - If the message exists.
/// * `Ok(None)` - If no message with the given ID exists.
/// * `Err` - If a database error occurs.
pub async fn get_message_by_id(
    db: &DatabaseConnection,
    message_id: i32,
) -> Result<Option<entities::message::Model>> {
    entities::message::Entity::find_by_id(message_id)
        .one(db)
        .await
        .map_err(ErrorBackend::from)
}

/// Retrieves all messages associated with a conversation ID from the request payload,
/// ordered by creation time in ascending order.
///
/// # Arguments
/// * `db` - A reference to the database connection.
/// * `payload` - The request payload containing the conversation ID.
///
/// # Returns
/// * `Ok(Some(Vec<Model>))` - A list of messages for the conversation.
/// * `Ok(None)` - If the conversation ID is missing or invalid.
/// * `Err` - If a database error occurs.
pub async fn get_messages_from_payload(
    db: &DatabaseConnection,
    payload: &Json<GenerateTextRequest>,
) -> Result<Option<Vec<entities::message::Model>>> {
    let GenerateTextRequest {
        conversation_id, ..
    } = payload.0;

    let conversation_id = match conversation_id {
        Some(id) => id,
        None => return Ok(None),
    };

    let conversation = match get_conversation_by_id(db, &conversation_id).await? {
        Some(conv) => conv,
        None => return Ok(None),
    };

    let messages = conversation
        .find_related(entities::message::Entity)
        .order_by_asc(entities::message::Column::CreatedAt)
        .all(db)
        .await?;

    Ok(Some(messages))
}
