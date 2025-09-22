use std::sync::Arc;

use axum::routing::get;

use crate::server::{app_state::AppState, ping::controller::ping};

pub fn routes() -> axum::Router<Arc<AppState>> {
    axum::Router::new().route("/ping", get(ping))
}
