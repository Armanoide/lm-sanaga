use crate::error::Result;
use migration::{Migrator, MigratorTrait};
use sea_orm::{
    ConnectOptions, ConnectionTrait, Database, DatabaseConnection, DbBackend, Statement,
};
use tracing::{info, log::LevelFilter};

pub async fn init_database(
    database_name: &str,
    database_url: &str,
) -> Result<Option<DatabaseConnection>> {
    let mut opt = ConnectOptions::new(database_url);
    opt.sqlx_logging(false)
        .sqlx_logging_level(LevelFilter::Error);

    let db = Database::connect(opt).await?;

    let db = match db.get_database_backend() {
        DbBackend::MySql => {
            info!("MySql database detected.");
            db.execute(Statement::from_string(
                db.get_database_backend(),
                format!("CREATE DATABASE IF NOT EXISTS `{}`;", database_name),
            ))
            .await?;
            Database::connect(format!("{database_url}/{database_name}")).await?
        }
        DbBackend::Postgres => {
            info!("Postgres database detected.");
            let exec_result = db
                .execute(Statement::from_string(
                    db.get_database_backend(),
                    format!("SELECT 1 FROM pg_database WHERE datname='{database_name}';"),
                ))
                .await?;
            if exec_result.rows_affected() == 0 {
                db.execute(Statement::from_string(
                    db.get_database_backend(),
                    format!("CREATE DATABASE \"{}\";", database_name),
                ))
                .await?;
            }
            Database::connect(format!("{database_url}/{database_name}")).await?
        }
        DbBackend::Sqlite => {
            info!("SQLite database detected.");
            db
        }
    };

    Migrator::down(&db, None).await?;
    Migrator::up(&db, None).await?;

    Ok(Some(db))
}
