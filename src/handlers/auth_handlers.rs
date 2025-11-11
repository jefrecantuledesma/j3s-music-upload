use crate::models::{LoginRequest, LoginResponse};
use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use std::sync::Arc;

pub async fn login(
    State(state): State<Arc<crate::AppState>>,
    Json(req): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, Response> {
    // Verify credentials
    let user = state
        .db
        .verify_password(&req.username, &req.password)
        .await
        .map_err(|_| {
            (
                StatusCode::UNAUTHORIZED,
                Json(json!({
                    "error": "Invalid username or password"
                })),
            )
                .into_response()
        })?;

    // Create token
    let token = state.auth.create_token(&user).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "error": format!("Failed to create token: {}", e)
            })),
        )
            .into_response()
    })?;

    Ok(Json(LoginResponse {
        token,
        username: user.username,
        is_admin: user.is_admin,
    }))
}

pub async fn logout() -> impl IntoResponse {
    // Since we're using JWT, logout is handled client-side
    // This endpoint exists for completeness
    Json(json!({
        "message": "Logged out successfully"
    }))
}
