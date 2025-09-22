use std::sync::Arc;

use crate::domain;
use crate::error::Result;
use sea_orm::ColumnTrait;
use sea_orm::QueryFilter;
use sea_orm::{ActiveModelTrait, DatabaseConnection, EntityTrait};

#[derive(Clone, Debug)]
pub struct SessionRepository {
    db: Arc<DatabaseConnection>,
}

impl SessionRepository {
    pub fn new(db: Arc<DatabaseConnection>) -> SessionRepository {
        SessionRepository { db }
    }

    pub async fn create(&self, name: Option<String>) -> Result<domain::session::entity::Model> {
        let session = if let Some(session) = self.find_by_name(name.clone()).await? {
            session
        } else {
            let new_session = domain::session::entity::ActiveModel {
                id: Default::default(),
                name: sea_orm::Set(name),
            };
            new_session.insert(self.db.as_ref()).await?
        };
        Ok(session)
    }

    pub async fn find_by_name(
        &self,
        name: Option<String>,
    ) -> Result<Option<domain::session::entity::Model>> {
        let session = domain::session::entity::Entity::find()
            .filter(domain::session::entity::Column::Name.eq(name))
            .one(self.db.as_ref())
            .await?;
        Ok(session)
    }

    pub async fn find_by_id(&self, id: &i32) -> Result<Option<domain::session::entity::Model>> {
        let session = domain::session::entity::Entity::find()
            .filter(domain::session::entity::Column::Id.eq(*id))
            .one(self.db.as_ref())
            .await?;
        Ok(session)
    }
}
