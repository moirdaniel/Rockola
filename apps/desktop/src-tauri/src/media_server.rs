use axum::{
    extract::Query,
    http::{header, HeaderMap, StatusCode},
    response::IntoResponse,
    routing::get,
    Router,
};
use percent_encoding::percent_decode_str;
use serde::Deserialize;
use std::{net::SocketAddr, path::PathBuf};
use tokio::net::TcpListener;

#[derive(Debug, Deserialize)]
pub struct MediaQuery {
    /// full_path URL-encoded, ej: %2Fstorage%2Fcloud%2F...
    pub path: String,
}

fn decode_path(p: &str) -> PathBuf {
    let decoded = percent_decode_str(p).decode_utf8_lossy().to_string();
    PathBuf::from(decoded)
}

async fn media_handler(Query(q): Query<MediaQuery>) -> impl IntoResponse {
    let path = decode_path(&q.path);

    if !path.is_absolute() {
        return (StatusCode::BAD_REQUEST, "path must be absolute").into_response();
    }

    let bytes = match tokio::fs::read(&path).await {
        Ok(b) => b,
        Err(_) => return (StatusCode::NOT_FOUND, "file not found").into_response(),
    };

    let mime = mime_guess::from_path(&path).first_or_octet_stream();

    let mut headers = HeaderMap::new();
    headers.insert(header::CONTENT_TYPE, mime.to_string().parse().unwrap());

    (headers, bytes).into_response()
}

pub async fn start_media_server() -> Result<u16, Box<dyn std::error::Error + Send + Sync>> {
    let app = Router::new().route("/media", get(media_handler));

    let listener = TcpListener::bind("127.0.0.1:0").await?;
    let addr: SocketAddr = listener.local_addr()?;
    let port = addr.port();

    tauri::async_runtime::spawn(async move {
        let _ = axum::serve(listener, app).await;
    });

    Ok(port)
}
