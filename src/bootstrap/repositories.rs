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
        Arc::new(InMemoryRepositoryManager::new()) as Arc<dyn RepositoryManager>
    } else {
        tracing::info!("Connected to PostgreSQL Repository Manager");
        let db = db.expect("Database connection is required for production");
        Arc::new(PostgresRepositoryManager::new(db)) as Arc<dyn RepositoryManager>
    }
}
