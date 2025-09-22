use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct PartitionStatusResponse {
    pub partition_id: i32,
    pub last_vector_id: i32,
}
