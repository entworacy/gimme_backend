use crate::modules::auth::registry::OAuthProviderRegistry;
use crate::shared::config::Config;
use crate::shared::repository::RepositoryManager;
use std::sync::Arc;

use crate::modules::auth::providers::email::EmailProvider;

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<Config>,
    pub auth_registry: OAuthProviderRegistry,
    pub repo_manager: Arc<dyn RepositoryManager>,
    pub email_provider: Arc<dyn EmailProvider>,
    pub redis_pool: deadpool_redis::Pool,
}
