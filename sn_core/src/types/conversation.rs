use crate::db::message::Message;
use crate::db::message_stats::MessageStats;

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

    pub fn add_user_message(&mut self, content: String) {
        let message = Message {
            role: String::from("user"),
            content,
            stats: None,
        };
        self.add_message(message);
    }

    pub fn add_assistant_message(&mut self, content: String, stats: Option<MessageStats>) {
        let message = Message {
            role: String::from("assistant"),
            content,
            stats,
        };
        self.add_message(message);
    }

    pub fn to_vec(&self) -> Vec<Message> {
        self.messages.clone()
    }
}
