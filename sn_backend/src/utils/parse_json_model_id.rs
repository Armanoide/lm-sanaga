use crate::error::ErrorBackend;
use serde_json::Value;
use std::collections::HashMap;

pub fn parse_json_model_id(json: &HashMap<String, Value>) -> Result<String, ErrorBackend> {
    Ok(json
        .get("id")
        .and_then(Value::as_str)
        .ok_or_else(|| ErrorBackend::InvalidRequest("Missing or invalid model_id".to_string()))?
        .to_string())
}
