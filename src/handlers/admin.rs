use crate::auth::AuthUser;
use crate::models::{AdminChangePasswordRequest, ChangePasswordRequest, CreateUser, UpdateLibraryPathRequest, User};
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
    pub library_path: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateConfigRequest {
    pub key: String,
    pub value: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateUsernameRequest {
    pub new_username: String,
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
            library_path: req.library_path,
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

// Update user's library path (admin only)
pub async fn update_user_library_path(
    State(state): State<Arc<crate::AppState>>,
    Extension(admin): Extension<AuthUser>,
    Path(user_id): Path<String>,
    Json(req): Json<UpdateLibraryPathRequest>,
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

    // Validate library path
    if req.library_path.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({
                "error": "Library path cannot be empty"
            })),
        )
            .into_response());
    }

    // SECURITY: Validate path doesn't contain path traversal attempts
    if req.library_path.contains("..") {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({
                "error": "Library path contains invalid characters (..)"
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

    // Update library path
    state
        .db
        .update_library_path(&user_id, &req.library_path)
        .await
        .map_err(|e| internal_error(&format!("Failed to update library path: {}", e)))?;

    Ok(Json(json!({
        "message": "User library path updated successfully",
        "library_path": req.library_path
    })))
}

// System info endpoint (admin only)
pub async fn get_system_info(
    State(state): State<Arc<crate::AppState>>,
    Extension(admin): Extension<AuthUser>,
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

    // Get ferric_enabled from database (overrides config file)
    let ferric_enabled = state
        .db
        .get_ferric_enabled(&state.config)
        .await
        .unwrap_or(state.config.paths.ferric_enabled);

    Ok(Json(json!({
        "ferric_enabled": ferric_enabled,
        "spotify_enabled": state.config.spotify.enabled,
        "youtube_enabled": state.config.youtube.enabled,
    })))
}

// User info endpoint (for current user)
pub async fn get_user_info(
    State(state): State<Arc<crate::AppState>>,
    Extension(user): Extension<AuthUser>,
) -> Result<Json<User>, Response> {
    let db_user = state
        .db
        .get_user_by_id(&user.user_id)
        .await
        .map_err(|e| internal_error(&format!("Failed to get user: {}", e)))?;

    Ok(Json(db_user))
}

// Get user directories (for debugging)
pub async fn get_user_directories_info(
    State(state): State<Arc<crate::AppState>>,
    Extension(user): Extension<AuthUser>,
) -> Result<Json<serde_json::Value>, Response> {
    let db_user = state
        .db
        .get_user_by_id(&user.user_id)
        .await
        .map_err(|e| internal_error(&format!("Failed to get user: {}", e)))?;

    // Get what the directories would be
    let (music_dir, temp_dir) = crate::paths::get_user_directories(&state.config, &db_user.library_path)
        .await
        .map_err(|e| internal_error(&format!("Failed to get directories: {}", e)))?;

    Ok(Json(json!({
        "username": user.username,
        "library_path": db_user.library_path,
        "music_dir": music_dir.display().to_string(),
        "temp_dir": temp_dir.display().to_string(),
        "music_dir_exists": music_dir.exists(),
        "temp_dir_exists": temp_dir.exists(),
    })))
}

// Change own username
pub async fn change_own_username(
    State(state): State<Arc<crate::AppState>>,
    Extension(user): Extension<AuthUser>,
    Json(req): Json<UpdateUsernameRequest>,
) -> Result<Json<serde_json::Value>, Response> {
    // Validate new username
    if req.new_username.is_empty() || req.new_username.len() < 3 {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({
                "error": "Username must be at least 3 characters"
            })),
        )
            .into_response());
    }

    // Update username
    state
        .db
        .update_username(&user.user_id, &req.new_username)
        .await
        .map_err(|e| {
            if e.to_string().contains("Duplicate") || e.to_string().contains("UNIQUE") {
                (
                    StatusCode::CONFLICT,
                    Json(json!({
                        "error": "Username already exists"
                    })),
                )
                    .into_response()
            } else {
                internal_error(&format!("Failed to update username: {}", e))
            }
        })?;

    Ok(Json(json!({
        "message": "Username changed successfully",
        "new_username": req.new_username
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
