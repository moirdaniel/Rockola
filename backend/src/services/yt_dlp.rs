//! Integración con yt-dlp: búsqueda en YouTube y descarga de audio/video.
//! Requiere yt-dlp (y opcionalmente ffmpeg) instalado.
//! Se eliminan variables de proxy del entorno para evitar 403 al usar proxy corporativo/VPN.

use std::collections::HashMap;
use std::path::Path;
use std::process::Command;
use crate::models::{ArtistSummary, MediaItem};

/// Crea un `Command` para yt-dlp sin variables de proxy (evita 403 con proxy/VPN).
/// Público para uso en download_worker (metadata con -j --no-download).
pub fn new_yt_dlp_command(yt_dlp_path: &str) -> Command {
    let mut cmd = Command::new(yt_dlp_path);
    cmd.env_remove("HTTP_PROXY");
    cmd.env_remove("HTTPS_PROXY");
    cmd.env_remove("http_proxy");
    cmd.env_remove("https_proxy");
    cmd.env_remove("ALL_PROXY");
    cmd.env_remove("all_proxy");
    cmd
}

fn yt_dlp_command(yt_dlp_path: &str) -> Command {
    new_yt_dlp_command(yt_dlp_path)
}

/// Resultado de una entrada en búsqueda YouTube (yt-dlp --flat-playlist -j).
#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
struct YtDlpEntry {
    id: Option<String>,
    title: Option<String>,
    #[serde(default)]
    duration: Option<f64>,
    #[serde(default)]
    thumbnail: Option<String>,
    #[serde(default)]
    thumbnails: Option<Vec<ThumbnailEntry>>,
    #[serde(default)]
    uploader: Option<String>,
    #[serde(default)]
    channel_id: Option<String>,
}

#[derive(Debug, serde::Deserialize)]
struct ThumbnailEntry {
    url: Option<String>,
}

/// Busca en YouTube usando yt-dlp (ytsearchN: query).
/// Devuelve artistas (canales únicos) y canciones (pistas entre ~1 y 10 min).
pub fn search_youtube(yt_dlp_path: &str, query: &str, limit: u32) -> Result<(Vec<ArtistSummary>, Vec<MediaItem>), String> {
    let search_str = format!("ytsearch{}:{}", limit, query.trim());
    let query_lower = query.to_lowercase();
    let output = yt_dlp_command(yt_dlp_path)
        .args(["--flat-playlist", "-j", "--no-warnings", "--no-download", &search_str])
        .output()
        .map_err(|e| {
            let msg = e.to_string();
            if msg.contains("No such file") || msg.contains("os error 2") {
                format!(
                    "yt-dlp no encontrado o error: {}. Instala: sudo pacman -S yt-dlp (Arch) o pip install yt-dlp. Si está en otra ruta, define YT_DLP_PATH en backend/.env",
                    msg
                )
            } else {
                format!("yt-dlp no encontrado o error: {}", msg)
            }
        })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("yt-dlp error: {}", stderr));
    }

    const MIN_SONG_SEC: i64 = 45;
    const MAX_SONG_SEC: i64 = 600;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut songs = Vec::new();
    let mut seen_channels: HashMap<String, (String, Option<String>)> = HashMap::new(); // channel_id -> (name, thumbnail)

    for line in stdout.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        if let Ok(entry) = serde_json::from_str::<YtDlpEntry>(line) {
            let id = match &entry.id {
                Some(id) if !id.is_empty() => id.clone(),
                _ => continue,
            };
            let thumb_url = entry
                .thumbnails
                .as_ref()
                .and_then(|v| v.first())
                .and_then(|t| t.url.clone())
                .or(entry.thumbnail);
            let title = entry.title.unwrap_or_else(|| id.clone());
            let duration_seconds = entry.duration.map(|d| d as i64).unwrap_or(0);

            // Artistas: único por channel_id
            if let Some(ref cid) = entry.channel_id {
                if !cid.is_empty() && !seen_channels.contains_key(cid) {
                    let name = entry.uploader.clone().unwrap_or_else(|| cid.clone());
                    seen_channels.insert(cid.clone(), (name, thumb_url.clone()));
                }
            }

            // Solo canciones: duración entre 45 s y 10 min (excluye shorts y álbumes largos)
            if (MIN_SONG_SEC..=MAX_SONG_SEC).contains(&duration_seconds) {
                let artist = entry.uploader.clone();
                songs.push(MediaItem {
                    id: id.clone(),
                    source: "youtube".to_string(),
                    title,
                    artist,
                    album: None,
                    duration_seconds,
                    thumbnail_url: thumb_url,
                    media_type: "video".to_string(), // YouTube: siempre tratamos como video para descargar mp4 y mostrarlo
                    stream_id: Some(id),
                });
            }
        }
    }

    let mut artists: Vec<ArtistSummary> = seen_channels
        .into_iter()
        // Solo canales cuyo nombre se parece al texto buscado (bandas/solistas con nombre similar)
        .filter(|(_, (name, _))| name.to_lowercase().contains(&query_lower))
        .map(|(id, (name, thumbnail_url))| ArtistSummary {
            id,
            name,
            thumbnail_url,
        })
        .collect();

    // Limitar a los primeros 8 artistas para no saturar la fila
    if artists.len() > 8 {
        artists.truncate(8);
    }

    Ok((artists, songs))
}

/// Descarga audio desde una URL YouTube con yt-dlp -x --audio-format mp3.
/// Guarda en `output_dir` con nombre derivado del título.
/// Devuelve la ruta absoluta del archivo creado.
pub fn download_audio(
    yt_dlp_path: &str,
    url: &str,
    output_dir: &Path,
    title_for_path: &str,
) -> Result<std::path::PathBuf, String> {
    std::fs::create_dir_all(output_dir).map_err(|e| e.to_string())?;

    let safe_title = title_for_path
        .chars()
        .map(|c| if c == '/' || c == '\\' || c == ':' { '-' } else { c })
        .collect::<String>();
    let template = output_dir.join(format!("{}.%(ext)s", safe_title));

    let output = yt_dlp_command(yt_dlp_path)
        .args([
            "-x",
            "--audio-format",
            "mp3",
            "--audio-quality",
            "0",
            "-o",
            template.to_str().unwrap(),
            "--no-warnings",
            url,
        ])
        .output()
        .map_err(|e| format!("yt-dlp: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("yt-dlp download error: {}", stderr));
    }

    let dir_entries = std::fs::read_dir(output_dir).map_err(|e| e.to_string())?;
    for entry in dir_entries.flatten() {
        let path = entry.path();
        if path.extension().is_some_and(|e| e == "mp3") {
            return Ok(path);
        }
    }
    Err("No se encontró archivo mp3 generado".to_string())
}

/// Descarga audio a un path exacto (para estructura artist/titulo.mp3).
pub fn download_audio_to_path(
    yt_dlp_path: &str,
    url: &str,
    output_path: &Path,
) -> Result<std::path::PathBuf, String> {
    let parent = output_path
        .parent()
        .ok_or("Path sin directorio padre")?;
    std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;

    let output = yt_dlp_command(yt_dlp_path)
        .args([
            "-x",
            "--audio-format",
            "mp3",
            "--audio-quality",
            "0",
            "-o",
            output_path.to_str().unwrap(),
            "--no-warnings",
            url,
        ])
        .output()
        .map_err(|e| format!("yt-dlp: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("yt-dlp download error: {}", stderr));
    }
    if output_path.exists() {
        return Ok(output_path.to_path_buf());
    }
    Err("No se encontró archivo mp3 generado".to_string())
}

/// Construye la URL de un video de YouTube a partir del id.
pub fn youtube_url_from_id(id: &str) -> String {
    format!("https://www.youtube.com/watch?v={}", id)
}

/// Descarga video desde una URL YouTube (mp4).
/// Nombre de archivo: "Artista - Título.mp4" (yt-dlp --restrict-filenames).
/// Devuelve la ruta absoluta del archivo creado.
pub fn download_video(
    yt_dlp_path: &str,
    url: &str,
    output_dir: &std::path::Path,
) -> Result<std::path::PathBuf, String> {
    std::fs::create_dir_all(output_dir).map_err(|e| e.to_string())?;

    // Nombre: "%(uploader)s - %(title)s.%(ext)s" -> Artista - Título.mp4
    let template = output_dir.join("%(uploader)s - %(title)s.%(ext)s");

    let output = yt_dlp_command(yt_dlp_path)
        .args([
            "-f",
            "bestvideo[ext=mp4]+bestaudio[ext=m4a]/best[ext=mp4]/best",
            "--merge-output-format",
            "mp4",
            "-o",
            template.to_str().unwrap(),
            "--restrict-filenames",
            "--no-warnings",
            url,
        ])
        .output()
        .map_err(|e| format!("yt-dlp: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("yt-dlp download error: {}", stderr));
    }

    // Buscar el .mp4 generado en el directorio
    let dir_entries = std::fs::read_dir(output_dir).map_err(|e| e.to_string())?;
    let mut best: Option<(std::path::PathBuf, std::time::SystemTime)> = None;
    for entry in dir_entries.flatten() {
        let path = entry.path();
        if path.extension().is_some_and(|e| e == "mp4") {
            if let Ok(meta) = entry.metadata() {
                if let Ok(mtime) = meta.modified() {
                    if best.as_ref().is_none_or(|(_, t)| mtime > *t) {
                        best = Some((path, mtime));
                    }
                }
            }
        }
    }
    best.map(|(p, _)| p).ok_or_else(|| "No se encontró archivo mp4 generado".to_string())
}

/// Descarga video a un path exacto (para estructura artist/titulo.mp4).
pub fn download_video_to_path(
    yt_dlp_path: &str,
    url: &str,
    output_path: &Path,
) -> Result<std::path::PathBuf, String> {
    let parent = output_path
        .parent()
        .ok_or("Path sin directorio padre")?;
    std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;

    let output = yt_dlp_command(yt_dlp_path)
        .args([
            "-f",
            "bestvideo[ext=mp4]+bestaudio[ext=m4a]/best[ext=mp4]/best",
            "--merge-output-format",
            "mp4",
            "-o",
            output_path.to_str().unwrap(),
            "--no-warnings",
            url,
        ])
        .output()
        .map_err(|e| format!("yt-dlp: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("yt-dlp download error: {}", stderr));
    }
    if output_path.exists() {
        return Ok(output_path.to_path_buf());
    }
    Err("No se encontró archivo mp4 generado".to_string())
}
