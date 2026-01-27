use axum::{
    extract::{Request, State},
    middleware::Next,
    response::Response,
};

use crate::modules::auth::service::Claims;
use crate::shared::{
    error::{AppError, AppResult},
    state::AppState,
};

pub async fn require_email_verified(
    State(state): State<AppState>,
    claims: Claims,
    request: Request,
    next: Next,
) -> AppResult<Response> {
    let (_, verification, _) = state
        .user_repo
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
