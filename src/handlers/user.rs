use axum::{
    extract::{Path, State},
    Json,
};
use sea_orm::{ActiveModelTrait, DatabaseConnection, EntityTrait, Set};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::entities::user;
use crate::error::{AppError, AppResult};

#[derive(Deserialize)]
pub struct CreateUserRequest {
    pub username: String,
    pub email: String,
}

#[derive(Serialize)]
pub struct UserResponse {
    pub id: i32,
    pub username: String,
    pub email: String,
    pub active: bool,
}

pub async fn create_user(
    State(db): State<Arc<DatabaseConnection>>,
    Json(payload): Json<CreateUserRequest>,
) -> AppResult<Json<UserResponse>> {
    let new_user = user::ActiveModel {
        username: Set(payload.username),
        email: Set(payload.email),
        active: Set(true),
        created_at: Set(chrono::Utc::now().naive_utc()), // Simple timestamp
        ..Default::default()
    };

    let user = new_user.insert(db.as_ref()).await?;

    Ok(Json(UserResponse {
        id: user.id,
        username: user.username,
        email: user.email,
        active: user.active,
    }))
}

pub async fn get_user(
    State(db): State<Arc<DatabaseConnection>>,
    Path(id): Path<i32>,
) -> AppResult<Json<UserResponse>> {
    let user = user::Entity::find_by_id(id)
        .one(db.as_ref())
        .await?
        .ok_or(AppError::NotFound)?;

    Ok(Json(UserResponse {
        id: user.id,
        username: user.username,
        email: user.email,
        active: user.active,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use sea_orm::{MockDatabase, DatabaseBackend};
    

    #[tokio::test]
    async fn test_create_user_success() {
        // Setup Mock DB
        let db = MockDatabase::new(DatabaseBackend::Postgres)
            .append_query_results([
                // First query result (insert returns this)
                vec![user::Model {
                    id: 1,
                    username: "testuser".to_owned(),
                    email: "test@example.com".to_owned(),
                    active: true,
                    created_at: chrono::Utc::now().naive_utc(),
                }],
            ])
            .into_connection();
        
        let db = Arc::new(db);

        // Create request
        let request = CreateUserRequest {
            username: "testuser".to_string(),
            email: "test@example.com".to_string(),
        };

        // Execute handler
        let response = create_user(State(db), Json(request)).await;

        // Verify
        assert!(response.is_ok());
        let user_response = response.unwrap().0;
        assert_eq!(user_response.username, "testuser");
        assert_eq!(user_response.email, "test@example.com");
        assert_eq!(user_response.id, 1);
    }
}
