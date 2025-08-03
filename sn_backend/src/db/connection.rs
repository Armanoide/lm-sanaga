use dotenv::dotenv;
use sea_orm::DatabaseConnection;
use crate::error::Result;
use std::env;
use crate::db::init_database::init_database;

pub async fn get_connection() -> Result<Option<DatabaseConnection>> {
        dotenv()?;
        let database_url = env::var("DATABASE_URL")?;
        let database_name = env::var("DATABASE_NAME")?;
        init_database(&database_name, &database_name).await?;
        Ok(Some(sea_orm::Database::connect(database_url).await?))
}