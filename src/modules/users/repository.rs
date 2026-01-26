use async_trait::async_trait;
use sea_orm::*;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use super::entities::{social, user, verification};
use crate::shared::error::{AppError, AppResult};

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
}

// Postgres Implementation
pub struct PostgresUserRepository {
    db: Arc<DatabaseConnection>,
}

impl PostgresUserRepository {
    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        Self { db }
    }
}

#[async_trait]
impl UserRepository for PostgresUserRepository {
    async fn find_by_id(&self, id: i32) -> AppResult<Option<user::Model>> {
        user::Entity::find_by_id(id)
            .one(self.db.as_ref())
            .await
            .map_err(AppError::DbError)
    }

    async fn find_by_uuid(&self, uuid: &str) -> AppResult<Option<user::Model>> {
        user::Entity::find()
            .filter(user::Column::Uuid.eq(uuid))
            .one(self.db.as_ref())
            .await
            .map_err(AppError::DbError)
    }

    async fn find_by_email(&self, email: &str) -> AppResult<Option<user::Model>> {
        user::Entity::find()
            .filter(user::Column::Email.eq(email))
            .one(self.db.as_ref())
            .await
            .map_err(AppError::DbError)
    }

    async fn find_social(
        &self,
        provider: social::SocialProvider,
        provider_id: &str,
    ) -> AppResult<Option<social::Model>> {
        social::Entity::find()
            .filter(social::Column::Provider.eq(provider))
            .filter(social::Column::ProviderId.eq(provider_id))
            .one(self.db.as_ref())
            .await
            .map_err(AppError::DbError)
    }

    async fn create_user_with_verification(
        &self,
        user: user::ActiveModel,
        social: Option<social::ActiveModel>,
        verification: verification::ActiveModel,
    ) -> AppResult<user::Model> {
        let txn = self.db.begin().await.map_err(AppError::DbError)?;

        let created_user = user.insert(&txn).await.map_err(AppError::DbError)?;

        if let Some(mut s) = social {
            s.user_id = Set(created_user.id);
            s.insert(&txn).await.map_err(AppError::DbError)?;
        }

        let mut v = verification;
        v.user_id = Set(created_user.id);
        v.insert(&txn).await.map_err(AppError::DbError)?;

        txn.commit().await.map_err(AppError::DbError)?;

        Ok(created_user)
    }

    async fn update_user(&self, user: user::ActiveModel) -> AppResult<user::Model> {
        user.update(self.db.as_ref())
            .await
            .map_err(AppError::DbError)
    }

    async fn find_with_details_by_uuid(
        &self,
        uuid: &str,
    ) -> AppResult<Option<(user::Model, Option<verification::Model>, Vec<social::Model>)>> {
        let user_model = user::Entity::find()
            .filter(user::Column::Uuid.eq(uuid))
            .one(self.db.as_ref())
            .await
            .map_err(AppError::DbError)?;

        match user_model {
            Some(u) => {
                let verification = u
                    .find_related(verification::Entity)
                    .one(self.db.as_ref())
                    .await
                    .map_err(AppError::DbError)?;

                let socials = u
                    .find_related(social::Entity)
                    .all(self.db.as_ref())
                    .await
                    .map_err(AppError::DbError)?;

                Ok(Some((u, verification, socials)))
            }
            None => Ok(None),
        }
    }
}

// In-Memory Implementation
#[derive(Default, Clone)]
pub struct InMemoryUserRepository {
    users: Arc<Mutex<HashMap<i32, user::Model>>>,
    socials: Arc<Mutex<Vec<social::Model>>>,
    verifications: Arc<Mutex<HashMap<i32, verification::Model>>>,
    counter: Arc<Mutex<i32>>,
}

impl InMemoryUserRepository {
    pub fn new() -> Self {
        Self::default()
    }
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

        // Simulate insertion (converting ActiveModel to Model roughly)
        // This is a bit manual because ActiveModel doesn't directly convert to Model without DB
        // But we can approximate for Mock
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
        };

        users.insert(new_id, model_user.clone());

        if let Some(s) = social {
            let model_social = social::Model {
                id: *counter * 10, // dummy logic
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
            id: *counter * 100, // dummy logic
            user_id: new_id,
            email_verified,
            email_verified_at,
            phone_verified,
            phone_verified_at,
            business_verified,
            business_info,
        };
        verifications.insert(new_id, model_verification);

        Ok(model_user)
    }

    async fn update_user(&self, user: user::ActiveModel) -> AppResult<user::Model> {
        let mut users = self.users.lock().unwrap();
        let id = user.id.unwrap(); // Must exist

        if let Some(existing) = users.get_mut(&id) {
            if let Set(v) = user.account_status {
                existing.account_status = v;
            }
            // ... map other fields if needed for full mock correctness
        }

        Ok(users.get(&id).unwrap().clone())
    }

    async fn find_with_details_by_uuid(
        &self,
        uuid: &str,
    ) -> AppResult<Option<(user::Model, Option<verification::Model>, Vec<social::Model>)>> {
        let users = self.users.lock().unwrap();
        if let Some(user) = users.values().find(|u| u.uuid == uuid).cloned() {
            let verifications = self.verifications.lock().unwrap();
            let verification = verifications.get(&user.id).cloned();

            let socials_lock = self.socials.lock().unwrap();
            let socials = socials_lock
                .iter()
                .filter(|s| s.user_id == user.id)
                .cloned()
                .collect();

            Ok(Some((user, verification, socials)))
        } else {
            Ok(None)
        }
    }
}
