use crate::error::{Error, ResultAPIStream};
use crate::utils::parse_json_model_id::parse_json_model_id;
use crate::utils::tokio_bridge::TokenBridge;
use axum::Json;
use axum::body::Body;
use axum::extract::State;
use axum::response::Response;
use crossbeam::channel::{Receiver, Sender, bounded};
use futures::StreamExt;
use serde_json::{json, Value};
use sn_core::utils::rw_lock::RwLockExt;
use std::collections::HashMap;
use std::sync::Arc;
use std::thread;
use axum::extract::rejection::JsonRejection;
use sea_orm::DatabaseConnection;
use tracing::error;
use sn_core::types::conversation::Conversation;
use sn_core::types::message::{Message, ROLE_USER};
use sn_core::server::payload::generate_text_request::GenerateTextRequest;
use sn_core::types::stream_data::StreamData;
use sn_inference::model::model_runtime::GenerateTextResult;
use crate::db::{repository};
use crate::db::entities::message::{Convert, Model};
use crate::db::repository::message::{get_conversation_id_from_last_message_id, create_message, get_message_by_id};
use crate::server::app_state::AppState;
use crate::error::Result;


pub fn handle_error_generate_text(err: &String, tx_err: &Sender<StreamData>) {
    error!("{}", err);
    let error = format!("Failed to generate text: {}", err);
    let _ = tx_err.send(StreamData::stream_error(error).into());
}

pub async fn create_or_get_conversation (db: Option<&DatabaseConnection>, payload: &Json<GenerateTextRequest>) -> Result<Conversation> {
    let db = match db {
        Some(db) => db,
        None => return Ok(Conversation::default()),
    };

    let messages = repository::message::get_messages_from_payload(db, payload).await?;
    match messages {
        Some(messages) => Ok(messages.into_conversation()),
        None => Ok(Conversation::default())
    }
}

pub async fn generate_text(
    State(state): State<Arc<AppState>>,
    payload: std::result::Result<Json<GenerateTextRequest>, JsonRejection>,
) -> ResultAPIStream {
    let payload = payload?;
    let mut conversation = create_or_get_conversation(state.db.as_ref(), &payload).await?;
    conversation.messages.push(Message {
        role: ROLE_USER.to_string(),
        content: payload.prompt.clone(),
        stats: None,
    });


    //println!("===========> payload:{:?} <===========", payload);
    //println!("===========> conversation:{:?} <===========", conversation);

    let (tx, rx): (Sender<StreamData>, Receiver<StreamData>) = bounded(100);
    let tx_2 = tx.clone();

    let bridge = TokenBridge::new(rx);
    let stream = bridge
        .into_stream()
        .map(|stream_data: StreamData | Ok::<_, Error>(format!("data: {}\n\n", stream_data.to_json())));
    let body = Body::from_stream(stream);

    tokio::spawn(async move {

        let generate_text_result: std::result::Result<_, Error> = {
            state.runner.read_lock("reading runner for generate_text")
                .map_err(|e| Error::Core(e))
                .and_then(| guard| {
                    guard.generate_text(&payload.model_id, &conversation, Some(tx))
                        .map_err(|e| Error::Inference(e))
                })
        };

        let generate_text_result = if let Err(err) = generate_text_result {
            handle_error_generate_text(&err.to_string(), &tx_2);
            return;
        } else {
            generate_text_result.unwrap()
        };


        let db = if let Some(db) = &state.db  { db } else { return; };

        match create_message(db, &payload, &generate_text_result).await {
            Ok(message) => {
                //println!("===========> message:{:?} <===========", message);
                if let Some(message) = message {
                    //println!("[===========> message2:{:?} <===========]", message);
                    let _ = tx_2.send(StreamData::stream_metadata(json!({"message_id": message.id})));
                }
            }
            Err(err) => {
                handle_error_generate_text(&err.to_string(), &tx_2);
            }

        }
    });

    let response = Response::builder()
        .header("Content-Type", "text/event-stream")
        .body(body)
        .unwrap();
    Ok(response)
}
