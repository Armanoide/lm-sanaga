use axum::{extract::rejection::JsonRejection, response::IntoResponse, Json};
use serde_json::{json, Value};
use thiserror::Error;
use tracing::error;

#[derive(Debug, Error)]
pub enum ErrorAnn {
    #[error(transparent)]
    Core(#[from] sn_core::error::ErrorCore),
    #[error(
        "ANNStore: invalid embedding for id {id}: expected dimension {expected_dim}, got {actual_dim}"
    )]
    AnnInvalidEmbedding {
        id: i32,
        expected_dim: usize,
        actual_dim: usize,
    },
    #[error("AnnStore: duplicate insert id {0}")]
    AnnDuplicateInsertId(i32),
    #[error("Dimension mismatch: expected {expected}, found {found}")]
    DimMismatch { expected: usize, found: usize },
    #[error("transparent")]
    JsonError(#[from] serde_json::Error),
    #[error(transparent)]
    EnvError(#[from] std::env::VarError),
    #[error("transparent")]
    IO(#[from] std::io::Error),
    #[error("url rejected with: {0}")]
    JsonRejection(#[from] JsonRejection),
    #[error("wrong search parms with k = {k} and nprobe nprobe = {nprobe}")]
    AnnWrongSearchParams { k: usize, nprobe: usize },
}

pub type Result<T> = std::result::Result<T, ErrorAnn>;
pub type ResultAPI = std::result::Result<Json<Value>, crate::error::ErrorAnn>;

impl IntoResponse for ErrorAnn {
    fn into_response(self) -> axum::response::Response {
        let status = match &self {
            ErrorAnn::Core(_) => axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            ErrorAnn::JsonError(_) => axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            ErrorAnn::EnvError(_) => axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            ErrorAnn::DimMismatch { .. } => axum::http::StatusCode::BAD_REQUEST,
            ErrorAnn::AnnInvalidEmbedding { .. } => axum::http::StatusCode::BAD_REQUEST,
            ErrorAnn::AnnDuplicateInsertId(_) => axum::http::StatusCode::BAD_REQUEST,
            ErrorAnn::IO(_) => axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            ErrorAnn::JsonRejection(_) => axum::http::StatusCode::BAD_REQUEST,
            ErrorAnn::AnnWrongSearchParams { .. } => axum::http::StatusCode::BAD_REQUEST,
        };

        let body = Json(json!({
            "error": match status {
                axum::http::StatusCode::BAD_REQUEST => self.to_string(),
                _ => "An unexpected error occurred".to_string(),
            }
        }));

        error!("ErrorAnn occurred: {}", self);
        (status, body).into_response()
    }
}
