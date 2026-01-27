use crate::modules::users::repository::UserRepository;
use crate::shared::error::AppResult;
use async_trait::async_trait;

#[async_trait]
pub trait UnitOfWork: Send + Sync {
    fn user_repo(&self) -> Box<dyn UserRepository>;

    async fn commit(self: Box<Self>) -> AppResult<()>;
    async fn rollback(self: Box<Self>) -> AppResult<()>;
}

#[async_trait]
pub trait RepositoryManager: Send + Sync {
    fn user_repo(&self) -> Box<dyn UserRepository>;

    async fn begin(&self) -> AppResult<Box<dyn UnitOfWork>>;
}
