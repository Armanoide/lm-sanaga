use crate::{error::Result, infrastructure::db::init_database::init_database};
use dotenv::dotenv;
use sea_orm::DatabaseConnection;
use std::env;

pub async fn get_connection() -> Result<Option<DatabaseConnection>> {
    dotenv()?;
    let database_url = env::var("DATABASE_URL")?;
    let database_name = env::var("DATABASE_NAME")?;
    let db = init_database(&database_name, &database_url).await?;
    Ok(db)
}
