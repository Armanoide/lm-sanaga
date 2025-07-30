use crate::app_state::AppState;
use crate::handlers::model::{get_model_list, get_models_running};
use axum::routing::get;
use std::sync::Arc;

pub fn routes() -> axum::Router<Arc<AppState>> {
    axum::Router::new()
        .route("/v1/models", get(get_model_list))
        .route("/v1/models/ps", get(get_models_running))
}
