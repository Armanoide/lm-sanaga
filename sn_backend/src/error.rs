use axum::Json;
use axum::response::IntoResponse;
use serde_json;
use serde_json::{Value, json};
use thiserror::Error;

pub type ResultAPI = std::result::Result<Json<Value>, crate::error::Error>;
pub type Result<T> = std::result::Result<T, crate::error::Error>;

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    Core(#[from] sn_core::error::Error),

    #[error(transparent)]
    Inference(#[from] sn_inference::error::Error),
}

impl IntoResponse for Error {
    fn into_response(self) -> axum::response::Response {
        let status = match self {
            Error::Core(_) => axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            Error::Inference(_) => axum::http::StatusCode::INTERNAL_SERVER_ERROR,
        };

        let body = Json(json!({
            "error": self.to_string(),
        }));

        (status, body).into_response()
    }
}
