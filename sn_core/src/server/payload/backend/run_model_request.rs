use crate::error::{ErrorCore, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RunModelRequest {
    Start {
        model_name: String,
        stream: Option<bool>,
    },
    Stop {
        id: String,
    },
}

impl RunModelRequest {
    pub fn get_stream(&self) -> Option<bool> {
        match self {
            RunModelRequest::Start { stream, .. } => *stream,
            RunModelRequest::Stop { .. } => None,
        }
    }

    pub fn get_id(&self) -> Result<String> {
        match self {
            RunModelRequest::Start { model_name, .. } => Err(ErrorCore::InvalidAction(format!(
                "Cannot get id from Start request (model_name: {})",
                model_name
            ))),
            RunModelRequest::Stop { id } => Ok(id.clone()),
        }
    }
    pub fn get_model_name(&self) -> Result<String> {
        match self {
            RunModelRequest::Start { model_name, .. } => Ok(model_name.clone()),
            RunModelRequest::Stop { id } => Err(ErrorCore::InvalidAction(format!(
                "Cannot get model_name from Stop request (id: {})",
                id
            ))),
        }
    }
}
