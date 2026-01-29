use async_trait::async_trait;
use sea_orm::*;
use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex as AsyncMutex;

use crate::shared::error::{AppError, AppResult};
use crate::shared::repository::{AsAny, RepositoryManager, UnitOfWork};

// =========================================================================
// Postgres Implementation (Generic Base)
// =========================================================================

#[derive(Clone)]
pub enum DbOrTxn {
    Conn(Arc<DatabaseConnection>),
    Txn(Arc<AsyncMutex<Option<DatabaseTransaction>>>),
}

#[derive(Clone)]
pub struct SeaOrmRepository<E>
where
    E: EntityTrait,
{
    pub conn: DbOrTxn,
    _marker: std::marker::PhantomData<E>,
}

impl<E> SeaOrmRepository<E>
where
    E: EntityTrait,
{
    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        tracing::info!("Creating SeaOrmRepository...");
        tracing::info!("Entity Type: {}", std::any::type_name::<E>());
        Self {
            conn: DbOrTxn::Conn(db),
            _marker: std::marker::PhantomData,
        }
    }

    pub fn get_connection(&self) -> &DbOrTxn {
        &self.conn
    }

    pub fn with_transaction_internal(&self, uow: &dyn UnitOfWork) -> Option<Self> {
        let postgres_uow = uow.as_any().downcast_ref::<PostgresUnitOfWork>()?;
        Some(Self {
            conn: DbOrTxn::Txn(postgres_uow.txn.clone()),
            _marker: std::marker::PhantomData,
        })
    }
}

// Macro to implement Repository Trait with boilerplate with_transaction
#[macro_export]
macro_rules! impl_sea_orm_repo {
    ($repo_type:ty, $trait_path:path, { $($methods:tt)* }) => {
        #[async_trait::async_trait]
        impl $trait_path for $repo_type {
            $($methods)*

            fn with_transaction(&self, uow: &dyn $crate::shared::repository::UnitOfWork) -> Option<Box<dyn $trait_path>> {
                 self.with_transaction_internal(uow)
                    .map(|r| Box::new(r) as Box<dyn $trait_path>)
            }
        }
    };
}

pub struct PostgresRepositoryManager {
    db: Arc<DatabaseConnection>,
    repos: HashMap<TypeId, Arc<dyn Any + Send + Sync>>,
}

impl PostgresRepositoryManager {
    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        tracing::info!("Creating PostgresRepositoryManager...");
        Self {
            db,
            repos: HashMap::new(),
        }
    }

    pub fn register<T: 'static + Send + Sync>(&mut self, repo: T) {
        self.repos.insert(TypeId::of::<T>(), Arc::new(repo));
    }
}

#[async_trait]
impl RepositoryManager for PostgresRepositoryManager {
    fn get_repository(&self, type_id: TypeId) -> Option<&(dyn Any + Send + Sync)> {
        self.repos.get(&type_id).map(|boxed| boxed.as_ref())
    }

    async fn begin(&self) -> AppResult<Box<dyn UnitOfWork>> {
        let txn = self.db.begin().await.map_err(AppError::DbError)?;
        Ok(Box::new(PostgresUnitOfWork {
            txn: Arc::new(AsyncMutex::new(Some(txn))),
        }))
    }
}

impl AsAny for PostgresUnitOfWork {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

pub struct PostgresUnitOfWork {
    txn: Arc<AsyncMutex<Option<DatabaseTransaction>>>,
}

#[async_trait]
impl UnitOfWork for PostgresUnitOfWork {
    async fn commit(self: Box<Self>) -> AppResult<()> {
        let mut lock = self.txn.lock().await;
        if let Some(txn) = lock.take() {
            txn.commit().await.map_err(AppError::DbError)
        } else {
            Ok(())
        }
    }

    async fn rollback(self: Box<Self>) -> AppResult<()> {
        let mut lock = self.txn.lock().await;
        if let Some(txn) = lock.take() {
            txn.rollback().await.map_err(AppError::DbError)
        } else {
            Ok(())
        }
    }
}

// =========================================================================
// InMemory Implementation
// =========================================================================

#[derive(Default, Clone)]
pub struct InMemoryRepositoryManager {
    repos: HashMap<TypeId, Arc<dyn Any + Send + Sync>>,
}

impl InMemoryRepositoryManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register<T: 'static + Send + Sync>(&mut self, repo: T) {
        self.repos.insert(TypeId::of::<T>(), Arc::new(repo));
    }
}

pub struct InMemoryUnitOfWork {
    // No specific fields needed for dummy implementation unless we want to simulate txn
}

impl AsAny for InMemoryUnitOfWork {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[async_trait]
impl RepositoryManager for InMemoryRepositoryManager {
    fn get_repository(&self, type_id: TypeId) -> Option<&(dyn Any + Send + Sync)> {
        self.repos.get(&type_id).map(|boxed| boxed.as_ref())
    }

    async fn begin(&self) -> AppResult<Box<dyn UnitOfWork>> {
        Ok(Box::new(InMemoryUnitOfWork {}))
    }
}

#[async_trait]
impl UnitOfWork for InMemoryUnitOfWork {
    async fn commit(self: Box<Self>) -> AppResult<()> {
        Ok(())
    }
    async fn rollback(self: Box<Self>) -> AppResult<()> {
        Ok(())
    }
}
