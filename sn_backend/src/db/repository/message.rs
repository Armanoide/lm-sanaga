use axum::Json;
use sea_orm::{ActiveModelTrait, DatabaseConnection, EntityTrait, Iden, ModelTrait, QueryOrder, Set};
use sn_core::types::conversation::Conversation;
use sn_core::types::message::{ROLE_ASSISTANT, ROLE_USER};
use sn_core::server::payload::generate_text_request::GenerateTextRequest;
use sn_inference::model::model_runtime::GenerateTextResult;
use crate::db;
use crate::db::{entities, repository};
use crate::db::repository::conversation::{create_conversation, get_conversation_by_id};
use crate::db::repository::session::get_session;
use crate::error::{Result, Error};

pub async fn get_conversation_id_from_last_message_id(
    db: &DatabaseConnection,
    last_message_id: &i32
) -> Result<Option<i32>> {
    if let Some(last_message) = entities::message::Entity::find_by_id(*last_message_id)
        .one(db).await? {
        return Ok(Some(last_message.conversation_id))
    }
    Ok(None)
}

pub async fn create_message(
    db: &DatabaseConnection,
    payload: &Json<GenerateTextRequest>,
    generate_text_result: &GenerateTextResult
) -> Result<(Option<entities::message::Model>)> {

    let GenerateTextRequest { session_id, last_message_id, prompt, .. } = &payload.0;

    let session_id = match session_id {
        Some(id) => id.clone(),
        None => return Ok(None),
    };

    let session = match get_session(db, session_id.clone()).await? {
        Some(s) => s,
        // It's possible that the session_id is not provided.
        None => return Ok(None),
    };


    let last_message_id = last_message_id.unwrap_or_default();

    let conversation_id = match get_conversation_id_from_last_message_id(&db, &last_message_id).await? {
        Some(conv) => conv,
        None => create_conversation(&db, &session.id).await?.id
    };

    let new_message_user = entities::message::ActiveModel {
        conversation_id: Set(conversation_id),
        content: Set(prompt.clone()),
        role: Set(ROLE_USER.to_string()),
        ..Default::default()
    };
    let _ = new_message_user.insert(db).await?;

    let stats = &generate_text_result.1.clone().unwrap_or_default();

    let new_message_assistant = entities::message::ActiveModel {
        conversation_id: Set(conversation_id),
        content: Set(generate_text_result.0.clone()),
        prompt_tps: Set(Some(stats.prompt_tps)),
        generation_tps: Set(Some(stats.generation_tps)),
        generation_duration: Set(Some(stats.generation_duration)),
        role: Set(ROLE_ASSISTANT.to_string()),
        ..Default::default()
    };
    Ok(Some(new_message_assistant.insert(db).await?))
}

pub async fn get_message_by_id(
    db: &DatabaseConnection,
    message_id: i32
) -> Result<Option<entities::message::Model>> {
    entities::message::Entity::find_by_id(message_id)
        .one(db).await
        .map_err(Error::from)
}


pub async fn get_messages_from_payload(
    db: &DatabaseConnection,
    payload: &Json<GenerateTextRequest>
) -> Result<Option<Vec<entities::message::Model>>> {

    let GenerateTextRequest { last_message_id,.. } = payload.0;

    let last_message_id = match last_message_id {
        Some(id) => id,
        None => return Ok(None),
    };

    let message = match get_message_by_id(db, last_message_id).await? {
        Some(m) => m,
        None => return Ok(None),
    };


    let conversation = match get_conversation_by_id(db, &message.conversation_id).await? {
        Some(conv) => conv,
        None => return Ok(None),
    };

    let messages = conversation.find_related(entities::message::Entity)
        .order_by_asc(entities::message::Column::CreatedAt)
        .all(db)
        .await?;

    Ok(Some(messages))
}