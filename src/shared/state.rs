use crate::modules::auth::registry::OAuthProviderRegistry;
use crate::modules::users::repository::UserRepository;
use crate::shared::config::Config;
use std::sync::Arc;

use crate::modules::auth::providers::email::EmailProvider;

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<Config>,
    pub auth_registry: OAuthProviderRegistry,
    pub user_repo: Arc<dyn UserRepository>,
    pub email_provider: Arc<dyn EmailProvider>,
    pub redis_pool: deadpool_redis::Pool,
}
