use crate::shared::{middleware::require_email_verified, state::AppState};
use axum::{Router, middleware};

pub fn router(state: AppState) -> Router {
    Router::new()
        .route(
            "/me",
            axum::routing::get(super::handlers::get_me).route_layer(
                middleware::from_fn_with_state(state.clone(), require_email_verified),
            ),
        )
        .with_state(state)
}
