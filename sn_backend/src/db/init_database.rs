use sea_orm::{ConnectionTrait, DatabaseConnection, DbBackend, DbErr};
use crate::error::Result;

pub async fn init_database(database_name: &str, database_url: &str) -> Result<()> {
    Ok(())
}