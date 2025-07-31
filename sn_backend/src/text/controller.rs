use crate::app_state::AppState;
use crate::error::{Error, ResultAPIStream};
use crate::utils::tokio_bridge::TokenBridge;
use axum::Json;
use axum::body::Body;
use axum::extract::State;
use axum::response::Response;
use crossbeam::channel::{Receiver, Sender, bounded};
use futures::StreamExt;
use serde_json::Value;
use sn_core::conversation::conversation::{Conversation, Message};
use sn_core::utils::rw_lock::RwLockExt;
use std::collections::HashMap;
use std::sync::Arc;
use std::thread;
use tracing::error;

fn parse_generate_text_params(
    json: &HashMap<String, Value>,
) -> Result<(String, String, bool), Error> {
    let model_id = json
        .get("model_id")
        .and_then(Value::as_str)
        .ok_or_else(|| Error::InvalidRequest("Missing or invalid model_id".to_string()))?
        .to_string();

    let prompt = json
        .get("prompt")
        .and_then(Value::as_str)
        .ok_or_else(|| Error::InvalidRequest("Missing or invalid prompt".to_string()))?
        .to_string();

    // stream is optional,
    let stream = json.get("stream").and_then(Value::as_bool).unwrap_or(false);

    Ok((model_id, prompt, stream))
}

pub async fn generate_text(
    State(state): State<Arc<AppState>>,
    json: Json<HashMap<String, Value>>,
) -> ResultAPIStream {
    let (model_id, prompt, stream) = parse_generate_text_params(&json)?;
    println!(
        "Model ID: {}, Prompt: {}, Stream: {}",
        model_id, prompt, stream
    );
    let conversation = Conversation::Single(Message {
        role: "user".to_string(),
        content: prompt.clone(),
    });

    let (tx, rx): (Sender<String>, Receiver<String>) = bounded(100);
    let mut tx_err = tx.clone();

    let bridge = TokenBridge::new(rx);
    let stream = bridge
        .into_stream()
        .map(|msg| Ok::<_, Error>(format!("data: {}\n\n", msg)));
    let body = Body::from_stream(stream);

    thread::spawn(move || {
        let context = "generate prompt";
        match state.runner.write_lock(context) {
            Ok(guard) => {
                if let (Err(err)) = guard.generate_text(&model_id, &conversation, Some(tx)) {
                    error!("Failed to generate text: {}", err);
                    let _ = tx_err.send(format!("[ERROR]: Failed to generate text: {}", err));
                }
            }
            Err(err) => {
                error!("Failed to acquire write lock: {}", err);
                let _ = tx_err.send(format!("[ERROR]: Failed to generate text: {}", err));
            }
        }
    });

    let response = Response::builder()
        .header("Content-Type", "text/event-stream")
        .body(body)
        .unwrap();
    Ok(response)
}
