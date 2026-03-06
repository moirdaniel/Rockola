//! Tests de integración de la API.

use axum::body::Body;
use axum::http::{Request, StatusCode};
use rockola_backend::{create_app, AppState, Config};
use sqlx::sqlite::SqliteConnectOptions;
use std::path::PathBuf;
use std::str::FromStr;
use tower::ServiceExt;

/// Config mínima para tests (SQLite en memoria).
fn test_config() -> Config {
    Config {
        database_url: "sqlite::memory:".to_string(),
        port: 3000,
        cost_per_song: 100,
        media_root: PathBuf::from("."),
        yt_dlp_path: "yt-dlp".to_string(),
        admin_pin: None,
    }
}

async fn setup_app() -> axum::Router {
    let opts = SqliteConnectOptions::from_str("sqlite::memory:").unwrap().create_if_missing(true);
    let pool = sqlx::sqlite::SqlitePoolOptions::new()
        .connect_with(opts)
        .await
        .unwrap();
    sqlx::migrate!("./migrations").run(&pool).await.unwrap();
    let state = AppState {
        pool,
        config: test_config(),
        admin_sessions: std::sync::Arc::new(tokio::sync::Mutex::new(std::collections::HashMap::new())),
        admin_failed: std::sync::Arc::new(tokio::sync::Mutex::new(rockola_backend::AdminFailedAttempts::default())),
    };
    create_app(state)
}

#[tokio::test]
async fn health_returns_ok() {
    let app = setup_app().await;
    let req = Request::get("/health").body(Body::empty()).unwrap();
    let res = app.oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body = axum::body::to_bytes(res.into_body(), usize::MAX).await.unwrap();
    assert_eq!(body.as_ref(), b"ok");
}

#[tokio::test]
async fn search_empty_query_returns_empty_response() {
    let app = setup_app().await;
    let req = Request::get("/api/search?q=").body(Body::empty()).unwrap();
    let res = app.oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body = axum::body::to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert!(json["artists"].is_array() && json["artists"].as_array().unwrap().is_empty());
    assert!(json["songs"].is_array() && json["songs"].as_array().unwrap().is_empty());
}

#[tokio::test]
async fn search_with_query_returns_json_object() {
    let app = setup_app().await;
    let req = Request::get("/api/search?q=test").body(Body::empty()).unwrap();
    let res = app.oneshot(req).await.unwrap();
    assert!(res.status() == StatusCode::OK || res.status() == StatusCode::BAD_GATEWAY);
    if res.status() == StatusCode::OK {
        let body = axum::body::to_bytes(res.into_body(), usize::MAX).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert!(json.get("artists").unwrap().is_array());
        assert!(json.get("songs").unwrap().is_array());
    }
}

#[tokio::test]
async fn get_credits_returns_default_user() {
    let app = setup_app().await;
    let req = Request::get("/api/credits").body(Body::empty()).unwrap();
    let res = app.oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body = axum::body::to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["id"].as_str(), Some("default"));
    assert!(json["balance"].as_i64().unwrap() >= 0);
}

#[tokio::test]
async fn get_queue_returns_empty_array() {
    let app = setup_app().await;
    let req = Request::get("/api/queue").body(Body::empty()).unwrap();
    let res = app.oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body = axum::body::to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert!(json["queue"].is_array());
    assert!(json["queue"].as_array().unwrap().is_empty());
}

#[tokio::test]
async fn add_credits_increases_balance() {
    let app = setup_app().await;
    let body = serde_json::json!({ "amount": 500 });
    let req = Request::post("/api/credits/add")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(&body).unwrap()))
        .unwrap();
    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let res_body = axum::body::to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&res_body).unwrap();
    let balance = json["balance"].as_i64().unwrap();
    assert!(balance >= 500);
}

#[tokio::test]
async fn add_to_queue_without_credits_fails_with_402() {
    let app = setup_app().await;
    // Primero gastamos créditos poniendo balance en 0 (o usamos un usuario sin créditos)
    // La migración inserta default con 1000. Restamos 1000 con 10 canciones de 100.
    for _ in 0..10 {
        let body = serde_json::json!({
            "mediaItem": {
                "id": "test-1",
                "source": "local",
                "title": "Test",
                "durationSeconds": 180,
                "type": "audio"
            }
        });
        let req = Request::post("/api/queue")
            .header("content-type", "application/json")
            .body(Body::from(serde_json::to_vec(&body).unwrap()))
            .unwrap();
        let _ = app.clone().oneshot(req).await.unwrap();
    }
    // La siguiente debería fallar con 402
    let body = serde_json::json!({
        "mediaItem": {
            "id": "test-11",
            "source": "local",
            "title": "Test 11",
            "durationSeconds": 180,
            "type": "audio"
        }
    });
    let req = Request::post("/api/queue")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(&body).unwrap()))
        .unwrap();
    let res = app.oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::PAYMENT_REQUIRED);
}

#[tokio::test]
async fn get_admin_maintenance_returns_ok() {
    let app = setup_app().await;
    let req = Request::get("/api/maintenance").body(Body::empty()).unwrap();
    let res = app.oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body = axum::body::to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert!(json.get("queue").unwrap().is_array());
    assert!(json.get("mediaLibrary").unwrap().is_array());
    assert!(json.get("credits").is_some());
    assert!(json.get("stats").is_some());
    assert!(json["stats"].get("queueCount").is_some());
    assert!(json["stats"].get("mediaLibraryCount").is_some());
    assert!(json["stats"].get("mediaCacheCount").is_some());
}
