use crate::server::app_state::AppState;
use crate::server::model::controller::{get_model_list, get_models_running, run_model, stop_model};
use axum::routing::{get, post};
use std::sync::Arc;

pub fn routes() -> axum::Router<Arc<AppState>> {
    axum::Router::new()
        .route("/v1/models", get(get_model_list))
        .route("/v1/models/ps", get(get_models_running))
        .route("/v1/models/run", post(run_model))
        .route("/v1/models/stop", post(stop_model))
}
