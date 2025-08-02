use serde::Serialize;
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize)]
pub struct Message {
    pub content: String,
    pub role: String,
    pub stat
}

impl From<Message> for Vec<Message> {
    fn from(message: Message) -> Self {
        vec![message]
    }
}

pub struct Conversation {
    pub messages: Vec<Message>,
}

impl Conversation {
    
    pub fn from_message(message: Message) -> Self {
        Conversation {
            messages: vec![message],
        }
    }
    pub fn from_vec(messages: Vec<Message>) -> Self {
        Conversation { messages }
    }
    pub fn new() -> Self {
        Conversation {
            messages: Vec::new(),
        }
    }

    pub fn add_message(&mut self, message: Message) {
        self.messages.push(message);
    }

    pub fn to_vec(&self) -> Vec<Message> {
        self.messages.clone()
    }
}
