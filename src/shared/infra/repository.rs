use async_trait::async_trait;
use sea_orm::*;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::sync::Mutex as AsyncMutex;

use crate::modules::users::entities::{social, user, verification};
use crate::modules::users::repository::UserRepository;
use crate::shared::error::{AppError, AppResult};
use crate::shared::repository::{RepositoryManager, UnitOfWork};

// =========================================================================
// Postgres Implementation
// =========================================================================

#[derive(Clone)]
enum DbOrTxn {
    Conn(Arc<DatabaseConnection>),
    Txn(Arc<AsyncMutex<Option<DatabaseTransaction>>>),
}

#[derive(Clone)]
pub struct PostgresUserRepository {
    conn: DbOrTxn,
}

impl PostgresUserRepository {
    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        Self {
            conn: DbOrTxn::Conn(db),
        }
    }
}

// Helper to execute query based on connection type

pub struct PostgresRepositoryManager {
    db: Arc<DatabaseConnection>,
}

impl PostgresRepositoryManager {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db: Arc::new(db) }
    }
}

#[async_trait]
impl RepositoryManager for PostgresRepositoryManager {
    fn user_repo(&self) -> Box<dyn UserRepository> {
        Box::new(PostgresUserRepository::new(self.db.clone()))
    }

    async fn begin(&self) -> AppResult<Box<dyn UnitOfWork>> {
        let txn = self.db.begin().await.map_err(AppError::DbError)?;
        Ok(Box::new(PostgresUnitOfWork {
            txn: Arc::new(AsyncMutex::new(Some(txn))),
        }))
    }
}

pub struct PostgresUnitOfWork {
    txn: Arc<AsyncMutex<Option<DatabaseTransaction>>>,
}

#[async_trait]
impl UnitOfWork for PostgresUnitOfWork {
    fn user_repo(&self) -> Box<dyn UserRepository> {
        Box::new(PostgresUserRepository {
            conn: DbOrTxn::Txn(self.txn.clone()),
        })
    }

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

#[async_trait]
impl UserRepository for PostgresUserRepository {
    async fn find_by_id(&self, id: i32) -> AppResult<Option<user::Model>> {
        match &self.conn {
            DbOrTxn::Conn(c) => user::Entity::find_by_id(id)
                .one(c.as_ref())
                .await
                .map_err(AppError::DbError),
            DbOrTxn::Txn(mutex) => {
                let lock = mutex.lock().await;
                let txn = lock.as_ref().ok_or(AppError::InternalServerError(
                    "Transaction finished".to_string(),
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
        // Warning: This method contains internal transaction logic which might conflict if called inside a transaction.
        // However, sea-orm supports nested transactions (savepoints).
        // Since we are moving to UoW, this complex method might be better split in the service layer,
        // BUT for compatibility we keep it. Ideally we refactor this to use UoW in service too.
        // For now, let's implement it using whatever connection we have.

        // ISSUE: If self.conn is Txn, calling `begin()` creates a SAVEPOINT.
        // If self.conn is Conn, calling `begin()` creates a Transaction.
        // We can just use the connection to execute directly if we assume the caller handles atomicity via UoW?
        // OR we just use the Savepoint feature to keep it atomic internally.

        match &self.conn {
            DbOrTxn::Conn(c) => {
                let txn = c.begin().await.map_err(AppError::DbError)?;
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
                // Use savepoint? Or just reuse transaction?
                // If we reuse, and it fails, the whole txn might be tainted depending on DB.
                // Let's just execute on the txn.
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

    async fn find_with_details_by_uuid(
        &self,
        uuid: &str,
    ) -> AppResult<Option<(user::Model, Option<verification::Model>, Vec<social::Model>)>> {
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

    async fn begin_txn(
        &self,
    ) -> AppResult<Box<dyn crate::modules::users::repository::TxUserRepository>> {
        // Deprecated / Not supported in the new UoW pattern when using this struct directly from UoW context.
        // But if called from RepositoryManager context...
        // Actually, we are adhering to the trait signature from `modules/users/repository.rs`.
        // We should implement it for compatibility or error out.
        Err(AppError::InternalServerError(
            "Use RepositoryManager::begin() instead".to_string(),
        ))
    }
}

// Internal helpers
impl PostgresUserRepository {
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

    async fn find_details_internal<C>(
        db: &C,
        uuid: &str,
    ) -> AppResult<Option<(user::Model, Option<verification::Model>, Vec<social::Model>)>>
    where
        C: ConnectionTrait,
    {
        let user_model = user::Entity::find()
            .filter(user::Column::Uuid.eq(uuid))
            .one(db)
            .await
            .map_err(AppError::DbError)?;

        match user_model {
            Some(u) => {
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

                Ok(Some((u, verification, socials)))
            }
            None => Ok(None),
        }
    }
}

// =========================================================================
// InMemory Implementation
// =========================================================================

#[derive(Default, Clone)]
pub struct InMemoryRepositoryManager {
    // Shared state
    users: Arc<Mutex<HashMap<i32, user::Model>>>,
    socials: Arc<Mutex<Vec<social::Model>>>,
    verifications: Arc<Mutex<HashMap<i32, verification::Model>>>,
    counter: Arc<Mutex<i32>>,
}

impl InMemoryRepositoryManager {
    pub fn new() -> Self {
        Self::default()
    }
}

pub struct InMemoryUnitOfWork {
    repo: InMemoryUserRepository,
}

#[derive(Clone)]
pub struct InMemoryUserRepository {
    users: Arc<Mutex<HashMap<i32, user::Model>>>,
    socials: Arc<Mutex<Vec<social::Model>>>,
    verifications: Arc<Mutex<HashMap<i32, verification::Model>>>,
    counter: Arc<Mutex<i32>>,
}

#[async_trait]
impl RepositoryManager for InMemoryRepositoryManager {
    fn user_repo(&self) -> Box<dyn UserRepository> {
        Box::new(InMemoryUserRepository {
            users: self.users.clone(),
            socials: self.socials.clone(),
            verifications: self.verifications.clone(),
            counter: self.counter.clone(),
        })
    }

    async fn begin(&self) -> AppResult<Box<dyn UnitOfWork>> {
        Ok(Box::new(InMemoryUnitOfWork {
            repo: InMemoryUserRepository {
                users: self.users.clone(),
                socials: self.socials.clone(),
                verifications: self.verifications.clone(),
                counter: self.counter.clone(),
            },
        }))
    }
}

#[async_trait]
impl UnitOfWork for InMemoryUnitOfWork {
    fn user_repo(&self) -> Box<dyn UserRepository> {
        Box::new(self.repo.clone())
    }
    async fn commit(self: Box<Self>) -> AppResult<()> {
        Ok(())
    }
    async fn rollback(self: Box<Self>) -> AppResult<()> {
        Ok(())
    }
}

#[async_trait]
impl UserRepository for InMemoryUserRepository {
    // Implement using logic from modules/users/repository.rs
    // For brevity, assuming direct copying or importing logic is ideal.
    // I will inline simplified logic for now as I can't import private things easily if they were private.
    // Logic is identical to previous InMemoryUserRepository.

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

    async fn begin_txn(
        &self,
    ) -> AppResult<Box<dyn crate::modules::users::repository::TxUserRepository>> {
        // Return dummy txn repo or handle error
        // Compatible with old trait
        Ok(Box::new(self.clone()))
    }
}
// Old trait compat
#[async_trait]
impl crate::modules::users::repository::TxUserRepository for InMemoryUserRepository {
    async fn commit(self: Box<Self>) -> AppResult<()> {
        Ok(())
    }
    async fn rollback(self: Box<Self>) -> AppResult<()> {
        Ok(())
    }
}
