use std::sync::Arc;

use crate::server::{app_state::AppState, partition::controller::get_partition_status};

pub fn routes() -> axum::Router<Arc<AppState>> {
    axum::Router::new().route(
        "/partition/{partition_id}/status",
        axum::routing::get(get_partition_status),
    )
}
