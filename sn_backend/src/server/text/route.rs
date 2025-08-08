use crate::server::app_state::AppState;
use crate::server::text::controller::generate_text;
use axum::routing::post;
use std::sync::Arc;

pub fn routes() -> axum::Router<Arc<AppState>> {
    axum::Router::new().route("/v1/texts/generate", post(generate_text))
}
