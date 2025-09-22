use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct SearchRequest {
    pub partition_id: i32,
    pub nprobe: usize,
    pub k: usize,
    pub vectors: Vec<f32>,
}
