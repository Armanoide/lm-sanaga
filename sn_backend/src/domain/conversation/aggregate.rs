use sn_core::types::{conversation::Conversation, message::Message};

#[derive(Debug, Clone)]
pub struct ConversationAggregate {
    root: Conversation,
    messages: Vec<Message>,
}

impl ConversationAggregate {
    pub fn new(root: Conversation) -> Self {
        Self {
            root,
            messages: vec![],
        }
    }

    pub fn rename(&mut self, new_name: String) {
        self.root.name = Some(new_name);
    }

    pub fn add_user_message(&mut self, into_message: Message) {
        self.messages.push(into_message);
    }

    pub fn first_user_message(&self) -> String {
        self.messages
            .first()
            .and_then(|m| Some(m.content.clone()))
            .unwrap_or_default()
    }
}
