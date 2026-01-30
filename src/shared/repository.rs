use crate::shared::error::AppResult;
use async_trait::async_trait;
use std::any::{Any, TypeId};

#[macro_export]
macro_rules! define_repo {
    ($trait_name:ident, { $($methods:tt)* }) => {
        #[async_trait::async_trait]
        pub trait $trait_name: Send + Sync {
            $($methods)*

            fn with_transaction(
                &self,
                uow: &dyn $crate::shared::repository::UnitOfWork,
            ) -> Option<Box<dyn $trait_name>>;
        }
    };
}

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

pub trait Repository<T: ?Sized>: Send + Sync {
    fn with_transaction(&self, uow: &dyn UnitOfWork) -> Option<Box<T>>;
}
