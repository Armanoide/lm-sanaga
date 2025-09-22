use std::sync::{Arc, RwLock};

use crate::ann::AnnIndex;

pub struct AppState {
    pub ann_idx: Arc<RwLock<AnnIndex>>,
}
impl AppState {
    pub fn new(ann_idx: AnnIndex) -> Self {
        AppState {
            ann_idx: Arc::new(RwLock::new(ann_idx)),
        }
    }
}
