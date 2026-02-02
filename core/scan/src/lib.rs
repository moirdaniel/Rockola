//! Scanner recursivo:
//! - sources (root_path) => recorre archivos
//! - artista = carpeta padre
//! - upsert a SQLite
//! - retorna ids upsertados + lista de seen full paths para marcar missing

use core_db::Db;
use core_domain::normalize_artist_key;
use core_events::{AppEvent, ScanProgress, LibraryDelta};
// use rusqlite::Connection;
use std::path::{Path, PathBuf};
use thiserror::Error;
use walkdir::WalkDir;

#[derive(Debug, Error)]
pub enum ScanError {
    #[error("db error: {0}")]
    Db(#[from] core_db::DbError),
    #[error("sqlite error: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("walk error: {0}")]
    Walk(String),
}

pub type ScanResult<T> = Result<T, ScanError>;

pub trait EventSink: Send + Sync {
    fn emit(&self, event: AppEvent);
}

#[derive(Clone)]
pub struct Scanner {
    db: Db,
}

impl Scanner {
    pub fn new(db: Db) -> Self {
        Self { db }
    }

    pub fn scan_source(&self, source_id: i64, root_path: &str, sink: &dyn EventSink) -> ScanResult<()> {

        let mut conn = self.db.connect()?;

        let root = PathBuf::from(root_path);
        if !root.exists() {
            // source offline
            sink.emit(AppEvent::Error(core_events::AppError{
                code: "SOURCE_NOT_FOUND".into(),
                message: format!("Source no existe: {root_path}"),
                context: None,
            }));
            return Ok(());
        }

        let mut processed: u64 = 0;
        let mut seen_full_paths: Vec<String> = Vec::new();
        let mut upserted_ids: Vec<i64> = Vec::new();

        sink.emit(AppEvent::ScanProgress(ScanProgress{
            source_id,
            processed,
            total: None,
            phase: "walking".into(),
            current_path: Some(root_path.into()),
        }));

        for entry in WalkDir::new(&root).follow_links(true).into_iter() {
            let entry = match entry {
                Ok(e) => e,
                Err(e) => {
                    // No abortamos todo por un archivo raro
                    sink.emit(AppEvent::Toast(core_events::Toast{
                        level: "warn".into(),
                        message: format!("Walk warning: {e}"),
                    }));
                    continue;
                }
            };

            if !entry.file_type().is_file() {
                continue;
            }

            let path = entry.path();

            // Filtrado MVP (amplio). Puedes expandir sin miedo.
            if !is_candidate_media(path) {
                continue;
            }

            let full_path = path.to_string_lossy().to_string();
            let rel_path = path.strip_prefix(&root)
                .unwrap_or(path)
                .to_string_lossy()
                .to_string();

            let artist_folder = path.parent()
                .and_then(|p| p.file_name())
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_else(|| "Unknown Artist".into());

            let artist_key = normalize_artist_key(&artist_folder).0;

            let title = path.file_stem()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_else(|| "Untitled".into());

            let media_type = guess_media_type(path);
            let meta = match std::fs::metadata(path) {
                Ok(m) => m,
                Err(_) => continue,
            };

            let size_bytes = meta.len() as i64;
            let mtime_unix = meta.modified()
                .ok()
                .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                .map(|d| d.as_secs() as i64)
                .unwrap_or(0);

            // upsert artist + media_item
            let artist_id = core_db::Db::upsert_artist(&conn, &artist_folder, &artist_key)?;
            let item_id = core_db::Db::upsert_media_item(
                &conn,
                source_id,
                artist_id,
                &full_path,
                &rel_path,
                &title,
                media_type,
                None,            // duration_ms: luego con probe
                size_bytes,
                mtime_unix,
                "unknown",       // playable: luego con probe
            )?;

            processed += 1;
            seen_full_paths.push(full_path);
            upserted_ids.push(item_id);

            if processed % 50 == 0 {
                sink.emit(AppEvent::ScanProgress(ScanProgress{
                    source_id,
                    processed,
                    total: None,
                    phase: "walking".into(),
                    current_path: Some(rel_path.clone()),
                }));
                sink.emit(AppEvent::LibraryDelta(LibraryDelta{
                    reason: "upsert".into(),
                    item_ids: upserted_ids.clone(),
                }));
                upserted_ids.clear();
            }
        }

        // flush deltas
        if !upserted_ids.is_empty() {
            sink.emit(AppEvent::LibraryDelta(LibraryDelta{
                reason: "upsert".into(),
                item_ids: upserted_ids,
            }));
        }

        // marcar missing lo que ya no está
        let missing_changed = core_db::Db::mark_missing_for_source(&mut conn, source_id, &seen_full_paths)?;
        
        if missing_changed > 0 {
            sink.emit(AppEvent::LibraryDelta(LibraryDelta{
                reason: "missing".into(),
                item_ids: vec![],
            }));
        }

        core_db::Db::mark_source_scan_done(&conn, source_id)?;

        sink.emit(AppEvent::ScanProgress(ScanProgress{
            source_id,
            processed,
            total: None,
            phase: "done".into(),
            current_path: None,
        }));

        Ok(())
    }
}

// MVP: set amplio por extensión.
// Puedes ampliar en caliente sin romper DB.
fn is_candidate_media(path: &Path) -> bool {
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();
    matches!(
        ext.as_str(),
        "mp3"|"flac"|"wav"|"m4a"|"aac"|"ogg"|"opus"|"wma"|"aiff"|
        "mp4"|"mkv"|"webm"|"mov"|"avi"|"m4v"|"mpg"|"mpeg"|"ts"
    )
}

fn guess_media_type(path: &Path) -> &'static str {
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();
    match ext.as_str() {
        "mp3"|"flac"|"wav"|"m4a"|"aac"|"ogg"|"opus"|"wma"|"aiff" => "audio",
        "mp4"|"mkv"|"webm"|"mov"|"avi"|"m4v"|"mpg"|"mpeg"|"ts" => "video",
        _ => "unknown",
    }
}
