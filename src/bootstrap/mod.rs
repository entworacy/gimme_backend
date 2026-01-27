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
    let user_repo = repositories::init_user_repo(config, db_pool).await;

    let auth_registry = services::init_auth_registry(config);
    let email_provider = services::init_email_provider(config);

    AppState {
        config: Arc::new(config.clone()),
        auth_registry,
        user_repo,
        email_provider,
        redis_pool,
    }
}
