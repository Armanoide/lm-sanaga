use axum::extract::rejection::{JsonRejection, QueryRejection};
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde_json;
use serde_json::{json, Value};
use thiserror::Error;
use tracing::error;

pub type ResultAPIStream = std::result::Result<Response, crate::error::ErrorBackend>;
pub type ResultAPI = std::result::Result<Json<Value>, crate::error::ErrorBackend>;
pub type Result<T> = std::result::Result<T, crate::error::ErrorBackend>;

#[derive(Debug, Error)]
pub enum ErrorBackend {
    #[error(transparent)]
    Core(#[from] sn_core::error::ErrorCore),

    #[error(transparent)]
    Inference(#[from] sn_inference::error::Error),

    #[error("Model name is required")]
    ModelNameRequired,

    #[error("{0}")]
    InvalidRequest(String),

    #[error(transparent)]
    DbError(#[from] sea_orm::error::DbErr),

    #[error(transparent)]
    EnvError(#[from] std::env::VarError),

    #[error(transparent)]
    DotEnv(#[from] dotenv::Error),

    #[error("transparent")]
    IO(#[from] std::io::Error),

    #[error("url rejected with: {0}")]
    JsonRejection(#[from] JsonRejection),

    #[error("url rejected with: {0}")]
    QueryRejection(#[from] QueryRejection),

    #[error("No database connection available")]
    NoDbAvailable,

    #[error("Conversation not found")]
    ConversationNotFound,

    #[error("Message with id {0} not found")]
    MessageNotFound(i32),

    #[error("Failed to generate text: {0}")]
    FailedToGenerateText(Value),

    #[error("Failed to build SSE response: {0}")]
    FailedBuildSSEResponse(String),

    #[error("Failed to parse JSON: {0}")]
    JsonError(#[from] serde_json::Error),
}

impl IntoResponse for ErrorBackend {
    fn into_response(self) -> axum::response::Response {
        let status = match &self {
            ErrorBackend::Core(_) => axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            ErrorBackend::Inference(_) => axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            ErrorBackend::DbError(_) => axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            ErrorBackend::DotEnv(_) => axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            ErrorBackend::JsonError(_) => axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            ErrorBackend::EnvError(_) => axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            ErrorBackend::FailedBuildSSEResponse(_) => {
                axum::http::StatusCode::INTERNAL_SERVER_ERROR
            }
            ErrorBackend::MessageNotFound(_) => axum::http::StatusCode::BAD_REQUEST,
            ErrorBackend::ConversationNotFound => axum::http::StatusCode::BAD_REQUEST,
            ErrorBackend::ModelNameRequired => axum::http::StatusCode::BAD_REQUEST,
            ErrorBackend::InvalidRequest(_) => axum::http::StatusCode::BAD_REQUEST,
            ErrorBackend::JsonRejection(_) => axum::http::StatusCode::BAD_REQUEST,
            ErrorBackend::QueryRejection(_) => axum::http::StatusCode::BAD_REQUEST,
            ErrorBackend::IO(_) => axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            ErrorBackend::NoDbAvailable => http::StatusCode::INTERNAL_SERVER_ERROR,
            ErrorBackend::FailedToGenerateText(_) => http::StatusCode::INTERNAL_SERVER_ERROR,
        };

        let body = Json(json!({
            "error": match status {
                axum::http::StatusCode::BAD_REQUEST => self.to_string(),
                _ => "An unexpected error occurred".to_string(),
            }
        }));

        error!("ErrorBackend occurred: {}", self);
        (status, body).into_response()
    }
}
