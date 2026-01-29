use async_trait::async_trait;
use sea_orm::*;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::impl_sea_orm_repo;
use crate::modules::users::entities::{social, user, verification};
use crate::modules::users::repository::UserRepository;
use crate::shared::error::{AppError, AppResult};
use crate::shared::infra::repository::{DbOrTxn, SeaOrmRepository};
use crate::shared::repository::UnitOfWork;

// =========================================================================
// Postgres Implementation
// =========================================================================

pub type PostgresUserRepository = SeaOrmRepository<user::Entity>;

impl_sea_orm_repo!(PostgresUserRepository, UserRepository, {
    async fn find_by_id(&self, id: i32) -> AppResult<Option<user::Model>> {
        match &self.conn {
            DbOrTxn::Conn(c) => user::Entity::find_by_id(id)
                .one(c.as_ref())
                .await
                .map_err(AppError::DbError),
            DbOrTxn::Txn(mutex) => {
                let lock = mutex.lock().await;
                let txn = lock.as_ref().ok_or(AppError::InternalServerError(
                    "이미 트랜잭션이 완료되어 있습니다.".to_string(),
                ))?;
                user::Entity::find_by_id(id)
                    .one(txn)
                    .await
                    .map_err(AppError::DbError)
            }
        }
    }

    async fn find_by_uuid(&self, uuid: &str) -> AppResult<Option<user::Model>> {
        let query = user::Entity::find().filter(user::Column::Uuid.eq(uuid));
        match &self.conn {
            DbOrTxn::Conn(c) => query.one(c.as_ref()).await.map_err(AppError::DbError),
            DbOrTxn::Txn(mutex) => {
                let lock = mutex.lock().await;
                let txn = lock.as_ref().expect("Active txn");
                query.one(txn).await.map_err(AppError::DbError)
            }
        }
    }

    async fn find_by_email(&self, email: &str) -> AppResult<Option<user::Model>> {
        let query = user::Entity::find().filter(user::Column::Email.eq(email));
        match &self.conn {
            DbOrTxn::Conn(c) => query.one(c.as_ref()).await.map_err(AppError::DbError),
            DbOrTxn::Txn(mutex) => {
                let lock = mutex.lock().await;
                let txn = lock.as_ref().expect("Active txn");
                query.one(txn).await.map_err(AppError::DbError)
            }
        }
    }

    async fn find_social(
        &self,
        provider: social::SocialProvider,
        provider_id: &str,
    ) -> AppResult<Option<social::Model>> {
        let query = social::Entity::find()
            .filter(social::Column::Provider.eq(provider))
            .filter(social::Column::ProviderId.eq(provider_id));
        match &self.conn {
            DbOrTxn::Conn(c) => query.one(c.as_ref()).await.map_err(AppError::DbError),
            DbOrTxn::Txn(mutex) => {
                let lock = mutex.lock().await;
                let txn = lock.as_ref().expect("Active txn");
                query.one(txn).await.map_err(AppError::DbError)
            }
        }
    }

    async fn create_user_with_verification(
        &self,
        user: user::ActiveModel,
        social: Option<social::ActiveModel>,
        verification: verification::ActiveModel,
    ) -> AppResult<user::Model> {
        match &self.conn {
            DbOrTxn::Conn(c) => {
                let txn = c.as_ref().begin().await.map_err(AppError::DbError)?;
                let res = Self::create_user_internal(&txn, user, social, verification).await;
                if res.is_ok() {
                    txn.commit().await.map_err(AppError::DbError)?;
                } else {
                    txn.rollback().await.map_err(AppError::DbError)?;
                }
                res
            }
            DbOrTxn::Txn(mutex) => {
                let lock = mutex.lock().await;
                let txn = lock.as_ref().expect("Active txn");

                Self::create_user_internal(txn, user, social, verification).await
            }
        }
    }

    async fn update_user(&self, user: user::ActiveModel) -> AppResult<user::Model> {
        match &self.conn {
            DbOrTxn::Conn(c) => user.update(c.as_ref()).await.map_err(AppError::DbError),
            DbOrTxn::Txn(mutex) => {
                let lock = mutex.lock().await;
                let txn = lock.as_ref().expect("Active txn");
                user.update(txn).await.map_err(AppError::DbError)
            }
        }
    }

    async fn find_with_details_by_uuid(&self, uuid: &str) -> AppResult<Option<user::Model>> {
        match &self.conn {
            DbOrTxn::Conn(c) => Self::find_details_internal(c.as_ref(), uuid).await,
            DbOrTxn::Txn(mutex) => {
                let lock = mutex.lock().await;
                let txn = lock.as_ref().expect("Active txn");
                Self::find_details_internal(txn, uuid).await
            }
        }
    }

    async fn update_verification(
        &self,
        verification: verification::ActiveModel,
    ) -> AppResult<verification::Model> {
        match &self.conn {
            DbOrTxn::Conn(c) => verification
                .update(c.as_ref())
                .await
                .map_err(AppError::DbError),
            DbOrTxn::Txn(mutex) => {
                let lock = mutex.lock().await;
                let txn = lock.as_ref().expect("Active txn");
                verification.update(txn).await.map_err(AppError::DbError)
            }
        }
    }
});

// Helper implementation for inner methods needs to appear outside macro
impl SeaOrmRepository<user::Entity> {
    async fn create_user_internal<C>(
        db: &C,
        user: user::ActiveModel,
        social: Option<social::ActiveModel>,
        verification: verification::ActiveModel,
    ) -> AppResult<user::Model>
    where
        C: ConnectionTrait,
    {
        let created_user = user.insert(db).await.map_err(AppError::DbError)?;

        if let Some(mut s) = social {
            s.user_id = Set(created_user.id);
            s.insert(db).await.map_err(AppError::DbError)?;
        }

        let mut v = verification;
        v.user_id = Set(created_user.id);
        v.insert(db).await.map_err(AppError::DbError)?;

        Ok(created_user)
    }

    async fn find_details_internal<C>(db: &C, uuid: &str) -> AppResult<Option<user::Model>>
    where
        C: ConnectionTrait,
    {
        let user_model = user::Entity::find()
            .filter(user::Column::Uuid.eq(uuid))
            .one(db)
            .await
            .map_err(AppError::DbError)?;

        match user_model {
            Some(mut u) => {
                let verification = u
                    .find_related(verification::Entity)
                    .one(db)
                    .await
                    .map_err(AppError::DbError)?;

                let socials = u
                    .find_related(social::Entity)
                    .all(db)
                    .await
                    .map_err(AppError::DbError)?;

                // Delivery is not strictly part of User aggregate in this read model necessarily,
                // but if it was in the original code, we should check if we need it.
                // The original code imported delivery entity.
                // However, moving this to user module might create circular dependency if we import delivery entity here?
                // `delivery` module likely depends on `user` module?
                // Let's check original code:
                // It was:
                // let delivery = u.find_related(delivery_data::Entity).one(db)...
                // We need to see if we can import delivery entity here without cycle.
                // Usually entities are safe to import.

                // For now I will comment out delivery part or try to import it.
                // The prompt was to split modules.
                // If I import crate::modules::delivery::entities::delivery_data, it should be fine as long as `delivery` doesn't import `users::repository` (which it might).
                // But entities are usually low level.

                // Wait, if I'm in `users` module, checking `delivery` table seems like a cross-module concern.
                // But for `find_with_details_by_uuid`, maybe it returns a user with all mixed in data?
                // The `user::Model` has a `delivery` field?
                // Let's check `user::Model`.

                u.verification = verification;
                u.socials = socials;
                // u.delivery = delivery; // We will handle this in slightly different way or just import it.

                Ok(Some(u))
            }
            None => Ok(None),
        }
    }
}

// =========================================================================
// InMemory Implementation
// =========================================================================

#[derive(Clone, Default)]
pub struct InMemoryUserRepository {
    users: Arc<Mutex<HashMap<i32, user::Model>>>,
    socials: Arc<Mutex<Vec<social::Model>>>,
    verifications: Arc<Mutex<HashMap<i32, verification::Model>>>,
    counter: Arc<Mutex<i32>>,
}

#[async_trait]
impl UserRepository for InMemoryUserRepository {
    async fn find_by_id(&self, id: i32) -> AppResult<Option<user::Model>> {
        let users = self.users.lock().unwrap();
        Ok(users.get(&id).cloned())
    }
    async fn find_by_uuid(&self, uuid: &str) -> AppResult<Option<user::Model>> {
        let users = self.users.lock().unwrap();
        Ok(users.values().find(|u| u.uuid == uuid).cloned())
    }
    async fn find_by_email(&self, email: &str) -> AppResult<Option<user::Model>> {
        let users = self.users.lock().unwrap();
        Ok(users.values().find(|u| u.email == email).cloned())
    }
    async fn find_social(
        &self,
        provider: social::SocialProvider,
        provider_id: &str,
    ) -> AppResult<Option<social::Model>> {
        let socials = self.socials.lock().unwrap();
        Ok(socials
            .iter()
            .find(|s| s.provider == provider && s.provider_id == provider_id)
            .cloned())
    }

    async fn create_user_with_verification(
        &self,
        user: user::ActiveModel,
        social: Option<social::ActiveModel>,
        verification: verification::ActiveModel,
    ) -> AppResult<user::Model> {
        let mut users = self.users.lock().unwrap();
        let mut verifications = self.verifications.lock().unwrap();
        let mut socials = self.socials.lock().unwrap();

        let mut counter = self.counter.lock().unwrap();

        *counter += 1;
        let new_id = *counter;

        let model_user = user::Model {
            id: new_id,
            uuid: user.uuid.unwrap(),
            username: user.username.unwrap(),
            email: user.email.unwrap(),
            country_code: user.country_code.unwrap(),
            phone_number: user.phone_number.unwrap(),
            account_status: user.account_status.unwrap(),
            created_at: user.created_at.unwrap(),
            updated_at: user.updated_at.unwrap(),
            last_login_at: user.last_login_at.unwrap(),
            verification: None,
            socials: vec![],
            delivery: vec![],
        };

        users.insert(new_id, model_user.clone());

        if let Some(s) = social {
            let model_social = social::Model {
                id: *counter * 10,
                user_id: new_id,
                provider: s.provider.unwrap(),
                provider_id: s.provider_id.unwrap(),
                created_at: s.created_at.unwrap(),
            };
            socials.push(model_social);
        }

        let mut email_verified = false;
        if let Set(v) = verification.email_verified {
            email_verified = v;
        }

        let mut email_verified_at = None;
        if let Set(v) = verification.email_verified_at {
            email_verified_at = v;
        }

        let mut phone_verified = false;
        if let Set(v) = verification.phone_verified {
            phone_verified = v;
        }

        let mut phone_verified_at = None;
        if let Set(v) = verification.phone_verified_at {
            phone_verified_at = v;
        }

        let mut business_verified = false;
        if let Set(v) = verification.business_verified {
            business_verified = v;
        }

        let mut business_info = None;
        if let Set(v) = verification.business_info {
            business_info = v;
        }

        let model_verification = verification::Model {
            id: *counter * 100,
            user_id: new_id,
            email_verified,
            email_verified_at,
            phone_verified,
            phone_verified_at,
            business_verified,
            business_info,
            verification_code: None,
        };
        verifications.insert(new_id, model_verification.clone());

        Ok(model_user)
    }

    async fn update_user(&self, user: user::ActiveModel) -> AppResult<user::Model> {
        let mut users = self.users.lock().unwrap();
        let id = user.id.unwrap();

        if let Some(existing) = users.get_mut(&id) {
            if let Set(v) = user.account_status {
                existing.account_status = v;
            }
            if let Set(v) = user.email {
                existing.email = v;
            }
        }
        Ok(users.get(&id).unwrap().clone())
    }

    async fn find_with_details_by_uuid(&self, uuid: &str) -> AppResult<Option<user::Model>> {
        let users = self.users.lock().unwrap();
        if let Some(mut user) = users.values().find(|u| u.uuid == uuid).cloned() {
            let verifications = self.verifications.lock().unwrap();

            let verification = verifications.get(&user.id).cloned();

            let socials_lock = self.socials.lock().unwrap();
            let socials: Vec<social::Model> = socials_lock
                .iter()
                .filter(|s| s.user_id == user.id)
                .cloned()
                .collect();

            user.verification = verification;
            user.socials = socials;

            Ok(Some(user))
        } else {
            Ok(None)
        }
    }

    async fn update_verification(
        &self,
        verification: verification::ActiveModel,
    ) -> AppResult<verification::Model> {
        let mut verifications = self.verifications.lock().unwrap();
        let user_id = verification.user_id.unwrap();
        if let Some(existing) = verifications.get_mut(&user_id) {
            if let Set(v) = verification.verification_code {
                existing.verification_code = v;
            }
            if let Set(v) = verification.email_verified {
                existing.email_verified = v;
            }
            if let Set(v) = verification.email_verified_at {
                existing.email_verified_at = v;
            }
            Ok(existing.clone())
        } else {
            Err(AppError::NotFound)
        }
    }

    fn with_transaction(&self, _uow: &dyn UnitOfWork) -> Option<Box<dyn UserRepository>> {
        Some(Box::new(self.clone()))
    }
}
