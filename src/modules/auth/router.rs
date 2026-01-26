use super::handlers;
use crate::shared::state::AppState;
use axum::{Router, routing::get};

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/login/kakao", get(handlers::login_kakao))
        .route("/callback/kakao", get(handlers::callback_kakao))
        .route(
            "/validate-email",
            axum::routing::post(handlers::request_email_verification),
        )
        .route(
            "/validate-email-code",
            axum::routing::post(handlers::verify_email_code),
        )
        .with_state(state)
}
