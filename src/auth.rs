use crate::models::{Claims, User};
use axum::{
    extract::{Request, State},
    http::{header, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde_json::json;

#[derive(Clone)]
pub struct AuthState {
    pub jwt_secret: String,
    pub session_timeout_hours: i64,
}

impl AuthState {
    pub fn new(jwt_secret: String, session_timeout_hours: i64) -> Self {
        Self {
            jwt_secret,
            session_timeout_hours,
        }
    }

    pub fn create_token(&self, user: &User) -> Result<String, jsonwebtoken::errors::Error> {
        let expiration = Utc::now()
            .checked_add_signed(Duration::hours(self.session_timeout_hours))
            .expect("valid timestamp")
            .timestamp();

        let claims = Claims {
            sub: user.id.clone(),
            username: user.username.clone(),
            is_admin: user.is_admin,
            exp: expiration,
        };

        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.jwt_secret.as_bytes()),
        )
    }

    pub fn verify_token(&self, token: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
        let token_data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(self.jwt_secret.as_bytes()),
            &Validation::default(),
        )?;

        Ok(token_data.claims)
    }
}

// Extension to store authenticated user info in request
#[derive(Clone)]
pub struct AuthUser {
    pub user_id: String,
    pub username: String,
    pub is_admin: bool,
}

impl AuthUser {
    pub fn from_claims(claims: Claims) -> Self {
        Self {
            user_id: claims.sub,
            username: claims.username,
            is_admin: claims.is_admin,
        }
    }
}

// Middleware for authentication
pub async fn auth_middleware(
    State(auth_state): State<AuthState>,
    mut request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let auth_header = request
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|h| h.to_str().ok());

    let token = if let Some(auth) = auth_header {
        auth.strip_prefix("Bearer ").unwrap_or(auth)
    } else {
        return Err(StatusCode::UNAUTHORIZED);
    };

    match auth_state.verify_token(token) {
        Ok(claims) => {
            request.extensions_mut().insert(AuthUser::from_claims(claims));
            Ok(next.run(request).await)
        }
        Err(_) => Err(StatusCode::UNAUTHORIZED),
    }
}

// Middleware for admin-only routes
pub async fn admin_middleware(
    auth_user: Option<axum::extract::Extension<AuthUser>>,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    match auth_user {
        Some(axum::extract::Extension(user)) if user.is_admin => Ok(next.run(request).await),
        _ => Err(StatusCode::FORBIDDEN),
    }
}

// Error response helper
pub fn auth_error(message: &str) -> Response {
    (
        StatusCode::UNAUTHORIZED,
        Json(json!({
            "error": message
        })),
    )
        .into_response()
}
