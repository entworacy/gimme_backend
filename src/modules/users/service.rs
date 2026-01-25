use crate::modules::auth::providers::OAuthUserInfo;
use crate::modules::users::entities::{
    social::{self, SocialProvider},
    user,
};
use crate::shared::error::{AppError, AppResult};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set,
    TransactionTrait,
};
use std::sync::Arc;

pub struct UserService;

impl UserService {
    pub async fn find_or_create_social_user(
        db: &DatabaseConnection,
        provider: SocialProvider,
        user_info: OAuthUserInfo,
    ) -> AppResult<user::Model> {
        // 1. Check if Social Account exists
        let social_account = social::Entity::find()
            .filter(social::Column::Provider.eq(provider.clone()))
            .filter(social::Column::ProviderId.eq(user_info.provider_id.clone()))
            .one(db)
            .await
            .map_err(AppError::DbError)?;

        if let Some(social) = social_account {
            // User exists, return user
            return user::Entity::find_by_id(social.user_id)
                .one(db)
                .await
                .map_err(AppError::DbError)?
                .ok_or(AppError::InternalServerError(
                    "User not found for social account".to_string(),
                ));
        }

        // 2. Create new User and Social Account
        Self::register_social_user(db, provider, user_info).await
    }

    async fn register_social_user(
        db: &DatabaseConnection,
        provider: SocialProvider,
        user_info: OAuthUserInfo,
    ) -> AppResult<user::Model> {
        let txn = db.begin().await.map_err(AppError::DbError)?;

        let now = chrono::Utc::now().naive_utc();
        let uuid = uuid::Uuid::new_v4().as_u128().to_string();

        // Create User
        let new_user = user::ActiveModel {
            uuid: Set(uuid),
            username: Set(user_info.name.unwrap_or_else(|| "User".to_string())),
            email: Set(user_info.email.unwrap_or_else(|| "".to_string())),
            country_code: Set("".to_string()),
            phone_number: Set("".to_string()),
            account_status: Set(crate::modules::users::entities::enums::AccountStatus::Active),
            created_at: Set(now),
            updated_at: Set(now),
            last_login_at: Set(Some(now)),
            ..Default::default()
        };

        let user = new_user.insert(&txn).await.map_err(AppError::DbError)?;

        // Create Social Record
        let new_social = social::ActiveModel {
            user_id: Set(user.id),
            provider: Set(provider),
            provider_id: Set(user_info.provider_id),
            created_at: Set(now),
            ..Default::default()
        };
        new_social.insert(&txn).await.map_err(AppError::DbError)?;

        // Create blank verification record
        let verification = crate::modules::users::entities::verification::ActiveModel {
            user_id: Set(user.id),
            email_verified: Set(true),
            phone_verified: Set(false),
            business_verified: Set(false),
            ..Default::default()
        };
        verification.insert(&txn).await.map_err(AppError::DbError)?;

        txn.commit().await.map_err(AppError::DbError)?;

        Ok(user)
    }
}
