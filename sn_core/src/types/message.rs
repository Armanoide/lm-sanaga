use crate::error::{Error, Result};
use crate::types::message_stats::MessageStats;
use serde::{Deserialize, Serialize};
use tracing::error;
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub content: String,
    pub role: MessageRole,
    pub stats: Option<MessageStats>,
}

impl Message {
    pub fn remove_think(&mut self) {
        if let Some(end_pos) = self.content.find("</think>") {
            let after_think = end_pos + "</think>".len();
            self.content = self.content[after_think..].trim_start().to_string();
        }
    }
}

impl From<Message> for Vec<Message> {
    fn from(message: Message) -> Self {
        vec![message]
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MessageRole {
    #[serde(rename = "user")]
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
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_remove_think_tag_present() {
        let mut msg = Message {
            content: "<think>\nReasoning here\n</think>\nFinal output.".to_string(),
            role: MessageRole::Assistant,
            stats: None,
        };
        msg.remove_think();
        assert_eq!(msg.content, "Final output.");
    }

    #[test]
    fn test_remove_think_tag_not_present() {
        let original = "Just normal content.";
        let mut msg = Message {
            content: original.to_string(),
            role: MessageRole::User,
            stats: None,
        };
        msg.remove_think();
        assert_eq!(msg.content, original);
    }

    #[test]
    fn test_remove_think_tag_with_leading_newline() {
        let mut msg = Message {
            content: "<think>\nThought\n</think>\n\n\nResponse.".to_string(),
            role: MessageRole::Assistant,
            stats: None,
        };
        msg.remove_think();
        assert_eq!(msg.content, "Response.");
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
        let msg = Message {
            content: "Hello".to_string(),
            role: MessageRole::User,
            stats: None,
        };
        let vec: Vec<Message> = msg.clone().into();
        assert_eq!(vec.len(), 1);
        assert_eq!(vec[0].content, "Hello");
        assert_eq!(vec[0].role.to_string(), "user");
    }
}
