use sn_inference::runner::Runner;
use std::sync::{Arc, RwLock};

#[derive(Clone, Debug)]
pub struct AppState {
    pub runner: Arc<RwLock<Runner>>,
}
impl AppState {
    pub fn new(runner: Arc<RwLock<Runner>>) -> Self {
        AppState { runner }
    }
}
