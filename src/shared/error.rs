use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde_json::json;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Database error: {0}")]
    DbError(#[from] sea_orm::DbErr),

    #[error("Not found")]
    NotFound,

    #[error("Internal server error: {0}")]
    InternalServerError(String),

    #[error("Bad request: {0}")]
    BadRequest(String),

    #[error("Conflict: {0}")]
    Conflict(String),

    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    #[error("Forbidden: {0}")]
    Forbidden(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message, pp, pa) = match self {
            AppError::DbError(err) => {
                tracing::error!("Database error: {:?}", err);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Database error".to_string(),
                    "".to_string(),
                    "INTERNAL_SERVER_ERROR",
                )
            }
            AppError::NotFound => (
                StatusCode::NOT_FOUND,
                "Not found".to_string(),
                "404".to_string(),
                "APP_UPDATE_REQUIRED",
            ),
            AppError::InternalServerError(msg) => {
                tracing::error!("Internal server error: {}", msg);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Internal server error".to_string(),
                    msg,
                    "INTERNAL_SERVER_ERROR",
                )
            }
            AppError::BadRequest(msg) => (
                StatusCode::BAD_REQUEST,
                msg.clone(),
                msg,
                "APP_UPDATE_REQUIRED",
            ),
            AppError::Conflict(msg) => (StatusCode::CONFLICT, msg, "409".to_string(), "CONFLICT"),
            AppError::Unauthorized(msg) => (
                StatusCode::UNAUTHORIZED,
                msg,
                "401".to_string(),
                "UNAUTHORIZED",
            ),
            AppError::Forbidden(msg) => {
                (StatusCode::FORBIDDEN, msg, "403".to_string(), "FORBIDDEN")
            }
        };

        (
            status,
            Json(json!({
                "error": message,
                "error_code": status.as_u16(),
                "error_time": "",
                "env": "",
                "pp": pp,
                "pa": pa
            })),
        )
            .into_response()
    }
}

pub type AppResult<T> = Result<T, AppError>;
