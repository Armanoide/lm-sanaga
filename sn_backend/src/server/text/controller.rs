use crate::db;
use crate::db::entities::message::Convert;
use crate::db::repository;
use crate::db::repository::conversation::update_conversation_name;
use crate::db::repository::message::create_message;
use crate::error::Result;
use crate::error::{Error, ResultAPIStream};
use crate::server::app_state::AppState;
use crate::utils::sse_response_builder::SseResponseBuilder;
use axum::Json;
use axum::extract::State;
use axum::extract::rejection::JsonRejection;
use axum::response::IntoResponse;
use crossbeam::channel::{Receiver, Sender, bounded};
use sea_orm::DatabaseConnection;
use serde_json::json;
use sn_core::server::payload::generate_text_request::GenerateTextRequest;
use sn_core::server::payload::text_generated_metadata_response_sse::TextGeneratedMetadataResponseSSE;
use sn_core::types::conversation::Conversation;
use sn_core::types::message::{Message, MessageRole};
use sn_core::types::stream_data::StreamData;
use sn_core::utils::rw_lock::RwLockExt;
use sn_inference::model::model_runtime::GenerateTextResult;
use std::sync::Arc;
use tracing::error;

pub async fn create_or_get_conversation(
    db: Option<&DatabaseConnection>,
    payload: &Json<GenerateTextRequest>,
) -> Result<Conversation> {
    let db = match db {
        Some(db) => db,
        None => return Ok(Conversation::default()),
    };

    let messages = repository::message::get_messages_from_payload(db, payload).await?;
    match messages {
        Some(messages) => Ok(messages.into_conversation()),
        None => Ok(Conversation::default()),
    }
}

/**
///
/// Generate Text
///
**/

fn handle_error_generate_text(err: &String, tx_err: Option<Arc<Sender<StreamData>>>) {
    error!("{}", err);
    let error = format!("Failed to generate text: {}", err);
    if let Some(tx_err) = tx_err {
        let _ = tx_err.send(StreamData::for_stream_error(error).into());
    }
}

/// Attempts to generate a short title for a conversation using a text generation model,
/// then updates the conversation's name in the database.
///
/// This function:
/// - Reads the AI runner from shared state.
/// - Uses the provided prompt to generate a concise title (4 words).
/// - Updates the conversation record in the database with the generated title.
///
/// # Parameters
/// - `state`: Shared application state (`Arc<AppState>`) containing the AI runner and DB connection.
/// - `payload`: JSON payload containing the prompt and model ID for generation.
/// - `conversation_id`: ID of the conversation to update.
///
/// # Behavior
/// - If the database connection is missing (`state.db` is `None`), the function returns early.
/// - If text generation fails, it logs an error but does not panic.
/// - If title generation is successful, it attempts to update the conversation name asynchronously.
///
/// ```
async fn generate_title_conversation(
    state: Arc<AppState>,
    payload: Json<GenerateTextRequest>,
    conversation_id: i32,
) {
    let db = if let Some(db) = &state.db {
        db
    } else {
        return;
    };
    let generate_text_result = (|| {
        let guard = state.runner.read_lock("reading runner for resume")?;
        let conversation = Conversation::from_user_with_content(format!(
            "resume with with 4 words only: {}",
            payload.prompt
        ));
        let generate_text_result =
            guard.generate_text(&payload.model_id, &conversation, None, None)?;
        Ok::<_, Error>(generate_text_result)
    })();
    match generate_text_result {
        Ok(result) => {
            let (title_conversation, _) = result;
            let title_conversation = Message::sanitize_content(title_conversation);
            let _ = update_conversation_name(db, &conversation_id, title_conversation).await;
        }
        Err(err) => {
            error!("Failed to generate title for conversation: {}", err);
        }
    }
}

/// Saves the result of a text generation operation to the database
/// and optionally sends metadata over a stream.
///
/// Returns the saved database message model on success, or None on failure.
async fn store_generate_text_result(
    state: Arc<AppState>,
    payload: Json<GenerateTextRequest>,
    generate_text_result: GenerateTextResult,
    tx: Option<Arc<Sender<StreamData>>>,
) -> Option<db::entities::message::Model> {
    let db = if let Some(db) = &state.db {
        db
    } else {
        return None;
    };

    match create_message(db, &payload, &generate_text_result).await {
        Ok(message) => {
            if let (Some(message), Some(tx)) = (&message, tx) {
                let _ = tx.send(StreamData::for_text_generated_metadata_sse_response(
                    TextGeneratedMetadataResponseSSE {
                        prompt_tps: message.prompt_tps,
                        generation_tps: message.generation_tps,
                        conversation_id: message.conversation_id,
                    },
                ));
                if payload.conversation_id.is_none() {
                    let conversation_id = message.conversation_id.clone();
                    tokio::spawn(generate_title_conversation(state, payload, conversation_id));
                }
            }
            message
        }
        Err(err) => {
            handle_error_generate_text(&err.to_string(), tx);
            None
        }
    }
}

async fn generate_text_with_sse(
    state: Arc<AppState>,
    payload: Json<GenerateTextRequest>,
    conversation: Conversation,
) -> ResultAPIStream {
    let (tx, rx): (Sender<StreamData>, Receiver<StreamData>) = bounded(100);
    let tx = Arc::new(tx);

    let response = SseResponseBuilder::new(rx).build();

    tokio::spawn(async move {
        let generate_text_result = (|| {
            let guard = state.runner.read_lock("reading runner for generate_text")?;
            let generate_text_result = guard.generate_text(
                &payload.model_id,
                &conversation,
                payload.session_id,
                Some(tx.clone()),
            )?;
            Ok::<_, Error>(generate_text_result)
        })();

        let generate_text_result = if let Err(err) = generate_text_result {
            handle_error_generate_text(&err.to_string(), Some(tx));
            return;
        } else {
            //todo: handle the case where generate_text_result is None
            generate_text_result.unwrap()
        };

        store_generate_text_result(state, payload, generate_text_result, Some(tx)).await;
    });

    Ok(response?)
}

async fn generate_text_with_json(
    state: Arc<AppState>,
    payload: Json<GenerateTextRequest>,
    conversation: Conversation,
) -> ResultAPIStream {
    let generate_text_result = {
        state
            .runner
            .read_lock("reading runner for generate_text")
            .map_err(|e| Error::Core(e))
            .and_then(|guard| {
                guard
                    .generate_text(&payload.model_id, &conversation, payload.session_id, None)
                    .map_err(|e| Error::Inference(e))
            })
    }?;

    let message = store_generate_text_result(state, payload, generate_text_result, None).await;
    if let Some(message) = message {
        Ok(Json(json!({
            "content": message.content,
            "generation_tps": message.generation_tps,
            "prompt_tps": message.prompt_tps,
            "conversation_id": message.conversation_id,
        }))
        .into_response())
    } else {
        Err(Error::FailedToGenerateText(json!({
            "error": "Failed to save generated text",
            "reason": "Could not persist message to database or inference result"
        })))
    }
}

/// Handles a text generation request, optionally streaming the response.
///
/// This endpoint processes a user's prompt and either returns a complete JSON
/// response or streams the generated text using Server-Sent Events (SSE),
/// depending on the `stream` flag in the request payload.
///
/// # Parameters
///
/// - `state`: Shared application state wrapped in `Arc<AppState>`, which gives
///   access to database, model runner, and other global services.
/// - `payload`: A `Result` wrapping a validated JSON request body (`GenerateTextRequest`).
///   If deserialization fails, an early `400 Bad Request` response is returned.
///
/// # Returns
///
/// A `ResultAPIStream` that either:
/// - Returns a full JSON response containing the generated output, or
/// - Streams the generated tokens progressively via SSE (if `stream = true`)
///
/// # Behavior
///
/// - Looks up or creates a conversation based on the input payload.
/// - Appends the user's prompt as a message in the conversation.
/// - Delegates to either `generate_text_with_json` or `generate_text_with_sse`
///   depending on the `stream` flag.
///
/// # Errors
///
/// Returns a 4xx or 5xx error if:
/// - The input payload is invalid.
/// - The conversation cannot be fetched or created.
/// - The text generation process fails internally.
pub async fn generate_text(
    State(state): State<Arc<AppState>>,
    payload: std::result::Result<Json<GenerateTextRequest>, JsonRejection>,
) -> ResultAPIStream {
    let payload = payload?;
    let mut conversation = create_or_get_conversation(state.db.as_ref(), &payload).await?;
    conversation.messages.push(Message {
        role: MessageRole::User,
        content: payload.prompt.clone(),
        stats: None,
    });

    if payload.stream.unwrap_or(false) {
        generate_text_with_sse(state, payload, conversation).await
    } else {
        generate_text_with_json(state, payload, conversation).await
    }
}
