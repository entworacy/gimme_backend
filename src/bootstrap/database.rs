use crate::shared::config::Config;
use deadpool_redis::{Config as RedisConfig, Runtime};
use migration::{Migrator, MigratorTrait};
use sea_orm::{ConnectOptions, Database, DatabaseConnection};
use std::time::Duration;

pub async fn connect_postgres(config: &Config) -> DatabaseConnection {
    let mut opt = ConnectOptions::new(&config.database_url);
    opt.max_connections(config.database_max_connections)
        .min_connections(config.database_min_connections)
        .connect_timeout(Duration::from_secs(config.database_connect_timeout))
        .idle_timeout(Duration::from_secs(config.database_idle_timeout))
        .sqlx_logging(config.app_env == "dev"); // Log only in dev

    let connection = Database::connect(opt)
        .await
        .expect("Failed to connect to database");

    Migrator::up(&connection, None)
        .await
        .expect("Failed to run migrations");

    connection
}

pub async fn connect_redis(config: &Config) -> deadpool_redis::Pool {
    tracing::info!("Connecting to Redis");
    let cfg = RedisConfig::from_url(&config.redis_url);
    cfg.create_pool(Some(Runtime::Tokio1))
        .expect("Failed to create Redis pool")
}
