use async_trait::async_trait;
use sea_orm::*;

use crate::impl_sea_orm_repo;
use crate::modules::delivery::entities::delivery_data;
use crate::modules::delivery::repository::DeliveryRepository;
use crate::shared::error::{AppError, AppResult};
use crate::shared::infra::repository::{DbOrTxn, SeaOrmRepository};
use crate::shared::repository::UnitOfWork;

// =========================================================================
// Postgres Implementation
// =========================================================================

pub type PostgresDeliveryRepository = SeaOrmRepository<delivery_data::Entity>;

impl_sea_orm_repo!(PostgresDeliveryRepository, DeliveryRepository, {
    async fn find_by_id(&self, id: i32) -> AppResult<Option<delivery_data::Model>> {
        match &self.conn {
            DbOrTxn::Conn(c) => delivery_data::Entity::find_by_id(id)
                .one(c.as_ref())
                .await
                .map_err(AppError::DbError),
            DbOrTxn::Txn(mutex) => {
                let lock = mutex.lock().await;
                let txn = lock.as_ref().ok_or(AppError::InternalServerError(
                    "Transaction unavailable".to_string(),
                ))?;
                delivery_data::Entity::find_by_id(id)
                    .one(txn)
                    .await
                    .map_err(AppError::DbError)
            }
        }
    }
});

// =========================================================================
// InMemory Implementation
// =========================================================================

#[derive(Clone, Default)]
pub struct InMemoryDeliveryRepository {}

#[async_trait]
impl DeliveryRepository for InMemoryDeliveryRepository {
    async fn find_by_id(&self, _id: i32) -> AppResult<Option<delivery_data::Model>> {
        Ok(None) // Dummy implementation
    }

    fn with_transaction(&self, _uow: &dyn UnitOfWork) -> Option<Box<dyn DeliveryRepository>> {
        Some(Box::new(self.clone()))
    }
}
