use serde::Serialize;
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize)]
pub struct Message {
    pub content: String,
    pub role: String,
}

impl Message {
    pub fn to_map(&self) -> HashMap<String, String> {
        let mut map = HashMap::new();
        map.insert(String::from("messages"), self.content.clone());
        map.insert(String::from("role"), self.role.clone());
        map
    }
}
pub enum Conversation {
    Single(Message),
    Batch(Vec<Message>),
}

impl Conversation {
    pub fn len(&self) -> usize {
        match &self {
            Conversation::Single(message) => 1,
            Conversation::Batch(messages) => messages.len(),
        }
    }

    pub fn as_array(&self) -> Vec<Message> {
        match &self {
            Conversation::Single(message) => vec![message.clone()],
            Conversation::Batch(messages) => messages.clone(),
        }
    }

    pub fn last_message(&self) -> Option<Message> {
        let arr = &self.as_array();
        arr.get(arr.len()).cloned()
    }
}
