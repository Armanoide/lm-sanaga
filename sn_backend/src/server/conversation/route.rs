use std::sync::Arc;
use crate::server::app_state::AppState;
use crate::server::conversation::controller::get_session_conversation_list;

pub fn routes() -> axum::Router<Arc<AppState>> {
  axum::Router::new()
    .route("/v1/sessions/{session_id}/conversations", axum::routing::get(get_session_conversation_list))
}