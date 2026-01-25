use chrono::{Duration, Utc};
use jsonwebtoken::{EncodingKey, Header, encode};
use sea_orm::DatabaseConnection;
use serde::{Deserialize, Serialize};

use super::providers::OAuthUserInfo;
use crate::modules::users::{entities::social::SocialProvider, service::UserService};
use crate::shared::config::Config;
use crate::shared::error::{AppError, AppResult};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String, // User UUID
    pub exp: usize,
    pub iat: usize,
}

pub struct AuthService;

impl AuthService {
    pub async fn handle_social_login(
        db: &DatabaseConnection,
        config: &Config,
        provider: SocialProvider,
        user_info: OAuthUserInfo,
    ) -> AppResult<String> {
        // Delegate finding/creating user to Domain Service
        let user = UserService::find_or_create_social_user(db, provider, user_info).await?;

        // Generate JWT
        Self::generate_jwt(config, &user.uuid)
    }

    fn generate_jwt(config: &Config, user_uuid: &str) -> AppResult<String> {
        // Use a secret from config
        // TODO: Add JWT_SECRET to Config
        let secret = "secret_key_change_me".as_bytes();

        let expiration = Utc::now()
            .checked_add_signed(Duration::hours(24))
            .expect("valid timestamp")
            .timestamp() as usize;

        let claims = Claims {
            sub: user_uuid.to_string(),
            exp: expiration,
            iat: Utc::now().timestamp() as usize,
        };

        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(secret),
        )
        .map_err(|e| AppError::InternalServerError(format!("JWT generation failed: {}", e)))
    }
}
