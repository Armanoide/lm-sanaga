use serde_json;
use std::convert::Infallible;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    Core(#[from] sn_core::error::ErrorCore),

    #[error(transparent)]
    ErrorInfallible(#[from] Infallible),

    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),

    #[error("Unsupported model weight parameter: {0}")]
    UnsupportedWeight(String),

    #[error("Unsupported parse model weight for: {0}")]
    UnsupportedParseWeight(String),

    #[error("Failed to parse integer value: {0}")]
    ParseIntError(#[from] std::num::ParseIntError),

    #[error("Failed to parse JSON: {0}")]
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

    #[error("MLX runtime exception: {0}")]
    ExceptionMLX(#[from] mlx_rs::error::Exception),

    #[error("I/O error: {0}")]
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

    #[error("MiniJinja template error: {0}")]
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

    #[error("Failed to load MLX library function: {0}")]
    MlxFunctionLoadFailure(String),

    #[error("UTF-8 decoding error: {0}")]
    Utf8Error(#[from] std::str::Utf8Error),

    #[error("Tensor '{tensor}' requested bytes {start}..{end}, but file size is {file_size}")]
    SafetensorsOutOfBounds {
        tensor: String,
        start: usize,
        end: usize,
        file_size: usize,
    },

    #[error("System time error :{0}")]
    SystemTimeError(#[from] std::time::SystemTimeError),

    #[error("Model runtime not found with id: {0}")]
    ModelRuntimeNotFoundWithId(String),

    #[error("MLX compute lock error")]
    MLXComputeLock(String),

    #[error("Empty prompt generated")]
    EmptyPrompt,

    #[error("Model not found when generating text")]
    MissingModel,

    #[error("Chat template not found when generating text")]
    MissingChatTemplate,

    #[error("Tokenizer not found when generating text")]
    MissingTokenizer,

    #[error("Routine missing weight for model: {0}")]
    RoutineMissingWeight(String),

    #[error("Routine missing for model: {0}")]
    RoutineMissingModel(String),

    #[error("Failed to find max padding length for encoding")]
    PaddingFailedFindMax,

    #[error("Failed to find padding token for encoding")]
    MissingPadToken,

    #[error("Unexpected mask shape: {0}")]
    UnexpectedMaskShape(String),

    #[error("{0}")]
    ErrorScaledDotProductAttentionGQA(String),

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
}

pub type Result<T> = std::result::Result<T, crate::error::Error>;
/*

/// A helper trait to add file/line info automatically to any Result
pub trait ResultExt<T> {
    fn here(self) -> anyhow::Result<T>;
}

impl<T, E> ResultExt<T> for std::result::Result<T, E>
where
    E: std::error::Error + Send + Sync + 'static,
{
    fn here(self) {
        self.with_context(|| format!("at {}:{}", file!(), line!()))
    }
}

pub fn test( ) -> Result<()> {
    // This is just a placeholder function to ensure the module compiles correctly.
    // You can remove or modify this function as needed.

    Err(Error::MLXComputeLock("d")).here();
    Ok(())
}
*/
