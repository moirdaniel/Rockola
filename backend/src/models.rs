//! Modelos de dominio y DTOs para la API.
//! Aceptamos camelCase desde el frontend (serde rename).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaItem {
    pub id: String,
    pub source: String,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub artist: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub album: Option<String>,
    pub duration_seconds: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thumbnail_url: Option<String>,
    #[serde(rename = "type")]
    pub media_type: String, // "audio" | "video"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream_id: Option<String>,
}

/// Resumen de artista/canal para filtro en búsqueda.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArtistSummary {
    pub id: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thumbnail_url: Option<String>,
}

/// Respuesta de búsqueda: artistas (filtro horizontal) y canciones (solo pistas).
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchResponse {
    pub artists: Vec<ArtistSummary>,
    pub songs: Vec<MediaItem>,
}

/// Respuesta de cola: item plano (queueId + campos de media) para el frontend.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QueueItemResponse {
    pub queue_id: String,
    pub added_at: String,
    pub order: i64,
    pub id: String,
    pub source: String,
    pub title: String,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub duration_seconds: i64,
    pub thumbnail_url: Option<String>,
    #[serde(rename = "type")]
    pub media_type: String,
    pub stream_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub download_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub download_status: Option<String>, // "queued" | "downloading" | "done" | "failed"
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddToQueueRequest {
    pub media_item: MediaItem,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QueueResponse {
    pub queue: Vec<QueueItemResponse>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UserCredits {
    pub id: String,
    pub balance: i64,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
pub struct AddCreditsRequest {
    pub amount: i64,
}

// ---------- Descargas y admin ----------

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DownloadQueueItem {
    pub id: String,
    pub youtube_video_id: String,
    pub requested_media_type: String, // "audio" | "video"
    pub status: String,               // "queued" | "downloading" | "done" | "failed"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub progress: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_message: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AdminAuditLogItem {
    pub id: String,
    pub action: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub entity_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub entity_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payload_json: Option<String>,
    pub created_at: String,
}
