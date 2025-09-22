use sn_core::server::routes::BackendConversationApi;

use crate::{
    interfaces::conversation::controller::list_conversations_handler, server::app_state::AppState,
};
use std::sync::Arc;

pub fn routes() -> axum::Router<Arc<AppState>> {
    axum::Router::new().route(
        BackendConversationApi::List.path(None).as_str(),
        axum::routing::get(list_conversations_handler),
    )
}
