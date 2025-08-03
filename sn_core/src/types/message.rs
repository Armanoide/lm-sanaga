use serde::Serialize;
use crate::types::message_stats::MessageStats;

#[derive(Debug, Clone, Serialize)]
pub struct Message {
    pub content: String,
    pub role: String,
    pub stats: Option<MessageStats>,
}

impl From<Message> for Vec<Message> {
    fn from(message: Message) -> Self {
        vec![message]
    }
}

pub const ROLE_USER: &str = "user";
pub const ROLE_ASSISTANT: &str = "assistant";
pub const ROLE_SYSTEM: &str = "system";