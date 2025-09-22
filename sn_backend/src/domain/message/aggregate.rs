use std::sync::Arc;

use sn_core::{
    server::payload::backend::generate_text_request::GenerateTextRequest,
    types::{
        conversation::{Conversation, ConversationBuilder},
        message::{Message, MessageBuilder, MessageRole},
    },
};
use sn_inference::model::model_runtime::GenerateTextResult;

use crate::error::{ErrorBackend, Result};

#[derive(Debug, Clone)]
pub struct MessageAggregate {
    messages: Vec<Message>,
    assistant_message: Option<Message>,
    user_message: Option<Message>,
    conversation_id: Option<i32>,
    model_id: Option<Arc<str>>,
}

impl MessageAggregate {
    pub fn new(conversation_id: Option<i32>) -> Self {
        Self {
            conversation_id,
            messages: vec![],
            assistant_message: None,
            user_message: None,
            model_id: None,
        }
    }

    pub fn messages(&self) -> &Vec<Message> {
        &self.messages
    }

    pub fn get_conversation_id(&self) -> Option<i32> {
        self.conversation_id
    }

    pub fn add_user_message(&mut self, req: &GenerateTextRequest) -> Result<()> {
        self.model_id = Some(req.model_id.clone());
        let message = MessageBuilder::default()
            .content(req.prompt.clone())
            .role(MessageRole::User)
            .build()
            .map_err(|e| ErrorBackend::Core(e.into()))?;
        self.messages.push(message.clone());
        self.user_message = Some(message);
        Ok(())
    }

    pub fn add_assistant_message(&mut self, res: GenerateTextResult) -> Result<()> {
        let (content, stats) = res;
        let message = MessageBuilder::default()
            .content(content)
            .role(MessageRole::Assistant)
            .stats(stats)
            .build()
            .map_err(|e| ErrorBackend::Core(e.into()))?;
        self.assistant_message = Some(message.clone());
        self.messages.push(message);
        Ok(())
    }

    pub fn to_conversation_core(&self) -> Result<Conversation> {
        let conversation = ConversationBuilder::default()
            .id(self.conversation_id)
            .messages(self.messages.clone())
            .build()
            .map_err(|e| ErrorBackend::Core(e))?;
        Ok(conversation)
    }

    pub fn get_assistant_message(&self) -> Option<Message> {
        self.assistant_message.clone()
    }

    pub fn get_model_id(&self) -> Option<Arc<str>> {
        self.model_id.clone()
    }

    pub fn get_user_message(&self) -> Option<Message> {
        self.user_message.clone()
    }
}
