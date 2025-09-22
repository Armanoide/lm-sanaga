use serde_json;
use thiserror::Error;
pub type Result<T> = std::result::Result<T, crate::error::ErrorCore>;

#[derive(Debug, Error)]
pub enum ErrorCore {
    #[error("Failed to parse integer value: {0}")]
    ParseIntError(#[from] std::num::ParseIntError),

    #[error("Failed to parse JSON {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("I/O error: {0}")]
    IOError(#[from] std::io::Error),

    #[error("UTF-8 decoding error: {0}")]
    Utf8Error(#[from] std::str::Utf8Error),

    #[error(" system time error :{0}")]
    SystemTimeError(#[from] std::time::SystemTimeError),

    #[error("Cache lock poisoned: {0}")]
    CacheLockPoisoned(String),

    #[error("Unknown role: {0}")]
    UnknownMessageRole(String),

    #[error("Uninitialize element: {0}")]
    UninitializeElement(#[from] derive_builder::UninitializedFieldError),

    #[error("Invalid action: {0}")]
    InvalidAction(String),
}
