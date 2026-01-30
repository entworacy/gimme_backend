use super::entities::delivery_data;
use crate::shared::error::AppResult;
crate::define_repo!(DeliveryRepository, {
    async fn find_by_id(&self, id: i32) -> AppResult<Option<delivery_data::Model>>;
});
