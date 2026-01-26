use super::handlers;
use crate::shared::state::AppState;
use axum::{Router, routing::get};

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/login/kakao", get(handlers::login_kakao))
        .route("/callback/kakao", get(handlers::callback_kakao))
        .with_state(state)
}
