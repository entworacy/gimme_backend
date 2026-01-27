use crate::modules::users::repository::{
    InMemoryUserRepository, PostgresUserRepository, UserRepository,
};
use crate::shared::config::Config;
use sea_orm::DatabaseConnection;
use std::sync::Arc;

pub async fn init_user_repo(
    config: &Config,
    db: Option<DatabaseConnection>,
) -> Arc<dyn UserRepository> {
    if config.app_env == "dev" {
        tracing::warn!("Using InMemory Repository for Dev Env");
        if db.is_none() {
            tracing::debug!(
                "인메모리 데이터베이스를 사용하는 중으로 Repository의 데이터베이스에는 None이 할당되어 있습니다."
            );
        }
        Arc::new(InMemoryUserRepository::new()) as Arc<dyn UserRepository>
    } else {
        tracing::info!("Connected to PostgreSQL User Repo");
        let db = db.expect("Database connection is required for production");
        Arc::new(PostgresUserRepository::new(Arc::new(db))) as Arc<dyn UserRepository>
    }
}
