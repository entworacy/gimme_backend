use axum::{
    Json,
    extract::{Query, State},
    response::Redirect,
};

use serde::Deserialize;

use super::service::AuthService;
use crate::modules::users::entities::social::SocialProvider;
use crate::shared::{
    error::{AppError, AppResult},
    state::AppState,
};

#[derive(Deserialize)]
pub struct AuthCallbackParams {
    code: String,
}

pub async fn login_kakao(State(state): State<AppState>) -> AppResult<Redirect> {
    let kakao_provider =
        state
            .auth_registry
            .get(SocialProvider::Kakao)
            .ok_or(AppError::InternalServerError(
                "Kakao provider not configured".to_string(),
            ))?;

    let auth_url = kakao_provider.get_authorization_url();
    Ok(Redirect::to(&auth_url))
}

pub async fn callback_kakao(
    State(state): State<AppState>,
    Query(params): Query<AuthCallbackParams>,
) -> AppResult<Json<serde_json::Value>> {
    let kakao_provider =
        state
            .auth_registry
            .get(SocialProvider::Kakao)
            .ok_or(AppError::InternalServerError(
                "Kakao provider not configured".to_string(),
            ))?;

    // 1. Get User Info from Provider
    let user_info = kakao_provider.get_user_info(&params.code).await?;

    // 2. Login or Register
    let (token, need_more_action) = AuthService::handle_social_login(
        state.user_repo.as_ref(),
        &state.config,
        SocialProvider::Kakao,
        user_info,
    )
    .await?;

    // Return JWT (In real app, maybe set cookie or redirect to frontend with token)
    Ok(Json(serde_json::json!({
        "token": token,
        "token_type": "Bearer",
        "need_more_action": need_more_action,
    })))
}

#[derive(Deserialize)]
pub struct ValidateEmailRequest {
    pub email: String,
}

pub async fn request_email_verification(
    State(state): State<AppState>,
    claims: crate::modules::auth::service::Claims,
    Json(body): Json<ValidateEmailRequest>,
) -> AppResult<Json<serde_json::Value>> {
    tracing::info!("Hit request_email_verification for user_id: {}", claims.sub);

    // 1. Fetch User with Details
    let (user, verification, socials) = state
        .user_repo
        .find_with_details_by_uuid(&claims.sub)
        .await?
        .ok_or(AppError::NotFound)?;

    // 2. Validate User Status (Must be Pending)
    if user.account_status != crate::modules::users::entities::enums::AccountStatus::Pending {
        return Err(AppError::BadRequest(
            "User is already active or banned".to_string(),
        ));
    }

    // 3. Validate Email matches or provided (Requirements said: receive body `{"email": "..."}`)
    // AND Check if verification already done?
    // "relationship된 verification table의 이메일 인증 여부가 true일 경우에는 Bad Request"
    if let Some(v) = &verification {
        if v.email_verified {
            return Err(AppError::BadRequest("Email already verified".to_string()));
        }
    } else {
        // Should not happen if created correctly, but if missing, can't verify
        return Err(AppError::InternalServerError(
            "Verification record missing".to_string(),
        ));
    }

    // 5. Generate 6-digit code
    use rand::Rng;
    let code: u32 = rand::rng().random_range(100000..999999);
    let code_str = code.to_string();

    // 6. Update Verification
    let verification_model = verification.ok_or(AppError::InternalServerError(
        "Verification record missing".to_string(),
    ))?;
    let mut verification_active: crate::modules::users::entities::verification::ActiveModel =
        verification_model.into();
    verification_active.verification_code = sea_orm::ActiveValue::Set(Some(code_str.clone()));

    state
        .user_repo
        .update_verification(verification_active)
        .await?;

    // 7. Mock Send Email
    tracing::info!("Sending verification code {} to {}", code_str, body.email);

    Ok(Json(serde_json::json!({
        "message": "Verification code sent"
    })))
}

#[derive(Deserialize)]
pub struct VerifyEmailCodeRequest {
    pub email: String,
    pub code: String,
}

pub async fn verify_email_code(
    State(state): State<AppState>,
    claims: crate::modules::auth::service::Claims,
    Json(body): Json<VerifyEmailCodeRequest>,
) -> AppResult<Json<serde_json::Value>> {
    // 1. Fetch User with Details
    let (user, verification, _) = state
        .user_repo
        .find_with_details_by_uuid(&claims.sub)
        .await?
        .ok_or(AppError::NotFound)?; // Unauthorized?

    // 2. Verify Code
    if let Some(v) = &verification {
        // Double check already verified
        if v.email_verified {
            return Ok(Json(serde_json::json!({ "message": "Already verified" })));
        }

        match &v.verification_code {
            Some(stored_code) => {
                if stored_code != &body.code {
                    return Err(AppError::BadRequest(
                        "Invalid verification code".to_string(),
                    ));
                }
            }
            None => {
                return Err(AppError::BadRequest(
                    "No verification code found (or expired)".to_string(),
                ));
            }
        }
    } else {
        return Err(AppError::InternalServerError(
            "Verification record missing".to_string(),
        ));
    }

    // 3. Update Verification (Verified = true, Clear Code)
    let verification_model = verification.ok_or(AppError::InternalServerError(
        "Verification record missing".to_string(),
    ))?;
    let mut verification_active: crate::modules::users::entities::verification::ActiveModel =
        verification_model.into();
    let mut user_active: crate::modules::users::entities::user::ActiveModel = user.into();
    user_active.account_status =
        sea_orm::ActiveValue::Set(crate::modules::users::entities::enums::AccountStatus::Active);
    verification_active.verification_code = sea_orm::ActiveValue::Set(None); // Clear code
    verification_active.email_verified = sea_orm::ActiveValue::Set(true);
    verification_active.email_verified_at =
        sea_orm::ActiveValue::Set(Some(chrono::Utc::now().naive_utc()));

    state
        .user_repo
        .update_verification(verification_active)
        .await?;
    state.user_repo.update_user(user_active).await?;

    // 5. Success
    Ok(Json(serde_json::json!({
        "message": "Email verified successfully"
    })))
}
