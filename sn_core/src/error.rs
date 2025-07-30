use serde_json;
use thiserror::Error;
pub type Result<T> = std::result::Result<T, crate::error::Error>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Failed to parse integer value")]
    ParseIntError(#[from] std::num::ParseIntError),

    #[error("Failed to parse JSON {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("I/O error")]
    IOError(#[from] std::io::Error),

    #[error("UTF-8 decoding error")]
    Utf8Error(#[from] std::str::Utf8Error),

    #[error(" system time error :{0}")]
    SystemTimeError(#[from] std::time::SystemTimeError),

    #[error("Cache lock poisoned: {0}")]
    CacheLockPoisoned(String),
}
