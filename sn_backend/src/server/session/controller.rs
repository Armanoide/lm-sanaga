use std::sync::Arc;
use axum::extract::State;
use axum::Json;
use serde_json::json;
use sn_core::server::payload::create_session_request::CreateSessionRequest;
use crate::db;
use crate::server::app_state::AppState;
use crate::error::{Error, ResultAPI};

pub async fn create_session(State(state): State<Arc<AppState>>, payload: Json<CreateSessionRequest>) -> ResultAPI {
    if let Some(db) = &state.db {
        let session = db::repository::session::create_session(db, payload).await?;
        Ok(Json(json!(session)))
    }
    else {
        Err(Error::NoDbAvailable)
    }
}
