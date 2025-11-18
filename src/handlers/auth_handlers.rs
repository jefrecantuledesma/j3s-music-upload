use crate::models::{LoginRequest, LoginResponse};
use axum::{
    extract::State,
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use axum_extra::extract::cookie::{Cookie, SameSite};
use serde_json::json;
use std::sync::Arc;

pub async fn login(
    State(state): State<Arc<crate::AppState>>,
    Json(req): Json<LoginRequest>,
) -> Result<Response, Response> {
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

    // Create HTTP-only cookie for browser-based auth
    let cookie = Cookie::build(("token", token.clone()))
        .path("/")
        .max_age(time::Duration::hours(state.auth.session_timeout_hours))
        .same_site(SameSite::Lax)
        .http_only(true)
        .build();

    // Return JSON response with Set-Cookie header
    let response = Json(LoginResponse {
        token: token.clone(),
        username: user.username,
        is_admin: user.is_admin,
    });

    Ok((
        [(header::SET_COOKIE, cookie.to_string())],
        response,
    )
        .into_response())
}

pub async fn logout() -> impl IntoResponse {
    // Clear the authentication cookie
    let cookie = Cookie::build(("token", ""))
        .path("/")
        .max_age(time::Duration::seconds(0))
        .same_site(SameSite::Lax)
        .http_only(true)
        .build();

    (
        [(header::SET_COOKIE, cookie.to_string())],
        Json(json!({
            "message": "Logged out successfully"
        })),
    )
}
