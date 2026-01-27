use axum::{
    extract::{Request, State},
    middleware::Next,
    response::Response,
};

use crate::modules::auth::service::Claims;
use crate::modules::users::repository::UserRepository;
use crate::shared::{
    error::{AppError, AppResult},
    state::AppState,
};
use std::sync::Arc;

pub async fn require_email_verified(
    State(state): State<AppState>,
    claims: Claims,
    request: Request,
    next: Next,
) -> AppResult<Response> {
    let user_repo = state.repo_manager.get::<Arc<dyn UserRepository>>().ok_or(
        AppError::InternalServerError("UserRepository not registered".to_string()),
    )?;

    let (_, verification, _) = user_repo
        .find_with_details_by_uuid(&claims.sub)
        .await?
        .ok_or(AppError::NotFound)?; // User not found implies invalid token effectively here

    if let Some(v) = verification {
        if v.email_verified {
            return Ok(next.run(request).await);
        }
    }

    Err(AppError::Forbidden("Email not verified".to_string()))
}
