use crate::server::app_state::AppState;
use crate::server::session::controller::create_session;
use std::sync::Arc;

pub fn routes() -> axum::Router<Arc<AppState>> {
    axum::Router::new().route("/v1/sessions", axum::routing::post(create_session))
}
