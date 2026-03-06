//! Cola de reproducción.

use axum::{
    extract::State,
    response::Json,
    Json as JsonExtract,
};
use crate::models::{AddToQueueRequest, MediaItem, QueueResponse};
use crate::repository;
use crate::AppState;

/// GET /api/queue
pub async fn get_queue(State(state): State<AppState>) -> Result<Json<QueueResponse>, (axum::http::StatusCode, String)> {
    let queue = repository::get_queue(&state.pool)
        .await
        .map_err(|e| {
            tracing::error!("get_queue: {}", e);
            (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
        })?;
    Ok(Json(QueueResponse { queue }))
}

/// POST /api/queue — local-first: si el ítem ya está en biblioteca se encola con stream local;
/// si no, se crea un job de descarga y se encola con download_id (el worker lo procesa).
pub async fn add_to_queue(
    State(state): State<AppState>,
    JsonExtract(body): JsonExtract<AddToQueueRequest>,
) -> Result<Json<QueueResponse>, (axum::http::StatusCode, String)> {
    let cost = state.config.cost_per_song;
    let ok = repository::deduct_credits(&state.pool, cost)
        .await
        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    if !ok {
        return Err((
            axum::http::StatusCode::PAYMENT_REQUIRED,
            "Créditos insuficientes".to_string(),
        ));
    }

    let media = &body.media_item;

    // YouTube: si ya está en biblioteca local, encolar con stream_id local; si no, crear job de descarga.
    let is_youtube = media.source == "youtube"
        || (media.id.len() == 11 && media.id.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_'));

    if is_youtube {
        if let Ok(Some(lib)) = repository::get_media_by_external_id(&state.pool, &media.id).await {
            let stream_id = format!("/api/media/stream?id={}&source=local", lib.id);
            let library_media = MediaItem {
                id: lib.id,
                source: "local".to_string(),
                title: lib.title,
                artist: lib.artist,
                album: lib.album,
                duration_seconds: lib.duration_seconds.unwrap_or(0),
                thumbnail_url: lib.thumbnail_url,
                media_type: lib.media_type,
                stream_id: Some(stream_id),
            };
            let queue = repository::add_to_queue(&state.pool, &library_media, None)
                .await
                .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
            return Ok(Json(QueueResponse { queue }));
        }

        let job = repository::insert_download_job(&state.pool, &media.id, &media.media_type)
            .await
            .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

        let mut pending_media = media.clone();
        pending_media.stream_id = None;
        let queue = repository::add_to_queue(&state.pool, &pending_media, Some(&job.id))
            .await
            .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
        return Ok(Json(QueueResponse { queue }));
    }

    let queue = repository::add_to_queue(&state.pool, media, None)
        .await
        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(QueueResponse { queue }))
}

/// POST /api/queue/next — marca el primero como reproducido y devuelve la cola actualizada.
pub async fn next(State(state): State<AppState>) -> Result<Json<QueueResponse>, (axum::http::StatusCode, String)> {
    let queue = repository::mark_next_played(&state.pool)
        .await
        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(QueueResponse { queue }))
}

/// DELETE /api/queue
pub async fn clear_queue(State(state): State<AppState>) -> Result<Json<QueueResponse>, (axum::http::StatusCode, String)> {
    repository::clear_queue(&state.pool)
        .await
        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(QueueResponse { queue: vec![] }))
}
