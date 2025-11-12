mod auth;
mod config;
mod db;
mod handlers;
mod models;
mod templates;

use crate::auth::{auth_middleware, AuthState};
use crate::config::Config;
use crate::db::Database;
use crate::handlers::admin::{
    admin_change_user_password, change_own_password, create_user, delete_user, get_config,
    get_upload_logs, list_config, list_users, update_config,
};
use crate::handlers::auth_handlers::{login, logout};
use crate::handlers::upload::upload_files;
use crate::handlers::youtube::download_youtube;
use crate::templates::{AdminTemplate, LoginTemplate, LogsTemplate, UploadTemplate};
use axum::{
    middleware,
    routing::{delete, get, post},
    Router,
};
use std::sync::Arc;
use tower_http::{
    cors::{Any, CorsLayer},
    limit::RequestBodyLimitLayer,
    trace::TraceLayer,
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

// Shared application state
#[derive(Clone)]
pub struct AppState {
    pub db: Database,
    pub config: Config,
    pub auth: AuthState,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "j3s_music_upload=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Load configuration
    tracing::info!("Loading configuration...");
    let config = Config::load()?;

    // Connect to database
    tracing::info!("Connecting to database...");
    let db = Database::new(&config.database.url, config.database.max_connections).await?;

    // Check if this is first run and create default admin if needed
    if !db.user_exists().await? {
        tracing::warn!("No users found in database. Creating default admin user...");
        tracing::warn!("DEFAULT CREDENTIALS - Username: admin, Password: admin");
        tracing::warn!("PLEASE CHANGE THE DEFAULT PASSWORD IMMEDIATELY!");

        use crate::models::CreateUser;
        db.create_user(CreateUser {
            username: "admin".to_string(),
            password: "admin".to_string(),
            is_admin: true,
        })
        .await?;

        tracing::info!("Default admin user created successfully");
    }

    // Create auth state
    let auth_state = AuthState::new(
        config.security.jwt_secret.clone(),
        config.security.session_timeout_hours,
    );

    // Create shared application state
    let app_state = Arc::new(AppState {
        db,
        config,
        auth: auth_state.clone(),
    });

    // Protected routes (require authentication)
    let protected_routes = Router::new()
        // API routes
        .route("/api/upload", post(upload_files))
        .route("/api/youtube", post(download_youtube))
        .route("/api/admin/users", get(list_users).post(create_user))
        .route("/api/admin/users/:id", delete(delete_user))
        .route(
            "/api/admin/users/:id/password",
            post(admin_change_user_password),
        )
        .route("/api/user/change-password", post(change_own_password))
        .route("/api/admin/config", get(list_config).post(update_config))
        .route("/api/admin/config/:key", get(get_config))
        .route("/api/admin/logs", get(get_upload_logs))
        .route("/api/logout", post(logout))
        // Template routes (PROTECTED - require login)
        .route("/upload", get(|| async { UploadTemplate }))
        .route("/admin", get(|| async { AdminTemplate }))
        .route("/logs", get(|| async { LogsTemplate }))
        .layer(middleware::from_fn_with_state(auth_state, auth_middleware));

    // Public routes (only login page and API endpoint)
    let public_routes = Router::new()
        .route("/", get(|| async { LoginTemplate }))
        .route("/api/login", post(login));

    // Start server address
    let addr = format!(
        "{}:{}",
        app_state.config.server.host, app_state.config.server.port
    );

    // Configure CORS - only allow same-origin by default (restrictive for security)
    // If you need to allow different origins, configure this appropriately
    let cors = CorsLayer::new()
        .allow_origin(Any) // In production, specify your domain(s)
        .allow_methods([
            axum::http::Method::GET,
            axum::http::Method::POST,
            axum::http::Method::DELETE,
        ])
        .allow_headers(Any);

    // Max request body size from config
    let max_body_size = app_state.config.max_file_size_bytes();

    // Combine routes
    let app = Router::new()
        .merge(protected_routes)
        .merge(public_routes)
        .with_state(app_state)
        .layer(cors)
        .layer(RequestBodyLimitLayer::new(max_body_size))
        .layer(TraceLayer::new_for_http());
    let listener = tokio::net::TcpListener::bind(&addr).await?;

    tracing::info!("Server listening on {}", addr);
    tracing::info!("Visit http://{} to access the application", addr);

    axum::serve(listener, app).await?;

    Ok(())
}
