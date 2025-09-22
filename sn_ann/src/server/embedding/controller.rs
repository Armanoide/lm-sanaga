use std::sync::Arc;

use axum::{
    extract::{rejection::JsonRejection, State},
    Json,
};
use serde_json::json;
use sn_core::{
    server::payload::ann::{search_request::SearchRequest, search_response::SearchResponse},
    types::ann_item::AnnItem,
    utils::rw_lock::RwLockExt,
};

use crate::{
    error::{ErrorAnn, ResultAPI},
    server::app_state::AppState,
};

pub async fn insert_embeddings(
    State(state): State<Arc<AppState>>,
    payload: std::result::Result<Json<AnnItem>, JsonRejection>,
) -> ResultAPI {
    let payload = payload?;
    let item = payload.0.clone();
    let ann_stare = &state.ann_idx;
    ann_stare
        .write_lock_mut("insert_embedding_request")?
        .insert(item)?;
    Ok(Json(serde_json::json!({"status": "ok"})))
}

pub async fn insert_bulk_embeddings(
    State(state): State<Arc<AppState>>,
    payload: std::result::Result<Json<Vec<AnnItem>>, JsonRejection>,
) -> ResultAPI {
    let payload = payload?;
    let items = payload.0.clone();
    let ann_stare = &state.ann_idx;
    ann_stare
        .write_lock_mut("insert_bulk_embedding_request")?
        .bulk_insert(items)?;
    Ok(Json(serde_json::json!({"status": "ok"})))
}

pub async fn similarity_search(
    State(state): State<Arc<AppState>>,
    payload: std::result::Result<Json<SearchRequest>, JsonRejection>,
) -> ResultAPI {
    let payload = payload?;
    let k = payload.k;
    let nprobe = payload.nprobe;
    let partition_id = payload.partition_id;
    let vectors = &payload.vectors;

    if k == 0 || nprobe == 0 || vectors.is_empty() {
        return Err(ErrorAnn::AnnWrongSearchParams { k: 0, nprobe: 0 });
    }

    let ann_idx = &state.ann_idx;
    let guard = ann_idx.read_lock("similarity_search_request")?;
    let results = guard.knn(vectors, k, nprobe);

    let filtered_results = results
        .iter()
        .filter_map(|(ann_item, _)| {
            if (ann_item.partition_id == partition_id) {
                Some((*ann_item).clone())
            } else {
                None
            }
        })
        .collect();

    let results = SearchResponse {
        vectors: filtered_results,
    };
    let results = json!({ "data": results });
    Ok(Json(results))
}
