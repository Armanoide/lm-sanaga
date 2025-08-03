use axum::Json;
use sea_orm::{ActiveModelTrait, DatabaseConnection, EntityTrait};
use sn_core::server::payload::create_session_request::CreateSessionRequest;
use crate::db::entities;
use crate::error::Result;
use crate::db::entities::session as Session;
use sea_orm::QueryFilter;
use sea_orm::ColumnTrait;
pub async fn create_session(
    db: &DatabaseConnection,
    payload: Json<CreateSessionRequest>
) -> Result<(entities::session::Model)> {
    let CreateSessionRequest {  name } = payload.0;

    if let Some(session) = Session::Entity::find()
        .filter(Session::Column::Name.eq(name.clone()))
        .one(db)
        .await? {
        return Ok(session)
    }

    let new_session = Session::ActiveModel {
        id: Default::default(),
        name: sea_orm::Set(name),
    };
    Ok(new_session.insert(db).await?)
}

pub async fn get_session(
    db: &DatabaseConnection,
    session_id: i32
) -> Result<Option<entities::session::Model>> {
    let session = Session::Entity::find()
        .filter(Session::Column::Id.eq(session_id))
        .one(db)
        .await?;
    Ok(session)
}