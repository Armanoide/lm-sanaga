use crate::error::ResultAPI;
use crate::server::app_state::AppState;
use axum::extract::rejection::JsonRejection;
use axum::extract::State;
use axum::Json;
use serde_json::json;
use sn_core::server::payload::backend::create_session_request::CreateSessionRequest;
use std::sync::Arc;

pub async fn create_session_handler(
    State(state): State<Arc<AppState>>,
    payload: std::result::Result<Json<CreateSessionRequest>, JsonRejection>,
) -> ResultAPI {
    let payload = payload?.0;
    let session = state.service_session.handle_create(payload).await?;
    Ok(Json(json!(session)))
}
