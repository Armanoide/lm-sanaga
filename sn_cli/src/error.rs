use thiserror::Error;
pub type Result<T> = std::result::Result<T, Error>;

#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Core(#[from] sn_core::error::Error),

    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),

    #[error("Could not connect to server at {0} — is it running?")]
    ConnectionRefused(String),

    #[error("Model {0} not installed")]
    ModelNotInstalled(String),
}
