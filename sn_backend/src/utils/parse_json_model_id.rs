use std::collections::HashMap;
use axum::Json;
use serde_json::Value;
use crate::error::Error;

pub fn parse_json_model_id(
    json: &HashMap<String, Value>,
) -> Result<String, Error> {
    Ok(json
        .get("id")
        .and_then(Value::as_str)
        .ok_or_else(|| Error::InvalidRequest("Missing or invalid model_id".to_string()))?
        .to_string())
}