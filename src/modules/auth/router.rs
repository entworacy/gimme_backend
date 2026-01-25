use super::handlers;
use crate::shared::config::Config;
use axum::{Router, extract::FromRef, routing::get};
use sea_orm::DatabaseConnection;
use std::sync::Arc;

use crate::modules::auth::registry::OAuthProviderRegistry;

#[derive(Clone)]
pub struct AppState {
    pub db: Arc<DatabaseConnection>,
    pub config: Arc<Config>,
    pub auth_registry: OAuthProviderRegistry,
}

impl FromRef<AppState> for Arc<DatabaseConnection> {
    fn from_ref(state: &AppState) -> Self {
        state.db.clone()
    }
}

impl FromRef<AppState> for Arc<Config> {
    fn from_ref(state: &AppState) -> Self {
        state.config.clone()
    }
}

impl FromRef<AppState> for OAuthProviderRegistry {
    fn from_ref(state: &AppState) -> Self {
        state.auth_registry.clone()
    }
}

pub fn router(
    db: Arc<DatabaseConnection>,
    config: Arc<Config>,
    auth_registry: OAuthProviderRegistry,
) -> Router {
    let state = AppState {
        db,
        config,
        auth_registry,
    };

    Router::new()
        .route("/login/kakao", get(handlers::login_kakao))
        .route("/callback/kakao", get(handlers::callback_kakao))
        .with_state(state)
}
