use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AnnItem {
    pub partition_id: i32,
    pub primary_key: i32,
    pub vectors: Vec<f32>,
}
