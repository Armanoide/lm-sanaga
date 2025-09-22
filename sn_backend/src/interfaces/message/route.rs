use crate::{interfaces::message::controller::generate_text_handler, server::app_state::AppState};
use axum::routing::post;
use sn_core::server::routes::BackendApiMessage;
use std::sync::Arc;

pub fn routes() -> axum::Router<Arc<AppState>> {
    axum::Router::new().route(
        BackendApiMessage::Generate.path().as_str(),
        post(generate_text_handler),
    )
}
