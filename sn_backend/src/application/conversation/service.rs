use std::sync::{Arc, RwLock};

use crate::{
    domain::{
        conversation::{
            aggregate::ConversationAggregate,
            entity::{IntoConversation, IntoConversations},
        },
        message::{entity::IntoMessage, repository::MessageRepository},
    },
    error::{ErrorBackend, Result},
    use_cases::conversation::generate_name_use_case::GenerateNameUseCase,
};
use sn_core::{
    server::payload::backend::generate_text_request::GenerateTextRequest,
    types::{conversation::Conversation, message::Message},
};
use sn_inference::runner::Runner;

use crate::domain::conversation::repository::ConversationRepository;

#[derive(Clone, Debug)]
pub struct ConversationService {
    repo_conversation: Arc<ConversationRepository>,
    repo_message: Arc<MessageRepository>,
    runner: Arc<RwLock<Runner>>,
}

impl ConversationService {
    pub fn new(
        repo_conversation: Arc<ConversationRepository>,
        repo_message: Arc<MessageRepository>,
        runner: Arc<RwLock<Runner>>,
    ) -> Self {
        Self {
            repo_conversation,
            repo_message,
            runner,
        }
    }

    pub async fn list_conversations(&self, session_id: &i32) -> Result<Vec<Conversation>> {
        let conversations = self
            .repo_conversation
            .find_all_by_session_id(session_id)
            .await?;
        Ok(conversations.into_conversations())
    }

    pub async fn get_or_create(&self, req: &GenerateTextRequest) -> Result<Option<Conversation>> {
        let session_id = match req.session_id {
            Some(id) => id,
            None => return Ok(None),
        };
        let conversation = if let Some(conversation_id) = req.conversation_id {
            self.repo_conversation
                .find_by_id(&conversation_id)
                .await?
                .unwrap_or(self.repo_conversation.create(&session_id).await?)
        } else {
            self.repo_conversation.create(&session_id).await?
        };
        Ok(Some(conversation.into_conversation()))
    }

    pub async fn generate_name(&self, model_id: Arc<str>, conversation_id: &i32) -> Result<()> {
        let conversation = match self.repo_conversation.find_by_id(conversation_id).await? {
            Some(conversation) => conversation,
            None => return Err(ErrorBackend::ConversationNotFound),
        };
        let use_case = GenerateNameUseCase::new(self.runner.clone());
        let mut agg = ConversationAggregate::new(conversation.into_conversation());
        let user_message = match self
            .repo_message
            .find_first_of_conversation(&conversation_id)
            .await
        {
            Ok(Some(message)) => message,
            _ => return Ok(()),
        };
        agg.add_user_message(user_message.into_message());
        let conversation_name = use_case.generate(model_id, agg).await?;
        let conversation_name = Message::sanitize_content(&conversation_name);
        self.repo_conversation
            .update_name(&conversation_id, conversation_name)
            .await;
        Ok(())
    }
}
