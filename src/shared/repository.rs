use crate::shared::error::AppResult;
use async_trait::async_trait;
use std::any::{Any, TypeId};

pub trait AsAny {
    fn as_any(&self) -> &dyn Any;
}

#[async_trait]
pub trait UnitOfWork: AsAny + Send + Sync {
    async fn commit(self: Box<Self>) -> AppResult<()>;
    async fn rollback(self: Box<Self>) -> AppResult<()>;
}

#[async_trait]
pub trait RepositoryManager: Send + Sync {
    fn get_repository(&self, type_id: TypeId) -> Option<&(dyn Any + Send + Sync)>;

    async fn begin(&self) -> AppResult<Box<dyn UnitOfWork>>;
}

impl dyn RepositoryManager {
    pub fn get<T: 'static + Send + Sync>(&self) -> Option<&T> {
        self.get_repository(TypeId::of::<T>())
            .and_then(|r| r.downcast_ref())
    }
}
