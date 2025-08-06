use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RunModelRequest {
    pub model_name: String,
    pub stream: Option<bool>,
}