use crate::app_state::AppState;
use crate::text::controller::generate_text;
use axum::routing::{get, post};
use std::sync::Arc;

pub fn routes() -> axum::Router<Arc<AppState>> {
    axum::Router::new().route("/v1/text/generate", post(generate_text))
}
