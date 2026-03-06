//! Servicio de búsqueda unificada: siempre consulta YouTube (yt-dlp) a partir del query.

use crate::models::SearchResponse;
use crate::services::yt_dlp;
use sqlx::SqlitePool;

/// Busca en YouTube con yt-dlp. Devuelve artistas y canciones (solo pistas).
pub async fn search(
    _pool: &SqlitePool,
    yt_dlp_path: &str,
    query: &str,
) -> Result<SearchResponse, String> {
    let q = query.trim();
    if q.is_empty() {
        return Ok(SearchResponse {
            artists: vec![],
            songs: vec![],
        });
    }
    let (artists, songs) = yt_dlp::search_youtube(yt_dlp_path, q, 20)?;
    Ok(SearchResponse { artists, songs })
}
