//! Worker de descargas: procesa jobs de downloads_queue, guarda en mediaRoot/video|audio/<artist>/<titulo>.ext
//! y actualiza media_library y la cola de reproducción.

use tokio::task::spawn_blocking;

use crate::repository::{self, DownloadQueueRow};
use crate::services::storage::{to_relative_path, unique_audio_path, unique_video_path};
use crate::services::yt_dlp::{download_audio_to_path, download_video_to_path, new_yt_dlp_command, youtube_url_from_id};
use crate::Config;
use sqlx::SqlitePool;

/// Ejecuta un job de descarga: descarga a mediaRoot/video|audio/<artist>/<titulo>.ext,
/// inserta en media_library y actualiza la cola con la URL de stream.
pub async fn run_download_job(
    pool: SqlitePool,
    config: &Config,
    job: &DownloadQueueRow,
) -> Result<(), String> {
    let yt_dlp_path = config.yt_dlp_path.clone();
    let media_root = config.media_root.clone();
    let youtube_id = job.youtube_video_id.clone();
    let media_type = job.requested_media_type.clone();
    let job_id = job.id.clone();

    repository::update_download_status(
        &pool,
        &job_id,
        "downloading",
        Some(0.0),
        None,
        None,
    )
    .await
    .map_err(|e| e.to_string())?;

    let url = youtube_url_from_id(&youtube_id);

    // Obtener metadata (título, artista) con yt-dlp -j
    let (title, artist) = spawn_blocking({
        let yt_dlp_path = yt_dlp_path.clone();
        let url = url.clone();
        move || {
            let out = new_yt_dlp_command(&yt_dlp_path)
                .args(["-j", "--no-warnings", "--no-download", &url])
                .output()
                .map_err(|e| e.to_string())?;
            if !out.status.success() {
                return Err(String::from_utf8_lossy(&out.stderr).to_string());
            }
            let json: serde_json::Value =
                serde_json::from_slice(&out.stdout).map_err(|e| e.to_string())?;
            let title = json
                .get("title")
                .and_then(|v| v.as_str())
                .unwrap_or("Unknown")
                .to_string();
            let artist = json
                .get("uploader")
                .or(json.get("channel"))
                .and_then(|v| v.as_str())
                .unwrap_or("Unknown")
                .to_string();
            Ok::<_, String>((title, artist))
        }
    })
    .await
    .map_err(|e| e.to_string())??;

    let full_path = spawn_blocking({
        let media_root = media_root.clone();
        let media_type = media_type.clone();
        let artist = artist.clone();
        let title = title.clone();
        move || -> Result<std::path::PathBuf, String> {
            if media_type == "audio" {
                unique_audio_path(&media_root, &artist, &title)
            } else {
                unique_video_path(&media_root, &artist, &title)
            }
            .map_err(|e| e.to_string())
        }
    })
    .await
    .map_err(|e| e.to_string())??;

    let download_result = spawn_blocking({
        let yt_dlp_path = yt_dlp_path.clone();
        let full_path = full_path.clone();
        let media_type = media_type.clone();
        let url = url.clone();
        move || {
            if media_type == "audio" {
                download_audio_to_path(&yt_dlp_path, &url, &full_path)
            } else {
                download_video_to_path(&yt_dlp_path, &url, &full_path)
            }
        }
    })
    .await
    .map_err(|e| e.to_string())?;

    let final_path = match download_result {
        Ok(p) => p,
        Err(e) => {
            repository::update_download_status(
                &pool,
                &job_id,
                "failed",
                None,
                None,
                Some(&e),
            )
            .await
            .ok();
            return Err(e);
        }
    };

    let relative_for_db = to_relative_path(&media_root, &final_path)
        .unwrap_or_else(|| final_path.to_string_lossy().to_string());

    let lib_id = uuid::Uuid::new_v4().to_string();
    repository::insert_media_library(
        &pool,
        &lib_id,
        &title,
        Some(&artist),
        None,
        "youtube",
        &relative_for_db,
        None,
        None,
        &media_type,
        Some(&youtube_id),
    )
    .await
    .map_err(|e| e.to_string())?;

    repository::update_download_status(
        &pool,
        &job_id,
        "done",
        Some(1.0),
        Some(&relative_for_db),
        None,
    )
    .await
    .map_err(|e| e.to_string())?;

    // URL de stream: el frontend usa /api/media/stream?id=<lib_id>&source=local
    let stream_url = format!("/api/media/stream?id={}&source=local", lib_id);
    repository::update_queue_stream_by_download_id(&pool, &job_id, &stream_url)
        .await
        .map_err(|e| e.to_string())?;

    tracing::info!(
        "Download done: {} -> {}",
        youtube_id,
        relative_for_db
    );
    Ok(())
}
