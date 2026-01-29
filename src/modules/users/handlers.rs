use axum::{
    Json,
    extract::{Path, State},
};
use serde::Serialize;

use crate::modules::users::entities::enums::AccountStatus;
use crate::modules::users::entities::user;
use crate::modules::users::repository::UserRepository;
use crate::shared::{
    error::{AppError, AppResult},
    state::AppState,
};
use std::sync::Arc;
#[derive(Serialize)]
pub struct UserResponse {
    #[serde(skip_serializing)]
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
    pub verification: Option<UserVerificationResponse>,
    pub social_accounts: Vec<UserSocialResponse>,
}

#[derive(Serialize)]
pub struct UserVerificationResponse {
    pub email_verified: bool,
    pub email_verified_at: Option<chrono::NaiveDateTime>,
    pub phone_verified: bool,
    pub phone_verified_at: Option<chrono::NaiveDateTime>,
    pub business_verified: bool,
    pub business_info: Option<String>,
}

#[derive(Serialize)]
pub struct UserSocialResponse {
    pub provider: String,
    pub provider_id: String,
    pub created_at: chrono::NaiveDateTime,
}

pub async fn get_user(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> AppResult<Json<UserResponse>> {
    let user_repo = state.repo_manager.get::<Arc<dyn UserRepository>>().ok_or(
        AppError::InternalServerError("UserRepository not registered".to_string()),
    )?;

    let user: user::Model = user_repo.find_by_id(id).await?.ok_or(AppError::NotFound)?;

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
        verification: None, // TODO: Fetch verification if needed for public profile or admin
        social_accounts: vec![], // TODO: Fetch socials if needed
    }))
}

pub async fn get_me(
    State(state): State<AppState>,
    claims: crate::modules::auth::service::Claims,
) -> AppResult<Json<UserResponse>> {
    let user_repo = state.repo_manager.get::<Arc<dyn UserRepository>>().ok_or(
        AppError::InternalServerError("UserRepository not registered".to_string()),
    )?;

    let user = user_repo
        .find_with_details_by_uuid(&claims.sub)
        .await?
        .ok_or(AppError::NotFound)?;

    let verification_response = user
        .verification
        .as_ref()
        .map(|v| UserVerificationResponse {
            email_verified: v.email_verified,
            email_verified_at: v.email_verified_at,
            phone_verified: v.phone_verified,
            phone_verified_at: v.phone_verified_at,
            business_verified: v.business_verified,
            business_info: v.business_info.clone(),
        });

    let social_responses = user
        .socials
        .iter()
        .map(|s| UserSocialResponse {
            provider: format!("{:?}", s.provider),
            provider_id: s.provider_id.clone(),
            created_at: s.created_at,
        })
        .collect();

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
        verification: verification_response,
        social_accounts: social_responses,
    }))
}
