use serde::{Deserialize, Serialize};

use crate::types::ann_item::AnnItem;

#[derive(Debug, Deserialize, Serialize)]
pub struct SearchResponse {
    pub vectors: Vec<AnnItem>,
}
