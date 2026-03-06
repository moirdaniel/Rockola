//! Librería del backend Rockola (compartida con binario y tests).

use axum::{routing::get, Router};
use tower_http::cors::{Any, CorsLayer};

pub mod config;
pub mod handlers;
pub mod models;
pub mod repository;
pub mod services;

pub use config::Config;
pub use handlers::{admin, credits, downloads, media, queue, search};
pub use models::*;
pub use repository::*;
pub use services::download_worker::run_download_job;

/// Estado compartido de la aplicación.
#[derive(Clone)]
pub struct AppState {
    pub pool: sqlx::SqlitePool,
    pub config: Config,
    /// token -> expires_at (unix secs). Solo usado si config.admin_pin está definido.
    pub admin_sessions: std::sync::Arc<tokio::sync::Mutex<std::collections::HashMap<String, u64>>>,
    /// Bloqueo por intentos fallidos de login.
    pub admin_failed: std::sync::Arc<tokio::sync::Mutex<AdminFailedAttempts>>,
}

/// Rate limit de login admin: tras 5 fallos se bloquea 5 minutos.
#[derive(Clone, Default)]
pub struct AdminFailedAttempts {
    pub count: u32,
    pub locked_until_secs: u64,
}

/// Construye el router de la API (para main y tests).
pub fn create_app(state: AppState) -> Router {
    Router::new()
        .route("/api/search", get(search::search))
        .route("/api/media/stream", get(media::stream))
        .route(
            "/api/queue",
            get(queue::get_queue)
                .post(queue::add_to_queue)
                .delete(queue::clear_queue),
        )
        .route("/api/queue/next", axum::routing::post(queue::next))
        .route("/api/credits", get(credits::get_credits))
        .route("/api/credits/add", axum::routing::post(credits::add_credits))
        .route("/api/admin/reset", axum::routing::post(admin::reset))
        .route("/api/admin/login", axum::routing::post(admin::login))
        .route("/api/admin/logout", axum::routing::post(admin::logout))
        .route("/api/admin/session", get(admin::session_status))
        .route("/api/admin/audit-log", get(admin::audit_log))
        .route("/api/admin/audit", axum::routing::post(admin::audit_create))
        .route("/api/admin/settings/updates", get(admin::get_update_settings).put(admin::put_update_settings))
        .route("/api/maintenance", axum::routing::get(admin::maintenance))
        .route("/api/downloads", get(downloads::list_downloads))
        .route("/api/downloads/:id/retry", axum::routing::post(downloads::retry_download))
        .route("/health", get(|| async { "ok" }))
        .layer(CorsLayer::new().allow_origin(Any).allow_methods(Any).allow_headers(Any))
        .with_state(state)
}
