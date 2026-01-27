use crate::shared::config::Config;
use crate::shared::infra::repository::{InMemoryRepositoryManager, PostgresRepositoryManager};
use crate::shared::repository::RepositoryManager;
use sea_orm::DatabaseConnection;
use std::sync::Arc;

pub async fn init_repo_manager(
    config: &Config,
    db: Option<DatabaseConnection>,
) -> Arc<dyn RepositoryManager> {
    if config.app_env == "dev" {
        tracing::warn!("Using InMemory Repository Manager for Dev Env");
        if db.is_none() {
            tracing::debug!(
                "인메모리 데이터베이스를 사용하는 중으로 Repository의 데이터베이스에는 None이 할당되어 있습니다."
            );
        }
        let mut manager = InMemoryRepositoryManager::new();
        let user_repo = crate::shared::infra::repository::InMemoryUserRepository::default();

        manager.register::<Arc<dyn crate::modules::users::repository::UserRepository>>(Arc::new(
            user_repo,
        ));

        Arc::new(manager) as Arc<dyn RepositoryManager>
    } else {
        tracing::info!("Connected to PostgreSQL Repository Manager");
        let db = Arc::new(db.expect("Database connection is required for production"));
        let mut manager = PostgresRepositoryManager::new(db.clone());
        let user_repo = crate::shared::infra::repository::PostgresUserRepository::new(db.clone());

        manager.register::<Arc<dyn crate::modules::users::repository::UserRepository>>(Arc::new(
            user_repo,
        ));

        Arc::new(manager) as Arc<dyn RepositoryManager>
    }
}
