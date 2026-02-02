//! Dominio de la rockola (modelos y helpers puros).
//! Regla: Artista unificado por `artist_key` normalizado.

use unicode_normalization::UnicodeNormalization;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ArtistKey(pub String);

/// Representa un elemento multimedia
#[derive(Debug, Clone)]
pub struct MediaItem {
    pub id: i64,
    pub title: String,
    pub artist: String,
    pub media_type: MediaType,
    pub file_path: String,
    pub duration_ms: Option<i64>,
    pub size_bytes: i64,
    pub mtime_unix: i64,
}

/// Tipo de medio
#[derive(Debug, Clone, PartialEq)]
pub enum MediaType {
    Audio,
    Video,
    Unknown,
}

impl std::fmt::Display for MediaType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MediaType::Audio => write!(f, "audio"),
            MediaType::Video => write!(f, "video"),
            MediaType::Unknown => write!(f, "unknown"),
        }
    }
}

impl From<&str> for MediaType {
    fn from(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "audio" => MediaType::Audio,
            "video" => MediaType::Video,
            _ => MediaType::Unknown,
        }
    }
}

/// Representa un artista
#[derive(Debug, Clone)]
pub struct Artist {
    pub id: i64,
    pub display_name: String,
    pub artist_key: ArtistKey,
}

/// Representa una fuente de medios
#[derive(Debug, Clone)]
pub struct MediaSource {
    pub id: i64,
    pub root_path: String,
    pub enabled: bool,
    pub status: String,
    pub last_scan_at: Option<i64>,
    pub last_seen_at: i64,
}

/// Normaliza nombre de artista para unificación:
/// - trim
/// - lowercase
/// - sin tildes (NFKD + filtrar diacríticos)
/// - remove caracteres no alfanuméricos (deja espacios)
/// - colapsa espacios
pub fn normalize_artist_key(input: &str) -> ArtistKey {
    let lower = input.trim().to_lowercase();

    let mut out = String::with_capacity(lower.len());

    // NFKD separa diacríticos; filtramos marcas combinantes.
    for ch in lower.nfkd() {
        // Filtra diacríticos (unicode combining marks)
        if is_combining_mark(ch) {
            continue;
        }
        // Permitimos letras/dígitos/espacio. Todo lo demás se vuelve espacio.
        if ch.is_alphanumeric() {
            out.push(ch);
        } else if ch.is_whitespace() {
            out.push(' ');
        } else {
            out.push(' ');
        }
    }

    // Colapsar espacios múltiples
    let collapsed = out.split_whitespace().collect::<Vec<_>>().join(" ");
    ArtistKey(collapsed)
}

/// Genera un slug a partir de un título o nombre
pub fn generate_slug(input: &str) -> String {
    let normalized = input.trim().to_lowercase();
    
    let mut out = String::with_capacity(normalized.len());
    for ch in normalized.chars() {
        if ch.is_alphanumeric() {
            out.push(ch);
        } else if ch.is_whitespace() || matches!(ch, '-' | '_' | '.') {
            out.push('-');
        } else {
            out.push('-');
        }
    }
    
    // Colapsar guiones múltiples
    let collapsed = out.split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-");
    
    collapsed
}

/// Extrae el nombre del archivo sin extensión
pub fn get_filename_without_extension(file_path: &str) -> String {
    std::path::Path::new(file_path)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_string()
}

/// Extrae la extensión del archivo
pub fn get_file_extension(file_path: &str) -> String {
    std::path::Path::new(file_path)
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_lowercase()
}

/// Calcula el tamaño en formato legible
pub fn format_file_size(bytes: i64) -> String {
    const UNITS: [&str; 5] = ["B", "KB", "MB", "GB", "TB"];
    
    let mut size = bytes as f64;
    let mut unit_idx = 0;
    
    while size >= 1024.0 && unit_idx < UNITS.len() - 1 {
        size /= 1024.0;
        unit_idx += 1;
    }
    
    if unit_idx == 0 {
        format!("{} {}", bytes, UNITS[unit_idx])
    } else {
        format!("{:.2} {}", size, UNITS[unit_idx])
    }
}

/// Formatea la duración en milisegundos a formato HH:MM:SS
pub fn format_duration_ms(duration_ms: Option<i64>) -> String {
    if let Some(ms) = duration_ms {
        if ms <= 0 {
            return "0:00".to_string();
        }
        
        let total_seconds = ms / 1000;
        let hours = total_seconds / 3600;
        let minutes = (total_seconds % 3600) / 60;
        let seconds = total_seconds % 60;
        
        if hours > 0 {
            format!("{}:{:02}:{:02}", hours, minutes, seconds)
        } else {
            format!("{}:{:02}", minutes, seconds)
        }
    } else {
        "--:--".to_string()
    }
}

/// Verifica si un archivo es multimedia según su extensión
pub fn is_media_file(file_path: &str) -> bool {
    let ext = get_file_extension(file_path);
    matches!(
        ext.as_str(),
        "mp3" | "flac" | "wav" | "m4a" | "aac" | "ogg" | "opus" | "wma" | "aiff" |
        "mp4" | "mkv" | "webm" | "mov" | "avi" | "m4v" | "mpg" | "mpeg" | "ts" |
        "wmv" | "flv" | "f4v" | "m2ts" | "mts" | "3gp" | "3g2" | "mxf"
    )
}

fn is_combining_mark(ch: char) -> bool {
    // Rango general de marcas combinantes; suficiente para este caso.
    // (Alternativa: unicode_general_category, pero es más pesado)
    matches!(ch as u32, 0x0300..=0x036F | 0x1AB0..=0x1AFF | 0x1DC0..=0x1DFF | 0x20D0..=0x20FF | 0xFE20..=0xFE2F)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_artist_key() {
        assert_eq!(normalize_artist_key("The Beatles").0, "the beatles");
        assert_eq!(normalize_artist_key(" Café con Leche ").0, "cafe con leche");
        assert_eq!(normalize_artist_key("José María").0, "jose maria");
    }

    #[test]
    fn test_generate_slug() {
        assert_eq!(generate_slug("My Great Song"), "my-great-song");
        assert_eq!(generate_slug("Song & Dance!"), "song-dance");
    }

    #[test]
    fn test_format_file_size() {
        assert_eq!(format_file_size(1023), "1023 B");
        assert_eq!(format_file_size(1024), "1.00 KB");
        assert_eq!(format_file_size(1024 * 1024), "1.00 MB");
    }

    #[test]
    fn test_format_duration_ms() {
        assert_eq!(format_duration_ms(Some(3000)), "0:03");
        assert_eq!(format_duration_ms(Some(65000)), "1:05");
        assert_eq!(format_duration_ms(Some(3665000)), "1:01:05");
    }
}