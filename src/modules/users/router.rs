use crate::shared::state::AppState;
use axum::Router;

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/me", axum::routing::get(super::handlers::get_me))
        .with_state(state)
}
