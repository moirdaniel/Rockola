//! API de cola de descargas: listar y reintentar. Requieren sesión admin si ADMIN_PIN está definido.

use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    Json,
};

use crate::handlers::admin;
use crate::models::DownloadQueueItem;
use crate::repository;
use crate::services::download_worker;
use crate::AppState;

/// GET /api/downloads — lista los últimos jobs de descarga.
pub async fn list_downloads(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<Vec<DownloadQueueItem>>, (StatusCode, String)> {
    admin::require_admin(&state, &headers).await?;
    let list = repository::list_downloads(&state.pool, 100)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(list))
}

/// POST /api/downloads/:id/retry — pone un job fallido de nuevo en cola y lo procesa.
pub async fn retry_download(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> impl IntoResponse {
    if let Err((status, msg)) = admin::require_admin(&state, &headers).await {
        return (status, msg).into_response();
    }

    let job = match repository::get_download_by_id(&state.pool, &id).await {
        Ok(Some(j)) => j,
        Ok(None) => return (StatusCode::NOT_FOUND, "Descarga no encontrada").into_response(),
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    };

    if job.status != "failed" {
        return (
            StatusCode::BAD_REQUEST,
            "Solo se puede reintentar un job con estado 'failed'".to_string(),
        )
            .into_response();
    }

    if repository::update_download_status(
        &state.pool,
        &id,
        "queued",
        None,
        None,
        None,
    )
    .await
    .is_err()
    {
        return (StatusCode::INTERNAL_SERVER_ERROR, "Error al actualizar estado").into_response();
    }

    let pool = state.pool.clone();
    let config = state.config.clone();
    tokio::spawn(async move {
        if let Ok(Some(job)) = repository::get_download_by_id(&pool, &id).await {
            if let Err(e) = download_worker::run_download_job(pool, &config, &job).await {
                tracing::error!("retry download {} failed: {}", id, e);
            }
        }
    });

    (StatusCode::OK, "Reintento encolado").into_response()
}
