use axum::extract::rejection::{JsonRejection, QueryRejection};
use axum::Json;
use axum::response::{IntoResponse, Response};
use serde_json;
use serde_json::{Value, json};
use thiserror::Error;
use tracing::error;

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

    #[error("transparent")]
    ErrorAxum(axum::Error),

    #[error("No database connection available")]
    NoDbAvailable,

    #[error("Conversation not found")]
    ConversationNotFound,

    #[error("Failed to generate text: {0}")]
    FailedToGenerateText(Value),
}

impl IntoResponse for Error {
    fn into_response(self) -> axum::response::Response {
        let status = match &self {
            Error::Core(_) => axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            Error::ErrorAxum(_) => axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            Error::Inference(_) => axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            Error::DbError(_) => axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            Error::DotEnv(_) => axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            Error::EnvError(_) => axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            Error::ConversationNotFound => axum::http::StatusCode::BAD_REQUEST,
            Error::ModelNameRequired => axum::http::StatusCode::BAD_REQUEST,
            Error::InvalidRequest(_) => axum::http::StatusCode::BAD_REQUEST,
            Error::JsonRejection(_) => axum::http::StatusCode::BAD_REQUEST,
            Error::QueryRejection(_) => axum::http::StatusCode::BAD_REQUEST,
            Error::ModelIdRequired => axum::http::StatusCode::BAD_REQUEST,
            Error::IO(_) => axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            Error::NoDbAvailable => http::StatusCode::INTERNAL_SERVER_ERROR,
            Error::FailedToGenerateText(value) => http::StatusCode::INTERNAL_SERVER_ERROR,
        };

        let body = Json(json!({
            "error": match status {
                axum::http::StatusCode::BAD_REQUEST => self.to_string(),
                _ => "An unexpected error occurred".to_string(),
            }
        }));

        error!("Error occurred: {}", self);
        (status, body).into_response()
    }
}
