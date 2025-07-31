use axum::Json;
use axum::response::{IntoResponse, Response};
use serde_json;
use serde_json::{Value, json};
use thiserror::Error;

pub type ResultAPIStream = std::result::Result<Response, crate::error::Error>;
pub type ResultAPI = std::result::Result<Json<Value>, crate::error::Error>;
pub type Result<T> = std::result::Result<T, crate::error::Error>;

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    Core(#[from] sn_core::error::Error),

    #[error(transparent)]
    Inference(#[from] sn_inference::error::Error),

    #[error("Model name is required")]
    ModelNameRequired,

    #[error("Model ID is required")]
    ModelIdRequired,

    #[error("{0}")]
    InvalidRequest(String),
}

impl IntoResponse for Error {
    fn into_response(self) -> axum::response::Response {
        let status = match &self {
            Error::Core(_) => axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            Error::Inference(_) => axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            Error::ModelNameRequired => axum::http::StatusCode::BAD_REQUEST,
            Error::InvalidRequest(_) => axum::http::StatusCode::BAD_REQUEST,
            Error::ModelIdRequired => axum::http::StatusCode::BAD_REQUEST,
        };

        println!("Error occurred: {:?}", self);
        let body = Json(json!({
            "error": self.to_string(),
        }));

        (status, body).into_response()
    }
}
