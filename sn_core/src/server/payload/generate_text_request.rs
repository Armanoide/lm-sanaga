use std::sync::Arc;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct GenerateTextRequest {
    pub model_id: Arc<str>,
    pub prompt: String,
    #[serde(default)] // default to false if not present
    pub stream: Option<bool>,
    #[serde(default)]
    pub conversation_id: Option<i32>,
    #[serde(default)]
    pub session_id: Option<i32>,
}
