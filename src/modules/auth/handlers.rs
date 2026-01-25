use axum::{
    Json,
    extract::{Query, State},
    response::{IntoResponse, Redirect},
};
use sea_orm::DatabaseConnection;
use serde::Deserialize;
use std::sync::Arc;

use super::{providers::OAuthProvider, registry::OAuthProviderRegistry, service::AuthService};
use crate::modules::users::entities::social::SocialProvider;
use crate::shared::{
    config::Config,
    error::{AppError, AppResult},
};

#[derive(Deserialize)]
pub struct AuthCallbackParams {
    code: String,
}

pub async fn login_kakao(
    State(auth_registry): State<OAuthProviderRegistry>,
) -> AppResult<Redirect> {
    let kakao_provider =
        auth_registry
            .get(SocialProvider::Kakao)
            .ok_or(AppError::InternalServerError(
                "Kakao provider not configured".to_string(),
            ))?;

    let auth_url = kakao_provider.get_authorization_url();
    Ok(Redirect::to(&auth_url))
}

pub async fn callback_kakao(
    State(db): State<Arc<DatabaseConnection>>,
    State(config): State<Arc<Config>>,
    State(auth_registry): State<OAuthProviderRegistry>,
    Query(params): Query<AuthCallbackParams>,
) -> AppResult<Json<serde_json::Value>> {
    let kakao_provider =
        auth_registry
            .get(SocialProvider::Kakao)
            .ok_or(AppError::InternalServerError(
                "Kakao provider not configured".to_string(),
            ))?;

    // 1. Get User Info from Provider
    let user_info = kakao_provider.get_user_info(&params.code).await?;

    // 2. Login or Register
    let token =
        AuthService::handle_social_login(&db, &config, SocialProvider::Kakao, user_info).await?;

    // Return JWT (In real app, maybe set cookie or redirect to frontend with token)
    Ok(Json(serde_json::json!({
        "token": token,
        "token_type": "Bearer"
    })))
}
