use axum::{
    Router,
    extract::FromRef,
    routing::{get, post},
};
use sea_orm::DatabaseConnection;
use std::sync::Arc;

use crate::{
    modules::{auth::registry::OAuthProviderRegistry, users::handlers},
    shared::config::Config,
};

#[derive(Clone)]
pub struct AppState {
    pub db: Arc<DatabaseConnection>,
    pub config: Arc<Config>,
    pub registry: OAuthProviderRegistry,
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
        state.registry.clone()
    }
}

pub fn router(
    db: Arc<DatabaseConnection>,
    config: Arc<Config>,
    registry: OAuthProviderRegistry,
) -> Router {
    let state = AppState {
        db,
        config,
        registry,
    };

    Router::new()
        .route("/", post(handlers::create_user))
        .route("/:id", get(handlers::get_user))
        .route("/oauth/:provider/login", get(handlers::login_oauth))
        .route("/oauth/:provider/callback", get(handlers::callback_oauth))
        .with_state(state)
}
