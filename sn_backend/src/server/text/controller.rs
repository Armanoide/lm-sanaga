use crate::app_state::AppState;
use crate::error::{Error, ResultAPIStream};
use crate::utils::parse_json_model_id::parse_json_model_id;
use crate::utils::tokio_bridge::TokenBridge;
use axum::Json;
use axum::body::Body;
use axum::extract::State;
use axum::response::Response;
use crossbeam::channel::{Receiver, Sender, bounded};
use futures::StreamExt;
use serde_json::Value;
use sn_core::utils::rw_lock::RwLockExt;
use std::collections::HashMap;
use std::sync::Arc;
use std::thread;
use axum::extract::rejection::JsonRejection;
use tracing::error;
use sn_core::conversation::Conversation;
use sn_core::message::Message;
use sn_core::server::payload::generate_text_request::GenerateTextRequest;
use sn_inference::model::model_runtime::GenerateTextResult;

enum ErrorGenerateText  {
    InvalidRequest(sn_inference::error::Error),
    InternalError(sn_core::error::Error),
}

/*fn parse_generate_text_params(
    json: &HashMap<String, Value>,
) -> Result<(String, String, bool, Option<String>, String), Error> {
    let model_id = parse_json_model_id(json)?;

    let prompt = json
        .get("prompt")
        .and_then(Value::as_str)
        .ok_or_else(|| Error::InvalidRequest("Missing or invalid prompt".to_string()))?
        .to_string();

    // stream is optional,
    let stream = json.get("stream").and_then(Value::as_bool).unwrap_or(false);

    let last_message_id = json
        .get("conversation_id")
        .and_then(Value::as_str)
        .map(|s| s.to_string());

    let user_id = json
        .get("user_id")
        .and_then(Value::as_str)
        .map(|s| s.to_string())
        .unwrap_or_else(|| "default_user".to_string());
    Ok((model_id, prompt, stream, last_message_id, user_id))
}*/


pub async fn save_message_in_db(
    state: &Arc<AppState>,
    conversation: &Conversation,
    payload: &GenerateTextRequest,
    generate_text_result: &GenerateTextResult) {
    // This function should implement the logic to save the message in the database.
    // For now, we will just print a message indicating that the message is being saved.
    println!(
        "Saving message in DB for conversation ID: {:?}",
        payload.last_message_id
    );
    // Here you would typically call your database save function.
}

pub fn handle_error_generate_text(err: &String, tx_err: &Sender<String>) {
    error!("Failed to acquire write lock: {}", err);
    let _ = tx_err.send(format!("[ERROR]: Failed to generate text: {}", err));
}

pub async fn generate_text(
    State(state): State<Arc<AppState>>,
    payload: Result<Json<GenerateTextRequest>, JsonRejection>,
) -> ResultAPIStream {
    let payload = payload?;

    let conversation = Conversation::from_message(Message {
        role: "user".to_string(),
        content: payload.prompt.clone(),
        stats: None,
    });

    let (tx, rx): (Sender<String>, Receiver<String>) = bounded(100);
    let tx_err = tx.clone();

    let bridge = TokenBridge::new(rx);
    let stream = bridge
        .into_stream()
        .map(|msg| Ok::<_, Error>(format!("data: {}\n\n", msg)));
    let body = Body::from_stream(stream);

    tokio::spawn(async move {

        let generate_text_result: Result<_, Error> = {
            state.runner.write_lock("reading runner for generate_text=")
                .map_err(|e| Error::Core(e))
                .and_then(| guard| {
                    guard.generate_text(&payload.model_id, &conversation, Some(tx))
                        .map_err(|e| Error::Inference(e))
                })
        };

        let generate_text_result = if let Err(err) = generate_text_result {
            handle_error_generate_text(&err.to_string(), &tx_err);
            return;
        } else {
            generate_text_result.unwrap()
        };

        save_message_in_db(&state, &conversation, &payload, &generate_text_result).await;

        /*match state.runner.write_lock(context) {
            Ok(guard) => {
                let generate_text_result = {
                    guard.generate_text(&payload.model_id, &conversation, Some(tx))
                };
                if let Err(err) = generate_text_result {
                    handle_error_generate_text(&err.to_string(), &tx_err);
                    return;
                }
                save_message_in_db(&conversation, &payload, &generate_text_result.ok().unwrap()).await;
                /*match guard.generate_text(&payload.model_id, &conversation, Some(tx)) {
                    Ok(generate_state) => {
                        if let Some(_) = payload.last_message_id {
                            save_message_in_db(&conversation, &payload, &generate_state).await;
                        }
                    },
                    Err(err) => handle_error_generate_text(&err.to_string(), &tx_err)
                }*/
            }
            Err(err) => handle_error_generate_text(&err.to_string(), &tx_err)
        }*/
    });

    let response = Response::builder()
        .header("Content-Type", "text/event-stream")
        .body(body)
        .unwrap();
    Ok(response)
}
