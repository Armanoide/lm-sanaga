use sea_orm::DatabaseConnection;
use sn_inference::runner::Runner;
use std::sync::{Arc, RwLock};

#[derive(Clone, Debug)]
pub struct AppState {
    pub runner: Arc<RwLock<Runner>>,
    pub db: Option<DatabaseConnection>,
}
impl AppState {
    pub fn new(runner: Arc<RwLock<Runner>>, db: Option<DatabaseConnection>) -> Self {
        AppState { runner, db }
    }
}
