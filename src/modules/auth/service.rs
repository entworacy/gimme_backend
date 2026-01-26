use chrono::{Duration, Utc};
use jsonwebtoken::{EncodingKey, Header, encode};

use serde::{Deserialize, Serialize};

use crate::modules::users::entities::enums::AccountStatus;
use crate::modules::users::repository::UserRepository;

use super::providers::OAuthUserInfo;
use crate::modules::users::{
    dtos::SocialLoginDto, entities::social::SocialProvider, service::UserService,
};
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
        repo: &dyn UserRepository,
        config: &Config,
        provider: SocialProvider,
        user_info: OAuthUserInfo,
    ) -> AppResult<(String, bool)> {
        let login_dto = SocialLoginDto {
            provider,
            provider_id: user_info.provider_id,
            email: user_info.email,
            name: user_info.name,
            phone_number: user_info.phone_number,
            connected_at: user_info.connected_at,
        };

        // Delegate finding/creating user to Domain Service
        let user = UserService::handle_social_login(repo, login_dto).await?;
        let need_more_action = !(user.account_status == AccountStatus::Active);
        // Generate JWT
        let token = Self::generate_jwt(config, &user.uuid)?;
        Ok((token, need_more_action))
    }

    fn generate_jwt(_config: &Config, user_uuid: &str) -> AppResult<String> {
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
