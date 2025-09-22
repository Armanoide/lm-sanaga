use crate::error::ResultAPI;
use crate::server::app_state::AppState;
use axum::extract::{Path, State};
use axum::Json;
use serde_json::json;
use std::sync::Arc;

pub async fn list_conversations_handler(
    State(state): State<Arc<AppState>>,
    Path(session_id): axum::extract::Path<i32>,
) -> ResultAPI {
    let conversations = state
        .service_conversation
        .list_conversations(&session_id)
        .await?;
    Ok(Json(json!(conversations)))
}
