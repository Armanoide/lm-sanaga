use crate::db;
use crate::error::{ErrorBackend, ResultAPI};
use crate::server::app_state::AppState;
use axum::Json;
use axum::extract::State;
use axum::extract::rejection::JsonRejection;
use serde_json::json;
use sn_core::server::payload::create_session_request::CreateSessionRequest;
use std::sync::Arc;

pub async fn create_session(
    State(state): State<Arc<AppState>>,
    payload: std::result::Result<Json<CreateSessionRequest>, JsonRejection>,
) -> ResultAPI {
    let payload = payload?;
    if let Some(db) = &state.db {
        let session = db::repository::session::create_session(db, payload).await?;
        Ok(Json(json!(session)))
    } else {
        Err(ErrorBackend::NoDbAvailable)
    }
}
