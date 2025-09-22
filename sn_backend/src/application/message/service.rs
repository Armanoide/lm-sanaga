use crate::{
    application::embedding::service::EmbeddingService,
    domain::message::{aggregate::MessageAggregate, entity::IntoMessage},
    error::{ErrorBackend, Result},
    use_cases::message::generate_text_use_case::GenerateTextUseCase,
    utils::stream_channel::StreamChannel,
};
use futures::future::join_all;
use sn_core::{
    server::payload::backend::generate_text_request::GenerateTextRequest, types::message::Message,
};
use sn_inference::runner::Runner;
use std::sync::{Arc, RwLock};
use tracing::error;

use crate::{
    application::conversation::service::ConversationService,
    domain::message::{repository::MessageRepository, value_object::GenerateTextOutput},
};

#[derive(Clone, Debug)]
pub struct MessageService {
    service_conversation: Arc<ConversationService>,
    service_background: Arc<MessageBackgroundService>,
    runner: Arc<RwLock<Runner>>,
    repo_message: Arc<MessageRepository>,
}

impl MessageService {
    pub fn new(
        repo_message: Arc<MessageRepository>,
        service_conversation: Arc<ConversationService>,
        runner: Arc<RwLock<Runner>>,
        service_embedding: Arc<EmbeddingService>,
    ) -> MessageService {
        let service_background = Arc::new(MessageBackgroundService::new(
            repo_message.clone(),
            service_conversation.clone(),
            service_embedding,
        ));
        MessageService {
            repo_message,
            service_conversation,
            runner,
            service_background,
        }
    }

    pub async fn generate_text(&self, req: GenerateTextRequest) -> Result<GenerateTextOutput> {
        let conversation = self.service_conversation.get_or_create(&req).await?;
        let stream = StreamChannel::new(&req.stream);
        let mut agg = MessageAggregate::new(conversation.and_then(|c| c.id));
        let use_case = GenerateTextUseCase::new(self.runner.clone());

        let _ = agg.add_user_message(&req)?;
        let result = use_case
            .generate(stream.as_ref(), agg, req.model_id.clone(), req.session_id)
            .await?;

        let result = self.service_background.clone().execute(result);

        Ok(result)
    }

    // pub async fn populate_conversation_with_similarity_message(
    //     state: Arc<AppState>,
    //     conversation_id: Option<i32>,
    //     new_message: String,
    // ) -> Result<(Conversation)> {
    //     let db = match &state.db {
    //         Some(db) => db,
    //         None => return Err(ErrorBackend::NoDbAvailable),
    //     };
    //     let client_ann = match &state.client_ann {
    //         Some(client) => client,
    //         None => return Err(ErrorBackend::NoAnnClientAvailable),
    //     };
    //     let conversation_id = match conversation_id {
    //         Some(id) => id,
    //         None => return Err(ErrorBackend::ConversationNotFound),
    //     };
    //     let mut conversation = repository::conversation::get_conversation_by_id(db, &conversation_id)
    //         .await?
    //         .ok_or(ErrorBackend::ConversationNotFound)?
    //         .into_conversation();
    //     match sync_embeddings(db, conversation_id, client_ann).await {
    //         Ok(_) => {}
    //         Err(e) => {
    //             error!("Failed to sync embeddings: {}", e);
    //         }
    //     };
    //     let vectors = state
    //         .runner
    //         .read_lock("populate_conversation_with_similarity_message")?
    //         .generate_embeddings(&vec![new_message])
    //         .map_err(|e| ErrorBackend::Inference(e))?;
    //     let vectors = vectors.as_slice::<f32>().to_vec();
    //     let retrival_ann = client_ann
    //         .search_similarity(SearchRequest {
    //             vectors,
    //             k: 5,
    //             nprobe: 10,
    //             partition_id: conversation_id,
    //         })
    //         .await?;
    //     let futures = retrival_ann
    //         .vectors
    //         .iter()
    //         .map(|a| async { repository::message::get_message_by_id(db, a.primary_key).await })
    //         .collect::<Vec<_>>();
    //
    //     let results_messages = join_all(futures).await;
    //     let messages: Vec<Message> = results_messages
    //         .into_iter()
    //         .filter_map(|res| match res {
    //             Ok(Some(m)) => Some(m.into_message()),
    //             _ => None,
    //         })
    //         .collect();
    //     conversation.messages.extend(messages);
    //     Ok(conversation)
    // }
}

#[derive(Clone, Debug)]
pub struct MessageBackgroundService {
    repo_message: Arc<MessageRepository>,
    service_conversation: Arc<ConversationService>,
    service_embedding: Arc<EmbeddingService>,
}

impl MessageBackgroundService {
    pub fn new(
        repo_message: Arc<MessageRepository>,
        service_conversation: Arc<ConversationService>,
        service_embedding: Arc<EmbeddingService>,
    ) -> MessageBackgroundService {
        MessageBackgroundService {
            repo_message,
            service_conversation,
            service_embedding,
        }
    }

    async fn handle_persist_message(&self, agg: MessageAggregate) -> Result<(Message, Message)> {
        if let (Some(assistant_message), Some(user_message), Some(conversation_id)) = (
            agg.get_assistant_message(),
            agg.get_user_message(),
            agg.get_conversation_id(),
        ) {
            let (user, assistant) = self
                .repo_message
                .create(
                    &conversation_id,
                    assistant_message.content.clone(),
                    assistant_message.stats.clone(),
                    user_message.content.clone(),
                )
                .await?;
            return Ok((user.into_message(), assistant.into_message()));
        }
        return Err(ErrorBackend::FailedToPersist(
            "failed to persist message".to_string(),
        ));
    }

    async fn handle_background_tasks(&self, agg: MessageAggregate) {
        let _ = || async {
            let model_id = &agg.get_model_id().ok_or_else(|| {
                ErrorBackend::MessageBackgroundParamNotFound("model_id".to_string())
            })?;
            let conversation_id = &agg.get_conversation_id().ok_or_else(|| {
                ErrorBackend::MessageBackgroundParamNotFound("conversation_id".to_string())
            })?;
            let (user_message, assistant_message) = self.handle_persist_message(agg).await?;

            self.service_conversation
                .generate_name(model_id.clone(), conversation_id)
                .await
                .err()
                .map(|e| error!("Failed to generate conversation name: {}", e));

            let futures = vec![user_message.clone(), assistant_message.clone()]
                .into_iter()
                .map(|m| self.service_embedding.generate_embedding(m));

            join_all(futures).await;
            Ok::<_, ErrorBackend>(())
        };
    }

    pub fn execute(self: Arc<Self>, output: GenerateTextOutput) -> GenerateTextOutput {
        match output {
            GenerateTextOutput::Json(agg) => {
                let agg_clone = agg.clone();
                let this = self;
                tokio::spawn(async move {
                    this.handle_background_tasks(agg_clone.clone());
                    Ok::<_, ErrorBackend>(agg_clone)
                });
                GenerateTextOutput::Json(agg)
            }

            GenerateTextOutput::Streaming {
                receiver,
                completion,
            } => {
                let this = self;
                if let Some(fut) = completion {
                    tokio::spawn(async move {
                        let agg = fut.await?;
                        this.handle_background_tasks(agg);
                        Ok::<_, ErrorBackend>(())
                    });
                }
                GenerateTextOutput::Streaming {
                    receiver,
                    completion: None,
                }
            }
        }
    }
}
