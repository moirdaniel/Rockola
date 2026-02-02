//! Eventos que el core emite hacia la UI (push).
//! En Modo A: Tauri events. En Modo B: WS /events (mismo payload).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload")]
pub enum AppEvent {
    ScanProgress(ScanProgress),
    LibraryDelta(LibraryDelta),
    QueueDelta(QueueDelta),
    PlayerState(PlayerState),
    Toast(Toast),
    Error(AppError),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanProgress {
    pub source_id: i64,
    pub processed: u64,
    pub total: Option<u64>,
    pub phase: String,
    pub current_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LibraryDelta {
    pub reason: String, // "added|updated|removed"
    pub item_ids: Vec<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueueDelta {
    pub reason: String, // "snapshot|added|removed|moved|cleared"
    pub queue_item_ids: Vec<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerState {
    pub state: String, // "idle|playing|paused|error"
    pub item_id: Option<i64>,
    pub position_ms: i64,
    pub duration_ms: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Toast {
    pub level: String, // "info|warn|error"
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppError {
    pub code: String,
    pub message: String,
    pub context: Option<serde_json::Value>,
}
