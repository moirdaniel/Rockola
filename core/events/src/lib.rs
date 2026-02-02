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
    PlaybackEvent(PlaybackEvent),
    SystemEvent(SystemEvent),
    Toast(Toast),
    Error(AppError),
    StatsUpdate(StatsUpdate),
    MediaInfo(MediaInfo),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanProgress {
    pub source_id: i64,
    pub processed: u64,
    pub total: Option<u64>,
    pub phase: String,
    pub current_path: Option<String>,
    pub progress_percent: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LibraryDelta {
    pub reason: String, // "added|updated|removed|missing|upsert"
    pub item_ids: Vec<i64>,
    pub affected_artists: Vec<i64>,
    pub timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueueDelta {
    pub reason: String, // "snapshot|added|removed|moved|cleared|reordered"
    pub queue_item_ids: Vec<i64>,
    pub position: Option<usize>,
    pub timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerState {
    pub state: String, // "idle|playing|paused|buffering|stopped|error"
    pub item_id: Option<i64>,
    pub position_ms: i64,
    pub duration_ms: Option<i64>,
    pub volume: f32,
    pub playback_rate: f32,
    pub repeat_mode: String, // "none|one|all"
    pub shuffle: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaybackEvent {
    pub event_type: String, // "started|ended|seeked|paused|resumed|error"
    pub item_id: Option<i64>,
    pub position_ms: Option<i64>,
    pub duration_ms: Option<i64>,
    pub reason: Option<String>,
    pub timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemEvent {
    pub event_type: String, // "startup|shutdown|memory_warning|disk_low|network_change"
    pub severity: String,   // "info|warn|error|critical"
    pub data: Option<serde_json::Value>,
    pub timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatsUpdate {
    pub total_artists: i64,
    pub total_items: i64,
    pub total_audio: i64,
    pub total_video: i64,
    pub total_size: i64,
    pub last_scan_time: Option<i64>,
    pub update_reason: String, // "library_change|periodic|manual_refresh"
    pub timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaInfo {
    pub item_id: i64,
    pub title: String,
    pub artist: String,
    pub media_type: String, // "audio|video"
    pub duration_ms: Option<i64>,
    pub size_bytes: i64,
    pub file_path: String,
    pub file_format: Option<String>,
    pub bitrate: Option<i32>,
    pub sample_rate: Option<i32>,
    pub channels: Option<u16>,
    pub resolution: Option<String>, // "1920x1080" for videos
    pub fps: Option<f32>, // frames per second for videos
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Toast {
    pub level: String, // "info|success|warn|error"
    pub message: String,
    pub duration: Option<u32>, // milliseconds, default 3000ms
    pub action: Option<ToastAction>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToastAction {
    pub label: String,
    pub event: String, // event to trigger when clicked
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppError {
    pub code: String,
    pub message: String,
    pub context: Option<serde_json::Value>,
    pub timestamp: i64,
    pub severity: String, // "info|warn|error|critical"
}

impl Default for AppEvent {
    fn default() -> Self {
        AppEvent::SystemEvent(SystemEvent {
            event_type: "unknown".to_string(),
            severity: "info".to_string(),
            data: None,
            timestamp: 0,
        })
    }
}

impl ScanProgress {
    pub fn new(source_id: i64, processed: u64, total: Option<u64>, phase: String) -> Self {
        let progress_percent = if let Some(t) = total {
            if t > 0 {
                Some((processed as f64 / t as f64) * 100.0)
            } else {
                Some(0.0)
            }
        } else {
            None
        };

        Self {
            source_id,
            processed,
            total,
            phase,
            current_path: None,
            progress_percent,
        }
    }
}

impl LibraryDelta {
    pub fn new(reason: String, item_ids: Vec<i64>, affected_artists: Vec<i64>) -> Self {
        Self {
            reason,
            item_ids,
            affected_artists,
            timestamp: time::OffsetDateTime::now_utc().unix_timestamp(),
        }
    }
}

impl QueueDelta {
    pub fn new(reason: String, queue_item_ids: Vec<i64>) -> Self {
        Self {
            reason,
            queue_item_ids,
            position: None,
            timestamp: time::OffsetDateTime::now_utc().unix_timestamp(),
        }
    }
}

impl StatsUpdate {
    pub fn new(
        total_artists: i64,
        total_items: i64,
        total_audio: i64,
        total_video: i64,
        total_size: i64,
        last_scan_time: Option<i64>,
        update_reason: String,
    ) -> Self {
        Self {
            total_artists,
            total_items,
            total_audio,
            total_video,
            total_size,
            last_scan_time,
            update_reason,
            timestamp: time::OffsetDateTime::now_utc().unix_timestamp(),
        }
    }
}