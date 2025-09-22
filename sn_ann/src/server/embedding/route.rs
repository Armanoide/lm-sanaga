use crate::server::{
    app_state::AppState,
    embedding::controller::{insert_bulk_embeddings, insert_embeddings},
};
use std::sync::Arc;

pub fn routes() -> axum::Router<Arc<AppState>> {
    axum::Router::new()
        .route("/insert", axum::routing::post(insert_embeddings))
        .route("/insert_bulk", axum::routing::post(insert_bulk_embeddings))
        .route("/search", axum::routing::post(insert_bulk_embeddings))
}
