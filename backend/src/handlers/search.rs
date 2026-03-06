//! Búsqueda unificada: siempre YouTube (yt-dlp). Si falla, devuelve 502 con mensaje claro.

use axum::{
    extract::{Query, State},
    response::{Json, IntoResponse, Response},
    http::StatusCode,
};
use serde::Deserialize;

use crate::models::SearchResponse;
use crate::services::search as search_service;
use crate::AppState;

#[derive(Debug, Deserialize)]
pub struct SearchQuery {
    pub q: Option<String>,
}

/// GET /api/search?q=...
/// Devuelve { artists: [...], songs: [...] }. Si falla, 502 con { "message": "..." }.
pub async fn search(
    State(state): State<AppState>,
    Query(query): Query<SearchQuery>,
) -> Response {
    let q = query.q.unwrap_or_default().trim().to_string();
    if q.is_empty() {
        return (StatusCode::OK, Json(SearchResponse {
            artists: vec![],
            songs: vec![],
        })).into_response();
    }

    match search_service::search(
        &state.pool,
        &state.config.yt_dlp_path,
        &q,
    )
    .await
    {
        Ok(res) => (StatusCode::OK, Json(res)).into_response(),
        Err(e) => {
            tracing::warn!("search error: {}", e);
            let body = serde_json::json!({ "message": e.to_string() });
            (StatusCode::BAD_GATEWAY, Json(body)).into_response()
        }
    }
}
