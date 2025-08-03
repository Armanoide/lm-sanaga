use std::sync::Arc;
use crate::server::app_state::AppState;
use crate::server::session::controller::create_session;

pub fn routes() -> axum::Router<Arc<AppState>> {
    axum::Router::new()
        .route("/v1/sessions", axum::routing::post(create_session))
}