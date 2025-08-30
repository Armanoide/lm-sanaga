use crate::error::{ErrorCore, Result};
use crate::types::message_stats::MessageStats;
use derive_builder::Builder;
use serde::{Deserialize, Serialize};
use tracing::error;

#[derive(Debug, Clone, Serialize, Deserialize, Default, Builder)]
#[builder(build_fn(error = "crate::error::ErrorCore"))]
pub struct Message {
    #[builder(default)]
    pub id: i32,
    pub content: String,
    pub role: MessageRole,
    #[builder(default)]
    pub stats: Option<MessageStats>,
    #[builder(default)]
    pub embeddings: Vec<f32>,
}

impl Message {
    pub fn sanitize_content(s: &str) -> String {
        Self::remove_think(s)
    }

    pub fn remove_think(content: &str) -> String {
        if let Some(end_pos) = content.find("</think>") {
            let after_think = end_pos + "</think>".len();
            content[after_think..].trim_start().to_string()
        } else {
            content.to_string()
        }
    }
}

impl From<Message> for Vec<Message> {
    fn from(message: Message) -> Self {
        vec![message]
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub enum MessageRole {
    #[serde(rename = "user")]
    #[default]
    User,
    #[serde(rename = "assistant")]
    Assistant,
    #[serde(rename = "system")]
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
    type Error = ErrorCore;
    fn try_from(role: &str) -> Result<MessageRole> {
        match role {
            "user" => Ok(MessageRole::User),
            "assistant" => Ok(MessageRole::Assistant),
            "system" => Ok(MessageRole::System),
            _ => {
                let format_err = format!("Unknown message role: {}", role);
                error!(format_err);
                Err(ErrorCore::UnknownMessageRole(format_err))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_remove_think_tag_present() {
        assert_eq!(
            Message::remove_think("<think>\nReasoning here\n</think>\nFinal output."),
            "Final output."
        );
    }

    #[test]
    fn test_remove_think_tag_not_present() {
        let original = "Just normal content.";
        assert_eq!(Message::remove_think(original), original);
    }

    #[test]
    fn test_remove_think_tag_with_leading_newline() {
        assert_eq!(
            Message::remove_think("\n<think>\nThought process\n</think>\n\nResponse."),
            "Response."
        );
    }

    #[test]
    fn test_message_role_to_string() {
        assert_eq!(MessageRole::User.to_string(), "user");
        assert_eq!(MessageRole::Assistant.to_string(), "assistant");
        assert_eq!(MessageRole::System.to_string(), "system");
    }

    #[test]
    fn test_message_role_try_from_valid() {
        assert_eq!(MessageRole::try_from("user").unwrap(), MessageRole::User);
        assert_eq!(
            MessageRole::try_from("assistant").unwrap(),
            MessageRole::Assistant
        );
        assert_eq!(
            MessageRole::try_from("system").unwrap(),
            MessageRole::System
        );
    }

    #[test]
    fn test_message_role_try_from_invalid() {
        let result = MessageRole::try_from("bot");
        assert!(result.is_err());
    }

    #[test]
    fn test_from_message_to_vec() {
        let msg = MessageBuilder::default()
            .content("Hello".to_string())
            .role(MessageRole::User)
            .build()
            .unwrap();
        let vec: Vec<Message> = msg.clone().into();
        assert_eq!(vec.len(), 1);
        assert_eq!(vec[0].content, "Hello");
        assert_eq!(vec[0].role.to_string(), "user");
    }
}
