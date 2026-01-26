use crate::modules::auth::registry::OAuthProviderRegistry;
use crate::modules::users::repository::UserRepository;
use crate::shared::config::Config;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<Config>,
    pub auth_registry: OAuthProviderRegistry,
    pub user_repo: Arc<dyn UserRepository>,
}
