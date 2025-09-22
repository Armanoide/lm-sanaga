use axum::Json;
use serde_json::json;

use crate::error::ResultAPI;

pub async fn ping() -> ResultAPI {
    return Ok(Json(json!({"message": "pong"})));
}
