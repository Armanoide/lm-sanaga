use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunModelMetadataResponseSSE {
    pub model_id: Arc<str>,
}
