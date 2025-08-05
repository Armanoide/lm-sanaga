use serde::{Deserialize, Serialize};
use tracing::error;
use crate::types::message_stats::MessageStats;
use crate::error::{Result, Error};
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub content: String,
    pub role: MessageRole,
    pub stats: Option<MessageStats>,
}

impl From<Message> for Vec<Message> {
    fn from(message: Message) -> Self {
        vec![message]
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageRole {
    User,
    Assistant,
    System,
}

impl MessageRole {
    pub fn to_string(&self) -> String {
        match self {
            MessageRole::User => String::from("user"),
            MessageRole::Assistant => String::from("assistant"),
            MessageRole::System => String::from("system"),
        }
    }
}

impl TryFrom<&str> for MessageRole {
    type Error = Error;
    fn try_from(role: &str) -> Result<MessageRole> {
        match role {
            "user" => Ok(MessageRole::User),
            "assistant" => Ok(MessageRole::Assistant),
            "system" => Ok(MessageRole::System),
            _ => {
                let format_err = format!("Unknown message role: {}", role);
                error!(format_err);
                Err(Error::UnknownMessageRole(format_err))
            },
        }
    }
}