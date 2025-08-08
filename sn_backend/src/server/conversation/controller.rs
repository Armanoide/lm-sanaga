use crate::db;
use crate::db::entities::conversation::Convert;
use crate::error::{Error, ResultAPI};
use crate::server::app_state::AppState;
use axum::Json;
use axum::extract::{Path, State};
use serde_json::json;
use std::sync::Arc;

/// Handler to retrieve all conversations for a given session ID.
///
/// # Arguments
/// * `State(state)` - Shared application state (includes the database connection).
/// * `Path(session_id)` - Path-extracted session ID from the request URL.
///
/// # Returns
/// * `Ok(Json)` - A JSON response containing a list of conversations for the session.
/// * `Err` - If the database is unavailable or a query fails.
pub async fn get_session_conversation_list(
    State(state): State<Arc<AppState>>,
    Path(session_id): axum::extract::Path<i32>,
) -> ResultAPI {
    let db = state.db.as_ref().ok_or(Error::NoDbAvailable)?;

    let conversations =
        db::repository::conversation::get_conversations_by_session_id(&db, session_id).await?;
    println!(
        "Retrieved {} conversations for session_id {}",
        conversations.len(),
        session_id
    );
    let conversations = conversations.into_conversations();
    Ok(Json(json!(conversations)))
}
