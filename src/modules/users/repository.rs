use async_trait::async_trait;
use sea_orm::*;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use super::entities::{social, user, verification};
use crate::shared::error::{AppError, AppResult};

#[async_trait]
pub trait TxUserRepository: UserRepository {
    async fn commit(self: Box<Self>) -> AppResult<()>;
    async fn rollback(self: Box<Self>) -> AppResult<()>;
}

#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn find_by_id(&self, id: i32) -> AppResult<Option<user::Model>>;
    async fn find_by_uuid(&self, uuid: &str) -> AppResult<Option<user::Model>>;
    async fn find_by_email(&self, email: &str) -> AppResult<Option<user::Model>>;
    async fn find_social(
        &self,
        provider: social::SocialProvider,
        provider_id: &str,
    ) -> AppResult<Option<social::Model>>;

    async fn create_user_with_verification(
        &self,
        user: user::ActiveModel,
        social: Option<social::ActiveModel>,
        verification: verification::ActiveModel,
    ) -> AppResult<user::Model>;

    async fn update_user(&self, user: user::ActiveModel) -> AppResult<user::Model>;

    async fn find_with_details_by_uuid(
        &self,
        uuid: &str,
    ) -> AppResult<Option<(user::Model, Option<verification::Model>, Vec<social::Model>)>>;

    async fn update_verification(
        &self,
        verification: verification::ActiveModel,
    ) -> AppResult<verification::Model>;

    // Deprecated for direct usage, but kept for compatibility or internal logic if needed
    async fn begin_txn(&self) -> AppResult<Box<dyn TxUserRepository>>;

    fn with_transaction(
        &self,
        uow: &dyn crate::shared::repository::UnitOfWork,
    ) -> Option<Box<dyn UserRepository>>;
}
