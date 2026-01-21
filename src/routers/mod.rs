use axum::{
    routing::{get, post},
    Router,
};
use sea_orm::DatabaseConnection;
use std::sync::Arc;

use crate::handlers::user;

pub fn init_router(db: DatabaseConnection) -> Router {
    let db = Arc::new(db);
    Router::new()
        .route("/health", get(|| async { "OK" }))
        .route("/users", post(user::create_user))
        .route("/users/:id", get(user::get_user))
        .with_state(db)
}
