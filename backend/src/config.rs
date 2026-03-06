//! Configuración desde variables de entorno.

use std::env;
use std::path::Path;

#[derive(Clone)]
pub struct Config {
    pub database_url: String,
    pub port: u16,
    pub cost_per_song: i64,
    pub media_root: std::path::PathBuf,
    pub yt_dlp_path: String,
    /// PIN para acceso al panel de mantenedor. Si no está definido, los endpoints admin no requieren auth.
    pub admin_pin: Option<String>,
}

/// Ruta por defecto de la BD: relativa al directorio del crate (backend) para que
/// funcione aunque el binario se ejecute desde otro cwd (p. ej. cargo run / sandbox).
fn default_database_path() -> String {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    format!("sqlite:{}/data/rockola.db", manifest_dir)
}

impl Config {
    pub fn from_env() -> anyhow::Result<Self> {
        let mut database_url = env::var("DATABASE_URL")
            .unwrap_or_else(|_| default_database_path());

        // Resolver rutas relativas (sqlite:./... o sqlite:../...) a ruta absoluta.
        if let Some(path_str) = database_url.strip_prefix("sqlite:") {
            let path = Path::new(path_str);
            if path.is_relative() {
                let cwd = env::current_dir()?;
                let absolute = cwd.join(path);
                database_url = format!("sqlite:{}", absolute.display());
            }
        }

        let port: u16 = env::var("PORT").unwrap_or_else(|_| "3000".into()).parse().unwrap_or(3000);
        let cost_per_song: i64 = env::var("COST_PER_SONG").unwrap_or_else(|_| "100".into()).parse().unwrap_or(100);

        let media_root = env::var("MEDIA_ROOT")
            .unwrap_or_else(|_| "./rockola-media".into());
        let media_root = Path::new(&media_root);
        let media_root = if media_root.is_relative() {
            let cwd = env::current_dir()?;
            cwd.join(media_root)
        } else {
            media_root.to_path_buf()
        };

        let yt_dlp_path = env::var("YT_DLP_PATH").unwrap_or_else(|_| "yt-dlp".into());

        let admin_pin = env::var("ADMIN_PIN").ok().filter(|s| !s.trim().is_empty());

        Ok(Self {
            database_url,
            port,
            cost_per_song,
            media_root,
            yt_dlp_path,
            admin_pin,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_env_with_memory_db() {
        std::env::set_var("DATABASE_URL", "sqlite::memory:");
        let result = Config::from_env();
        std::env::remove_var("DATABASE_URL");
        let config = result.unwrap();
        assert!(config.database_url.contains("memory"), "database_url should contain 'memory': {:?}", config.database_url);
        assert_eq!(config.port, 3000);
        assert_eq!(config.cost_per_song, 100);
        assert_eq!(config.yt_dlp_path, "yt-dlp");
    }
}
