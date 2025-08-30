use sea_orm::DatabaseConnection;
use sn_inference::{ann::TinyAnnStore, runner::Runner};
use std::sync::{Arc, RwLock};

#[derive(Clone, Debug)]
pub struct AppState {
    pub runner: Arc<RwLock<Runner>>,
    pub db: Option<DatabaseConnection>,
    pub ann_store: Arc<RwLock<TinyAnnStore>>,
}
impl AppState {
    pub fn new(
        runner: Arc<RwLock<Runner>>,
        db: Option<DatabaseConnection>,
        ann_store: Arc<RwLock<TinyAnnStore>>,
    ) -> Self {
        AppState {
            runner,
            db,
            ann_store,
        }
    }
}
