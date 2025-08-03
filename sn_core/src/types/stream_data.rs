use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tracing::error;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct StreamData {
    pub content: String,
    pub error: String,
    pub metadata: Value,
}

impl StreamData {
    pub fn new(content: String, error: String, metadata: Value) -> Self {
        StreamData {
            content,
            error,
            metadata,
        }
    }

    pub fn stream_error(error: String) -> Self {
        let mut stream_data = StreamData::default();
        stream_data.error = error;
        stream_data
    }

    pub fn stream_content(content: String) -> Self {
        let mut stream_data = StreamData::default();
        stream_data.content = content;
        stream_data
    }

    pub fn stream_metadata(metadata: Value) -> Self {
        let mut stream_data = StreamData::default();
        stream_data.metadata = metadata;
        stream_data
    }

    pub fn to_json(&self) -> Value {
        json!({
            "content": self.content,
            "error": self.error,
            "metadata": self.metadata
        })
    }
}

impl From<StreamData> for String {
    fn from(data: StreamData) -> Self {
        serde_json::to_string(&data.to_json()).unwrap_or_else(|e| {
            error!("Failed to serialize StreamData to JSON: {}", e);
            String::default()
        })
    }
}