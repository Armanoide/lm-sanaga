use std::sync::Arc;

use crate::domain::session::entity::IntoSession;
use crate::domain::session::repository::SessionRepository;
use crate::error::Result;
use sn_core::server::payload::backend::create_session_request::CreateSessionRequest;
use sn_core::types::session::Session;

#[derive(Clone, Debug)]
pub struct SessionService {
    repo: Arc<SessionRepository>,
}

impl SessionService {
    pub fn new(repo: Arc<SessionRepository>) -> Self {
        Self { repo }
    }

    pub async fn handle_create(&self, req: CreateSessionRequest) -> Result<Session> {
        let session = self.repo.create(req.name).await?;
        Ok(session.into_session())
    }

    // pub async fn get_session(&self, session_id: Uuid) -> Result<Option<Session>, ServiceError> {
    //     self.repo.find_by_id(session_id).await
    // }
    //
    // pub async fn delete_session(&self, session_id: Uuid) -> Result<(), ServiceError> {
    //     self.repo.delete(session_id).await
    // }
}
