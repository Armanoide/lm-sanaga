use std::sync::Arc;

use axum::{
    extract::{Path, State},
    Json,
};
use serde_json::json;
use sn_core::{
    server::payload::ann::partition_status_response::PartitionStatusResponse,
    utils::rw_lock::RwLockExt,
};

use crate::{error::ResultAPI, server::app_state::AppState};

pub async fn get_partition_status(
    State(state): State<Arc<AppState>>,
    Path(partition_id): axum::extract::Path<i32>,
) -> ResultAPI {
    let last_vector_id = state
        .ann_idx
        .read_lock("get_partition_status")?
        .status(partition_id);
    Ok(Json(json!(PartitionStatusResponse {
        partition_id,
        last_vector_id
    })))
}
