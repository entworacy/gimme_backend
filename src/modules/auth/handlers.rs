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
