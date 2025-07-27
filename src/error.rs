use serde_json;
use thiserror::Error;

pub type Result<T> = std::result::Result<T, crate::error::Error>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),

    #[error("Unsupported model weight parameter: {0}")]
    UnsupportedWeight(String),

    #[error("Failed to parse integer value")]
    ParseIntError(#[from] std::num::ParseIntError),

    #[error("Failed to parse JSON")]
    JsonError(#[from] serde_json::Error),

    #[error("Root model could not be found in the path: {0}")]
    RootModelPathNotFound(String),

    #[error("Model could not be found in the path: {0}")]
    ModelPathNotFound(String),

    #[error("Model weight not found in path: {0}")]
    ModelWeightPathNotFound(String),

    #[error("Failed to open file: {0}")]
    FileOpenError(String),

    #[error("Model decoder not found")]
    ModelDecoderNotFound,

    #[error("No tensor found in the model file")]
    NoTensorInModelFile,

    #[error("Unsupported model type: {0}")]
    UnsupportedModelType(String),

    #[error("Unsupported tokenizer format")]
    UnsupportedTokenizer,

    #[error("MLX runtime exception")]
    ExceptionMLX(#[from] mlx_rs::error::Exception),

    #[error("I/O error")]
    IOError(#[from] std::io::Error),

    #[error("Tensor size mismatch: read {0} bytes, expected {1} bytes")]
    TensorSizeMismatch(usize, usize),

    #[error("Template processing error: {0}")]
    TemplateError(String),

    #[error("Tracking error: {0}")]
    TrackingError(&'static str),

    #[error("Tokenizer error")]
    TokenizerError(#[from] tokenizers::tokenizer::Error),

    #[error("Tokenizer template builder error")]
    TokenizerTemplateError(
        #[from] tokenizers::processors::template::TemplateProcessingBuilderError,
    ),

    #[error("MiniJinja template error")]
    MiniJinjaError(#[from] minijinja::Error),

    #[error("Unable to process tokenizer encoding: {0}")]
    EncodingProcessingError(tokenizers::tokenizer::Error),

    #[error("Failed to read safetensors header")]
    SafetensorsHeaderReadError,

    #[error("Invalid glob pattern")]
    GlobPatternError(#[from] glob::PatternError),

    #[error("Missing rope configuration in config file")]
    RopeConfigMissing,

    #[error("Failed to initialize token generation process")]
    TokenGenerationStartFailure,

    #[error("Unable to retrieve peak memory usage")]
    MemoryPeakQueryFailure,

    #[error("Cache lock poisoned: {0}")]
    CacheLockPoisoned(String),

    #[error("Failed to load MLX library function: {0}")]
    MlxFunctionLoadFailure(String),

    #[error("UTF-8 decoding error")]
    Utf8Error(#[from] std::str::Utf8Error),

    #[error("Tensor '{tensor}' requested bytes {start}..{end}, but file size is {file_size}")]
    SafetensorsOutOfBounds {
        tensor: String,
        start: usize,
        end: usize,
        file_size: usize,
    },
}
