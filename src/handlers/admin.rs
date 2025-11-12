use crate::auth::AuthUser;
use crate::models::{AdminChangePasswordRequest, ChangePasswordRequest, CreateUser, User};
use axum::{
    extract::{Extension, Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Arc;

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateUserRequest {
    pub username: String,
    pub password: String,
    pub is_admin: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateConfigRequest {
    pub key: String,
    pub value: String,
}

// User management endpoints
pub async fn list_users(
    State(state): State<Arc<crate::AppState>>,
    Extension(_user): Extension<AuthUser>, // Ensures user is authenticated
) -> Result<Json<Vec<User>>, Response> {
    let users = state
        .db
        .list_users()
        .await
        .map_err(|e| internal_error(&format!("Failed to list users: {}", e)))?;

    Ok(Json(users))
}

pub async fn create_user(
    State(state): State<Arc<crate::AppState>>,
    Extension(_user): Extension<AuthUser>,
    Json(req): Json<CreateUserRequest>,
) -> Result<Json<User>, Response> {
    // Validate username
    if req.username.is_empty() || req.username.len() < 3 {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({
                "error": "Username must be at least 3 characters"
            })),
        )
            .into_response());
    }

    // Validate password
    if req.password.len() < 8 {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({
                "error": "Password must be at least 8 characters"
            })),
        )
            .into_response());
    }

    let user = state
        .db
        .create_user(CreateUser {
            username: req.username,
            password: req.password,
            is_admin: req.is_admin,
        })
        .await
        .map_err(|e| {
            if e.to_string().contains("Duplicate") {
                (
                    StatusCode::CONFLICT,
                    Json(json!({
                        "error": "Username already exists"
                    })),
                )
                    .into_response()
            } else {
                internal_error(&format!("Failed to create user: {}", e))
            }
        })?;

    Ok(Json(user))
}

pub async fn delete_user(
    State(state): State<Arc<crate::AppState>>,
    Extension(admin): Extension<AuthUser>,
    Path(user_id): Path<String>,
) -> Result<Json<serde_json::Value>, Response> {
    // Prevent deleting yourself
    if admin.user_id == user_id {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({
                "error": "Cannot delete your own account"
            })),
        )
            .into_response());
    }

    state
        .db
        .delete_user(&user_id)
        .await
        .map_err(|e| internal_error(&format!("Failed to delete user: {}", e)))?;

    Ok(Json(json!({
        "message": "User deleted successfully"
    })))
}

// Config management endpoints
pub async fn list_config(
    State(state): State<Arc<crate::AppState>>,
    Extension(_user): Extension<AuthUser>,
) -> Result<Json<Vec<(String, String)>>, Response> {
    let configs = state
        .db
        .list_config()
        .await
        .map_err(|e| internal_error(&format!("Failed to list config: {}", e)))?;

    Ok(Json(configs))
}

pub async fn update_config(
    State(state): State<Arc<crate::AppState>>,
    Extension(_user): Extension<AuthUser>,
    Json(req): Json<UpdateConfigRequest>,
) -> Result<Json<serde_json::Value>, Response> {
    state
        .db
        .set_config(&req.key, &req.value)
        .await
        .map_err(|e| internal_error(&format!("Failed to update config: {}", e)))?;

    Ok(Json(json!({
        "message": "Config updated successfully",
        "key": req.key,
        "value": req.value
    })))
}

pub async fn get_config(
    State(state): State<Arc<crate::AppState>>,
    Extension(_user): Extension<AuthUser>,
    Path(key): Path<String>,
) -> Result<Json<serde_json::Value>, Response> {
    let value = state
        .db
        .get_config(&key)
        .await
        .map_err(|e| internal_error(&format!("Failed to get config: {}", e)))?;

    match value {
        Some(v) => Ok(Json(json!({
            "key": key,
            "value": v
        }))),
        None => Err((
            StatusCode::NOT_FOUND,
            Json(json!({
                "error": "Config key not found"
            })),
        )
            .into_response()),
    }
}

// Upload logs endpoint
pub async fn get_upload_logs(
    State(state): State<Arc<crate::AppState>>,
    Extension(user): Extension<AuthUser>,
) -> Result<Json<serde_json::Value>, Response> {
    let logs = if user.is_admin {
        // Admin can see all logs
        state.db.get_upload_logs(None, 100).await
    } else {
        // Regular users can only see their own logs
        state.db.get_upload_logs(Some(&user.user_id), 100).await
    }
    .map_err(|e| internal_error(&format!("Failed to get upload logs: {}", e)))?;

    Ok(Json(json!({
        "logs": logs
    })))
}

// Password change endpoints
pub async fn change_own_password(
    State(state): State<Arc<crate::AppState>>,
    Extension(user): Extension<AuthUser>,
    Json(req): Json<ChangePasswordRequest>,
) -> Result<Json<serde_json::Value>, Response> {
    // Validate new password
    if req.new_password.len() < 8 {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({
                "error": "New password must be at least 8 characters"
            })),
        )
            .into_response());
    }

    // Verify old password
    let db_user = state
        .db
        .verify_password(&user.username, &req.old_password)
        .await
        .map_err(|_| {
            (
                StatusCode::UNAUTHORIZED,
                Json(json!({
                    "error": "Current password is incorrect"
                })),
            )
                .into_response()
        })?;

    // Update password
    state
        .db
        .update_password(&db_user.id, &req.new_password)
        .await
        .map_err(|e| internal_error(&format!("Failed to update password: {}", e)))?;

    Ok(Json(json!({
        "message": "Password changed successfully"
    })))
}

pub async fn admin_change_user_password(
    State(state): State<Arc<crate::AppState>>,
    Extension(admin): Extension<AuthUser>,
    Path(user_id): Path<String>,
    Json(req): Json<AdminChangePasswordRequest>,
) -> Result<Json<serde_json::Value>, Response> {
    // Ensure requester is admin
    if !admin.is_admin {
        return Err((
            StatusCode::FORBIDDEN,
            Json(json!({
                "error": "Admin privileges required"
            })),
        )
            .into_response());
    }

    // Validate new password
    if req.new_password.len() < 8 {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({
                "error": "New password must be at least 8 characters"
            })),
        )
            .into_response());
    }

    // Verify user exists
    state.db.get_user_by_id(&user_id).await.map_err(|_| {
        (
            StatusCode::NOT_FOUND,
            Json(json!({
                "error": "User not found"
            })),
        )
            .into_response()
    })?;

    // Update password
    state
        .db
        .update_password(&user_id, &req.new_password)
        .await
        .map_err(|e| internal_error(&format!("Failed to update password: {}", e)))?;

    Ok(Json(json!({
        "message": "User password changed successfully"
    })))
}

fn internal_error(message: &str) -> Response {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(json!({
            "error": message
        })),
    )
        .into_response()
}
