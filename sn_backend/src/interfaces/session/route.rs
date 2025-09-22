use sn_core::server::routes::BackendApiSession;

use crate::{interfaces::session::controller::create_session_handler, server::app_state::AppState};
use std::sync::Arc;

pub fn routes() -> axum::Router<Arc<AppState>> {
    axum::Router::new().route(
        BackendApiSession::Create.path().as_str(),
        axum::routing::post(create_session_handler),
    )
}
