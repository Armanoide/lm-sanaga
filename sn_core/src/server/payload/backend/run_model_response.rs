use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RunModelResponse {
    SSE(RunModelResponseSSE),
    Json(RunModelResponseJson),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RunModelResponseSSE {
    pub load_type: String,
    pub tensor_name: String,
    pub tensor_index: usize,
    pub total_tensors: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RunModelAction {
    Start,
    Stop,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RunModelResponseJson {
    pub model_id: String,
    pub status: RunModelAction,
}
