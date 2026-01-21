use sea_orm::{Database, DatabaseConnection, DbErr, MockDatabase, DatabaseBackend};
use crate::config::Config;

pub async fn connect(config: &Config) -> Result<DatabaseConnection, DbErr> {
    if config.app_env == "dev" {
        tracing::warn!("APP_ENV is 'dev', using MockDatabase. Database queries will generally fail if not mocked.");
        // Create a MockDatabase connection. 
        // Note: For a real usable dev server without DB, SQLite is usually better. 
        // But requested is MockDatabase. 
        // We initialize it with no expectations, so any query will likely panic/fail 
        // unless we somehow inject expectations (which is hard in main run).
        // However, this allows the server to START.
        Ok(MockDatabase::new(DatabaseBackend::Postgres)
            .into_connection())
    } else {
        Database::connect(&config.database_url).await
    }
}
