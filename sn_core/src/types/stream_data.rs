use crate::server::payload::backend::run_model_metadata_response_sse::RunModelMetadataResponseSSE;
use crate::server::payload::backend::run_model_response::RunModelResponseSSE;
use crate::server::payload::backend::text_generated_metadata_response_sse::TextGeneratedMetadataResponseSSE;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tracing::error;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "stream_type", content = "data")]
pub enum StreamDataContent {
    String(String),
    RunModelResponseSSE(RunModelResponseSSE),
    TextGeneratedMetadataResponseSSE(TextGeneratedMetadataResponseSSE),
    RunModelMetadataResponseSSE(RunModelMetadataResponseSSE),
}

impl Default for StreamDataContent {
    fn default() -> Self {
        StreamDataContent::String(String::new())
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct StreamData {
    pub content: StreamDataContent,
    pub error: String,
}

impl StreamData {
    pub fn new(content: StreamDataContent, error: String) -> Self {
        StreamData { content, error }
    }

    pub fn for_stream_error(error: String) -> Self {
        StreamData {
            error,
            ..Default::default()
        }
    }

    pub fn for_string(content: String) -> Self {
        StreamData {
            content: StreamDataContent::String(content),
            ..Default::default()
        }
    }

    pub fn for_metadata_run_model_sse_response(content: RunModelMetadataResponseSSE) -> Self {
        StreamData {
            content: StreamDataContent::RunModelMetadataResponseSSE(content),
            ..Default::default()
        }
    }

    pub fn for_text_generated_metadata_sse_response(
        content: TextGeneratedMetadataResponseSSE,
    ) -> Self {
        StreamData {
            content: StreamDataContent::TextGeneratedMetadataResponseSSE(content),
            ..Default::default()
        }
    }

    pub fn for_run_model_sse_response(content: RunModelResponseSSE) -> Self {
        StreamData {
            content: StreamDataContent::RunModelResponseSSE(content),
            ..Default::default()
        }
    }

    pub fn to_json(&self) -> Value {
        serde_json::to_value(self).unwrap_or(Value::Null)
    }
}

impl From<StreamData> for String {
    fn from(data: StreamData) -> Self {
        serde_json::to_string(&data).unwrap_or_else(|e| {
            error!("Failed to serialize StreamData to JSON: {}", e);
            String::default()
        })
    }
}
