//! Rutas de almacenamiento local: sanitización y unicidad.
//! Estructura: mediaRoot/video/<artist>/<titulo>.mp4 o mediaRoot/audio/<artist>/<titulo>.mp3

use std::path::{Path, PathBuf};

/// Caracteres prohibidos en nombres de archivo (Windows/Unix).
const FORBIDDEN: &[char] = &['/', '\\', ':', '*', '?', '"', '<', '>', '|'];

/// Longitud máxima de un componente de ruta (evitar paths demasiado largos).
const MAX_COMPONENT_LEN: usize = 120;

/// Sanitiza una cadena para usarla como parte de ruta (artista o título).
/// Quita caracteres prohibidos, trim, reemplaza espacios múltiples, limita longitud.
pub fn sanitize_path_component(s: &str) -> String {
    let mut out: String = s
        .trim()
        .chars()
        .map(|c| if FORBIDDEN.contains(&c) { '-' } else { c })
        .collect();
    // Colapsar guiones/espacios múltiples
    while out.contains("--") {
        out = out.replace("--", "-");
        out = out.replace("  ", " ");
        out = out.trim_matches(|c| c == '-' || c == ' ').to_string();
    }
    if out.len() > MAX_COMPONENT_LEN {
        out.truncate(MAX_COMPONENT_LEN);
        out = out.trim_end_matches(|c| c == '-' || c == ' ').to_string();
    }
    if out.is_empty() {
        out = "Unknown".to_string();
    }
    out
}

/// Devuelve la ruta relativa (dentro de media_root) para un archivo de video.
/// Formato: `video/<artist>/<titulo>.mp4`. Si ya existe, añade sufijo (1), (2), etc.
pub fn unique_video_path(
    media_root: &Path,
    artist: &str,
    title: &str,
) -> std::io::Result<PathBuf> {
    let artist_s = sanitize_path_component(artist);
    let title_s = sanitize_path_component(title);
    let base = media_root.join("video").join(&artist_s);
    std::fs::create_dir_all(&base)?;
    unique_file_path(&base, &title_s, "mp4")
}

/// Devuelve la ruta relativa para un archivo de audio.
/// Formato: `audio/<artist>/<titulo>.mp3`.
pub fn unique_audio_path(
    media_root: &Path,
    artist: &str,
    title: &str,
) -> std::io::Result<PathBuf> {
    let artist_s = sanitize_path_component(artist);
    let title_s = sanitize_path_component(title);
    let base = media_root.join("audio").join(&artist_s);
    std::fs::create_dir_all(&base)?;
    unique_file_path(&base, &title_s, "mp3")
}

/// Genera path único en `dir` con nombre base y extensión. Añade (1), (2)... si existe.
fn unique_file_path(dir: &Path, base_name: &str, ext: &str) -> std::io::Result<PathBuf> {
    let mut path = dir.join(format!("{}.{}", base_name, ext));
    let mut n = 1u32;
    while path.exists() {
        path = dir.join(format!("{} ({}).{}", base_name, n, ext));
        n += 1;
        if n > 9999 {
            // Evitar bucle infinito; usar hash corto
            let suffix = format!("{:x}", std::time::SystemTime::now().elapsed().unwrap_or_default().as_secs() % 0xFFFF);
            path = dir.join(format!("{}-{}.{}", base_name, suffix, ext));
            break;
        }
    }
    Ok(path)
}

/// Convierte path absoluto en relativo respecto a media_root (para guardar en DB).
pub fn to_relative_path(media_root: &Path, absolute: &Path) -> Option<String> {
    absolute
        .strip_prefix(media_root)
        .ok()
        .and_then(|p| p.to_str())
        .map(|s| s.replace('\\', "/"))
    .or_else(|| absolute.to_str().map(String::from))
}
