use crate::domain::message::value_object::GenerateTextOutput;
use crate::error::{ErrorBackend, ResultAPIStream};
use crate::server::app_state::AppState;
use crate::utils::sse_response_builder::SseResponseBuilder;
use axum::extract::rejection::JsonRejection;
use axum::extract::State;
use axum::response::IntoResponse;
use axum::Json;
use serde_json::json;
use sn_core::server::payload::backend::generate_text_request::GenerateTextRequest;
use std::sync::Arc;

/**
///
/// Generate Text
///
**/

/// /// Saves the result of a text generation operation to the database
/// /// and optionally sends metadata over a stream.
/// ///
/// /// Returns the saved database message model on success, or None on failure.
/// async fn store_generate_text_result(
///     state: Arc<AppState>,
///     payload: Json<GenerateTextRequest>,
///     generate_text_result: GenerateTextResult,
///     tx: Option<Arc<Sender<StreamData>>>,
/// ) -> Option<(db::entities::message::Model, db::entities::message::Model)> {
///     let db = match &state.db {
///         Some(db) => db,
///         None => return None,
///     };
///
///     // we store the user message & reponse ia
///     match create_message(db, &payload, &generate_text_result).await {
///         Ok(store_messages) => {
///             if let (Some((message_user, message_assistant)), Some(tx)) = (&store_messages, tx) {
///                 let _ = tx.send(StreamData::for_text_generated_metadata_sse_response(
///                     TextGeneratedMetadataResponseSSE {
///                         prompt_tps: message_assistant.prompt_tps,
///                         generation_tps: message_assistant.generation_tps,
///                         conversation_id: message_assistant.conversation_id,
///                     },
///                 ));
///                 let conversation_id = message_user.conversation_id.clone();
///                 let message_user_id = message_user.id.clone();
///                 let message_assistant_id = message_assistant.id.clone();
///                 debug!("Spawning task to generate conversation title and embeddings");
///                 tokio::spawn(handle_post_store_generate_text(
///                     state,
///                     payload,
///                     conversation_id,
///                     message_user_id,
///                     message_assistant_id,
///                 ));
///             }
///             store_messages
///         }
///         Err(err) => {
///             handle_error_generate_text(&err.to_string(), tx);
///             None
///         }
///     }
/// }

pub async fn generate_text_handler(
    State(state): State<Arc<AppState>>,
    req: std::result::Result<Json<GenerateTextRequest>, JsonRejection>,
) -> ResultAPIStream {
    let req = req?.0;
    let result = state.service_message.generate_text(req).await?;
    match result {
        GenerateTextOutput::Json(agg) => {
            if let Some(message_assistant) = agg.get_assistant_message() {
                return Ok(Json(json!(message_assistant)).into_response());
            }
            return Err(ErrorBackend::FailedToGenerateText(
                "No assistant message generated".into(),
            ));
        }
        GenerateTextOutput::Streaming { receiver, .. } => {
            if let Some(receiver) = receiver {
                return SseResponseBuilder::new(receiver).build();
            }
            return Err(ErrorBackend::FailedToGenerateText(
                "No streaming receiver available".into(),
            )
            .into());
        }
    }
}
