pub mod database;
pub mod repositories;
pub mod services;

use crate::shared::{config::Config, state::AppState};
use std::sync::Arc;

pub async fn create_app_state(config: &Config) -> AppState {
    let db_pool = if config.app_env == "dev" {
        None
    } else {
        Some(database::connect_postgres(config).await)
    };
    let redis_pool = database::connect_redis(config).await;

    // Pass ownership of db_pool because repo needs it.
    // connect_postgres returns DatabaseConnection which is an Arc-wrapper.
    let repo_manager = repositories::init_repo_manager(config, db_pool).await;

    let auth_registry = services::init_auth_registry(config);
    let email_provider = services::init_email_provider(config);

    AppState {
        config: Arc::new(config.clone()),
        auth_registry,
        repo_manager,
        email_provider,
        redis_pool,
    }
}
