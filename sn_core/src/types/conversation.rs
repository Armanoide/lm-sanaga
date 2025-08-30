use crate::types::message::{Message, MessageBuilder, MessageRole};
use crate::types::message_stats::MessageStats;
use serde::{Deserialize, Serialize};
use std::fmt::Display;

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct Conversation {
    pub name: Option<String>,
    pub id: Option<i32>,
    pub messages: Vec<Message>,
}

impl Conversation {
    pub fn from_message(message: Message) -> Self {
        Conversation {
            name: None,
            id: None,
            messages: vec![message],
        }
    }

    pub fn from_user_with_content(content: String) -> Self {
        let message = MessageBuilder::default()
            .role(MessageRole::User)
            .content(content)
            .build()
            .unwrap_or_default();
        Conversation {
            name: None,
            id: None,
            messages: vec![message],
        }
    }
    pub fn from_vec(messages: Vec<Message>) -> Self {
        Conversation {
            name: None,
            id: None,
            messages,
        }
    }
    pub fn new() -> Self {
        Conversation {
            name: None,
            id: None,
            messages: Vec::new(),
        }
    }

    pub fn add_message(&mut self, message: Message) {
        self.messages.push(message);
    }

    pub fn add_user_message(&mut self, content: String) {
        let message = MessageBuilder::default()
            .role(MessageRole::User)
            .content(content)
            .build()
            .unwrap_or_default();
        self.add_message(message);
    }

    pub fn add_assistant_message(&mut self, content: String, stats: Option<MessageStats>) {
        let message = MessageBuilder::default()
            .role(MessageRole::Assistant)
            .content(content)
            .stats(stats)
            .build()
            .unwrap_or_default();
        self.add_message(message);
    }

    pub fn to_vec(&self) -> Vec<Message> {
        self.messages.clone()
    }
}

impl Display for Conversation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let default_name = String::from("Untitled Conversation");
        let name = self.name.as_ref().unwrap_or(&default_name);
        write!(f, "{}", name)
    }
}
