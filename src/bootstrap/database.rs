use crate::shared::config::Config;
use deadpool_redis::{Config as RedisConfig, Runtime};
use sea_orm::{Database, DatabaseConnection};

pub async fn connect_postgres(config: &Config) -> DatabaseConnection {
    Database::connect(&config.database_url)
        .await
        .expect("Failed to connect to database")
}

pub async fn connect_redis(config: &Config) -> deadpool_redis::Pool {
    let cfg = RedisConfig::from_url(&config.redis_url);
    cfg.create_pool(Some(Runtime::Tokio1))
        .expect("Failed to create Redis pool")
}
