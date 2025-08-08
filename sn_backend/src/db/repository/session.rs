use crate::db::entities;
use crate::db::entities::session as Session;
use crate::error::Result;
use axum::Json;
use sea_orm::ColumnTrait;
use sea_orm::QueryFilter;
use sea_orm::{ActiveModelTrait, DatabaseConnection, EntityTrait};
use sn_core::server::payload::create_session_request::CreateSessionRequest;

/// Creates a new session with the given name, or returns the existing one if it already exists.
///
/// # Arguments
/// * `db` - A reference to the database connection.
/// * `payload` - JSON payload containing the session name.
///
/// # Returns
/// * `Ok(Model)` - The newly created or existing session model.
/// * `Err` - If a database error occurs.
pub async fn create_session(
    db: &DatabaseConnection,
    payload: Json<CreateSessionRequest>,
) -> Result<(entities::session::Model)> {
    let CreateSessionRequest { name } = payload.0;

    if let Some(session) = Session::Entity::find()
        .filter(Session::Column::Name.eq(name.clone()))
        .one(db)
        .await?
    {
        return Ok(session);
    }

    let new_session = Session::ActiveModel {
        id: Default::default(),
        name: sea_orm::Set(name),
    };
    Ok(new_session.insert(db).await?)
}

/// Retrieves a session by its ID from the database.
///
/// # Arguments
/// * `db` - A reference to the database connection.
/// * `session_id` - The ID of the session to retrieve.
///
/// # Returns
/// * `Ok(Some(Model))` - If the session exists.
/// * `Ok(None)` - If no session with the given ID is found.
/// * `Err` - If a database error occurs.
pub async fn get_session(
    db: &DatabaseConnection,
    session_id: i32,
) -> Result<Option<entities::session::Model>> {
    let session = Session::Entity::find()
        .filter(Session::Column::Id.eq(session_id))
        .one(db)
        .await?;
    Ok(session)
}
