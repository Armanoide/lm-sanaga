use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ListRunningModelResponse {
    pub name: String,
    pub id: String,
}
