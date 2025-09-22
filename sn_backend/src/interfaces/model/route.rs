use crate::{
    interfaces::model::controller::{
        list_models_handler, list_running_models_handler, run_model_handler, stop_model_handler,
    },
    server::app_state::AppState,
};
use axum::routing::{get, post};
use sn_core::server::routes::BackendApiModel;
use std::sync::Arc;

pub fn routes() -> axum::Router<Arc<AppState>> {
    axum::Router::new()
        .route(
            BackendApiModel::List.path().as_str(),
            get(list_models_handler),
        )
        .route(
            BackendApiModel::ListRunning.path().as_str(),
            get(list_running_models_handler),
        )
        .route(
            BackendApiModel::Run.path().as_str(),
            post(run_model_handler),
        )
        .route(
            BackendApiModel::Stop.path().as_str(),
            post(stop_model_handler),
        )
}
