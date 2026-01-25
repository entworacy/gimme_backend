use axum::{
    routing::{get, post},
    Router,
};
use std::sync::Arc;
use sea_orm::DatabaseConnection;
use crate::modules::users::handlers;

pub fn router(db: Arc<DatabaseConnection>) -> Router {
    Router::new()
        .route("/", post(handlers::create_user))
        .route("/:id", get(handlers::get_user))
        .with_state(db)
}
