//! Streaming de media: solo desde biblioteca local (modo kiosko).

use axum::{
    extract::{Query, State},
    http::{header, StatusCode},
    response::IntoResponse,
};
use serde::Deserialize;
use std::path::PathBuf;
use tokio::fs;

use crate::repository;
use crate::AppState;

#[derive(Debug, Deserialize)]
pub struct StreamQuery {
    pub id: String,
    pub source: Option<String>,
}

/// GET /api/media/stream?id=xxx&source=local|youtube
/// - source=local: id es media_library.id, se sirve el archivo local.
/// - source=youtube: solo si ya está en biblioteca local; si no, 404 (modo kiosko, sin descarga on-the-fly).
pub async fn stream(State(state): State<AppState>, Query(q): Query<StreamQuery>) -> impl IntoResponse {
    if q.id.is_empty() {
        return (StatusCode::BAD_REQUEST, "id requerido").into_response();
    }
    let source = q.source.as_deref().unwrap_or("local");

    if source == "local" {
        let row = match repository::get_media_by_id(&state.pool, &q.id).await {
            Ok(Some(r)) => r,
            Ok(None) => return (StatusCode::NOT_FOUND, "Media no encontrado").into_response(),
            Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
        };
        let local_path = match &row.local_path {
            Some(p) => p.clone(),
            None => return (StatusCode::NOT_FOUND, "Archivo local no disponible").into_response(),
        };
        let path = state.config.media_root.join(&local_path);
        if !path.starts_with(&state.config.media_root) {
            return (StatusCode::FORBIDDEN, "Ruta no permitida").into_response();
        }
        return serve_file(&path).await;
    }

    if source == "youtube" {
        // Modo kiosko: solo servir si ya está en biblioteca local; no descargar on-the-fly.
        if let Ok(Some(row)) = repository::get_media_by_external_id(&state.pool, &q.id).await {
            if let Some(ref local_path) = row.local_path {
                let path = state.config.media_root.join(local_path);
                if path.exists() {
                    return serve_file(&path).await;
                }
            }
        }
        return (StatusCode::NOT_FOUND, "No descargado; añádelo a la cola para descargar").into_response();
    }

    (StatusCode::BAD_REQUEST, "source debe ser local o youtube").into_response()
}

async fn serve_file(path: &PathBuf) -> axum::response::Response {
    let body = match fs::read(path).await {
        Ok(b) => b,
        Err(_) => return (StatusCode::NOT_FOUND, "Archivo no encontrado").into_response(),
    };
    let mime = mime_guess::from_path(path).first_or_octet_stream();
    let mut headers = axum::http::HeaderMap::new();
    headers.insert(header::CONTENT_TYPE, mime.as_ref().parse().unwrap());
    (StatusCode::OK, headers, body).into_response()
}
