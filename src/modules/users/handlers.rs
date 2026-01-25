use axum::{
    Json,
    extract::{Path, Query, State},
    response::Redirect,
};
use sea_orm::{ActiveModelTrait, DatabaseConnection, EntityTrait, Set, TransactionTrait};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::modules::auth::{registry::OAuthProviderRegistry, service::AuthService};
use crate::modules::users::entities::{
    enums::AccountStatus, social::SocialProvider, user, verification,
};
use crate::shared::{
    config::Config,
    error::{AppError, AppResult},
};

#[derive(Deserialize)]
pub struct OAuthCallbackParams {
    pub code: String,
}

#[derive(Deserialize)]
pub struct CreateUserRequest {
    pub username: String,
    pub email: String,
    pub country_code: String,
    pub phone_number: String,
}

#[derive(Serialize)]
pub struct UserResponse {
    pub id: i32,
    pub uuid: String,
    pub username: String,
    pub email: String,
    pub country_code: String,
    pub phone_number: String,
    pub account_status: AccountStatus,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
    pub last_login_at: Option<chrono::NaiveDateTime>,
}

pub async fn create_user(
    State(db): State<Arc<DatabaseConnection>>,
    Json(payload): Json<CreateUserRequest>,
) -> AppResult<Json<UserResponse>> {
    // Generate UUID v4
    let uuid = uuid::Uuid::new_v4();
    // Convert to u128
    let uuid_u128 = uuid.as_u128();
    // Convert to decimal string
    let uuid_str = uuid_u128.to_string();

    let now = chrono::Utc::now().naive_utc();

    let new_user = user::ActiveModel {
        uuid: Set(uuid_str),
        username: Set(payload.username),
        email: Set(payload.email),
        country_code: Set(payload.country_code),
        phone_number: Set(payload.phone_number),
        account_status: Set(AccountStatus::Pending),
        created_at: Set(now),
        updated_at: Set(now),
        last_login_at: Set(None),
        ..Default::default()
    };

    // Transaction to insert user and verification
    let txn = db
        .begin()
        .await
        .map_err(|e| AppError::InternalServerError(e.to_string()))?;

    let user = new_user
        .insert(&txn)
        .await
        .map_err(|e| AppError::InternalServerError(e.to_string()))?;

    // Create verification entry
    let verification = verification::ActiveModel {
        user_id: Set(user.id),
        email_verified: Set(false),
        phone_verified: Set(false),
        business_verified: Set(false),
        ..Default::default()
    };
    verification
        .insert(&txn)
        .await
        .map_err(|e| AppError::InternalServerError(e.to_string()))?;

    txn.commit()
        .await
        .map_err(|e| AppError::InternalServerError(e.to_string()))?;

    Ok(Json(UserResponse {
        id: user.id,
        uuid: user.uuid,
        username: user.username,
        email: user.email,
        country_code: user.country_code,
        phone_number: user.phone_number,
        account_status: user.account_status,
        created_at: user.created_at,
        updated_at: user.updated_at,
        last_login_at: user.last_login_at,
    }))
}

pub async fn get_user(
    State(db): State<Arc<DatabaseConnection>>,
    Path(id): Path<i32>,
) -> AppResult<Json<UserResponse>> {
    let user = user::Entity::find_by_id(id)
        .one(db.as_ref())
        .await?
        .ok_or(AppError::NotFound)?;

    Ok(Json(UserResponse {
        id: user.id,
        uuid: user.uuid,
        username: user.username,
        email: user.email,
        country_code: user.country_code,
        phone_number: user.phone_number,
        account_status: user.account_status,
        created_at: user.created_at,
        updated_at: user.updated_at,
        last_login_at: user.last_login_at,
    }))
}

#[derive(Serialize)]
pub struct OAuthLoginResponse {
    pub token: String,
    pub token_type: String,
}

pub async fn login_oauth(
    Path(provider): Path<String>,
    State(auth_registry): State<OAuthProviderRegistry>,
) -> AppResult<Redirect> {
    let provider_enum = match provider.to_lowercase().as_str() {
        "kakao" => SocialProvider::Kakao,
        "google" => SocialProvider::Google,
        "apple" => SocialProvider::Apple,
        _ => return Err(AppError::BadRequest("Invalid provider".to_string())),
    };

    let oauth_provider = auth_registry
        .get(provider_enum)
        .ok_or(AppError::InternalServerError(
            "Provider not configured".to_string(),
        ))?;

    let auth_url = oauth_provider.get_authorization_url();
    Ok(Redirect::to(&auth_url))
}

pub async fn callback_oauth(
    State(db): State<Arc<DatabaseConnection>>,
    State(config): State<Arc<Config>>,
    State(auth_registry): State<OAuthProviderRegistry>,
    Path(provider): Path<String>,
    Query(params): Query<OAuthCallbackParams>,
) -> AppResult<Json<OAuthLoginResponse>> {
    let provider_enum = match provider.to_lowercase().as_str() {
        "kakao" => SocialProvider::Kakao,
        "google" => SocialProvider::Google,
        "apple" => SocialProvider::Apple,
        _ => return Err(AppError::BadRequest("Invalid provider".to_string())),
    };

    let oauth_provider =
        auth_registry
            .get(provider_enum.clone())
            .ok_or(AppError::InternalServerError(
                "Provider not configured".to_string(),
            ))?;

    // 1. Get User Info from Provider
    let user_info = oauth_provider.get_user_info(&params.code).await?;

    // 2. Login or Register
    let token = AuthService::handle_social_login(&db, &config, provider_enum, user_info).await?;

    Ok(Json(OAuthLoginResponse {
        token,
        token_type: "Bearer".to_string(),
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use sea_orm::{DatabaseBackend, MockDatabase};

    #[tokio::test]
    async fn test_create_user_success() {
        // Setup Mock DB
        let now = chrono::Utc::now().naive_utc();

        let db = MockDatabase::new(DatabaseBackend::Postgres)
            .append_query_results(vec![vec![user::Model {
                id: 1,
                uuid: "340282366920938463463374607431768211455".to_owned(),
                username: "testuser".to_owned(),
                email: "test@example.com".to_owned(),
                country_code: "82".to_owned(),
                phone_number: "01012345678".to_owned(),
                account_status: AccountStatus::Pending,
                created_at: now,
                updated_at: now,
                last_login_at: None,
            }]])
            .append_query_results(vec![vec![verification::Model {
                id: 1,
                user_id: 1,
                email_verified: false,
                email_verified_at: None,
                phone_verified: false,
                phone_verified_at: None,
                business_verified: false,
                business_info: None,
            }]])
            .into_connection();

        let db = Arc::new(db);

        // Create request
        let request = CreateUserRequest {
            username: "testuser".to_string(),
            email: "test@example.com".to_string(),
            country_code: "82".to_string(),
            phone_number: "01012345678".to_string(),
        };

        // Execute handler
        let response = create_user(State(db), Json(request)).await;

        // Verify
        assert!(response.is_ok());
        let user_response = response.unwrap().0;
        // Initialize tracing for test (if not already acting)
        let _ = tracing_subscriber::fmt()
            .with_test_writer()
            .with_max_level(tracing::Level::INFO)
            .try_init();

        tracing::info!("User Name: {}", user_response.username);
        tracing::info!("User Email: {}", user_response.email);
        tracing::info!("User Country Code: {}", user_response.country_code);
        tracing::info!("User Phone Number: {}", user_response.phone_number);
        tracing::info!("User ID: {}", user_response.id);
        assert_eq!(user_response.username, "testuser");
        assert_eq!(user_response.email, "test@example.com");
        assert_eq!(user_response.country_code, "82");
        assert_eq!(user_response.phone_number, "01012345678");
        assert_eq!(user_response.id, 1);
        // Verify UUID is a number string
        assert!(user_response.uuid.parse::<u128>().is_ok());
    }
}
