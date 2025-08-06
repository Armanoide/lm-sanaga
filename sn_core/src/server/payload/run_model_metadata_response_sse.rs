use std::sync::Arc;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunModelMetadataResponseSSE {
    pub model_id: Arc<str>,
}