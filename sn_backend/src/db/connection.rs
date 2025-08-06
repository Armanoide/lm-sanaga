use dotenv::dotenv;
use sea_orm::DatabaseConnection;
use crate::error::Result;
use std::env;
use crate::db::init_database::init_database;

pub async fn get_connection() -> Result<Option<DatabaseConnection>> {
        dotenv()?;
        let database_url = env::var("DATABASE_URL")?;
        let database_name = env::var("DATABASE_NAME")?;
        let db = init_database(&database_name, &database_url).await?;
        Ok(db)
}