use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct CreateSessionRequest {
    pub name: Option<String>,
}

