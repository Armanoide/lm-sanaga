use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RunModelResponseSSE {
    pub load_type: String,
    pub tensor_name: String,
    pub tensor_index: usize,
    pub total_tensors: usize,
}

