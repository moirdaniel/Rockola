//! Servicios de negocio: búsqueda unificada, yt-dlp y almacenamiento local.

pub mod download_worker;
pub mod storage;
pub mod yt_dlp;

pub use download_worker::run_download_job;
pub use storage::{sanitize_path_component, to_relative_path, unique_audio_path, unique_video_path};
pub use yt_dlp::{download_audio, download_audio_to_path, download_video, download_video_to_path, search_youtube, youtube_url_from_id};

pub mod search;
