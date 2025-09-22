use sea_orm::DatabaseConnection;
use sn_inference::runner::Runner;
use std::sync::{Arc, RwLock};

use crate::{
    application::{
        conversation::service::ConversationService, embedding::service::EmbeddingService,
        message::service::MessageService, model::service::ModelService,
        session::service::SessionService,
    },
    clients::ann::AnnClient,
    domain::{
        conversation::repository::ConversationRepository,
        embedding::repository::EmbeddingRepository, message::repository::MessageRepository,
        session::repository::SessionRepository,
    },
};

#[derive(Clone, Debug)]
pub struct AppState {
    pub runner: Arc<RwLock<Runner>>,
    pub db: Arc<DatabaseConnection>,
    pub client_ann: Arc<AnnClient>,
    pub service_conversation: Arc<ConversationService>,
    pub service_session: Arc<SessionService>,
    pub service_message: Arc<MessageService>,
    pub service_embedding: Arc<EmbeddingService>,
    pub service_model: Arc<ModelService>,
}

impl AppState {
    pub fn new(runner: Arc<RwLock<Runner>>, db: DatabaseConnection, client_ann: AnnClient) -> Self {
        let db = Arc::new(db);
        let client_ann = Arc::new(client_ann);

        let repo_conversation = Arc::new(ConversationRepository::new(db.clone()));
        let repo_session = Arc::new(SessionRepository::new(db.clone()));
        let repo_message = Arc::new(MessageRepository::new(db.clone()));
        let repo_embedding = Arc::new(EmbeddingRepository::new(db.clone()));

        let service_model = Arc::new(ModelService::new(runner.clone()));
        let service_embedding = Arc::new(EmbeddingService::new(
            repo_embedding,
            runner.clone(),
            client_ann.clone(),
        ));
        let service_conversation = Arc::new(ConversationService::new(
            repo_conversation.clone(),
            repo_message.clone(),
            runner.clone(),
        ));
        let service_session = Arc::new(SessionService::new(repo_session));
        let service_message = Arc::new(MessageService::new(
            repo_message.clone(),
            service_conversation.clone(),
            runner.clone(),
            service_embedding.clone(),
        ));

        AppState {
            runner,
            db,
            client_ann,
            service_message,
            service_session,
            service_embedding,
            service_conversation,
            service_model,
        }
    }
}
