use super::entities::delivery_data;
use crate::shared::error::AppResult;
use async_trait::async_trait;

#[async_trait]
pub trait DeliveryRepository: Send + Sync {
    async fn find_by_id(&self, id: i32) -> AppResult<Option<delivery_data::Model>>;

    fn with_transaction(
        &self,
        uow: &dyn crate::shared::repository::UnitOfWork,
    ) -> Option<Box<dyn DeliveryRepository>>;
}
