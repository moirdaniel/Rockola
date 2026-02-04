//! Scanner recursivo de biblioteca (Rockola):
//! - sources (root_path) => recorre archivos candidatos (audio/video).
//! - artista = carpeta padre inmediata del archivo.
//! - upsert a SQLite (artist + media_item).
//! - NO mantiene en memoria la lista completa de "seen paths":
//!   usa una tabla TEMP `_scan_seen` para registrar lo visto y luego marca "missing" por SQL.
//!
//! ## Objetivo de eficiencia
//! - Evitar `Vec<String>` gigantes (1 string por archivo) => alta RAM.
//! - Reducir allocations al filtrar extensiones.
//! - Emitir eventos de progreso y deltas por lotes para bajar overhead.
//!
//! > Nota: Este módulo asume un esquema de tabla `media_items` con columnas:
//! > `source_id`, `full_path` y `missing` (0/1). Si en tu `core_db` el nombre difiere,
//! > ajusta las constantes `MEDIA_ITEMS_TABLE`, `COL_FULL_PATH`, `COL_MISSING`.
//!
//! Requiere: `rusqlite`, `walkdir`, `thiserror`.

use core_db::Db;
use core_domain::normalize_artist_key;
use core_events::{AppEvent, LibraryDelta, ScanProgress};
use rusqlite::{params, Connection};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use thiserror::Error;
use walkdir::WalkDir;

/// Tabla y columnas esperadas para el update missing.
/// Si tu esquema difiere, cámbialo acá y recompila.
const MEDIA_ITEMS_TABLE: &str = "media_items";
const COL_SOURCE_ID: &str = "source_id";
const COL_FULL_PATH: &str = "full_path";
const COL_MISSING: &str = "missing";

/// TEMP table name para el scan (con PRIMARY KEY => dedup sin RAM).
const TEMP_SEEN_TABLE: &str = "_scan_seen";

/// Emitir progreso cada N upserts (trade-off UI vs overhead).
const PROGRESS_EVERY: u64 = 50;

/// Emitir deltas cada N upserts (batch para frontend/cache).
const DELTA_EVERY: usize = 200;

/// Extensiones soportadas (curadas).
/// Mantener este set pequeño reduce I/O innecesario y acelera el walk.
/// Si quieres ampliar, agrega y deja ORDENADO (para binary_search).
const AUDIO_EXTS: &[&str] = &[
    "aac", "aiff", "alac", "ape", "flac", "m4a", "m4b", "mp1", "mp2", "mp3", "oga", "ogg", "opus",
    "ra", "ram", "snd", "spx", "tta", "voc", "vqf", "w64", "wav", "weba", "wma", "wv",
];

const VIDEO_EXTS: &[&str] = &[
    "3g2", "3gp", "avi", "flv", "m2ts", "m4v", "mkv", "mov", "mp4", "mpeg", "mpg", "mts", "ogv",
    "rm", "rmvb", "ts", "webm", "wmv",
];

#[derive(Debug, Error)]
pub enum ScanError {
    #[error("db error: {0}")]
    Db(#[from] core_db::DbError),

    #[error("sqlite error: {0}")]
    Sqlite(#[from] rusqlite::Error),

    #[error("walk error: {0}")]
    Walk(String),

    #[error("cancelled")]
    Cancelled,
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
        self.scan_source_with_cancel(source_id, root_path, sink, &AtomicBool::new(false))
    }

    pub fn scan_source_with_cancel(
        &self,
        source_id: i64,
        root_path: &str,
        sink: &dyn EventSink,
        cancelled: &AtomicBool,
    ) -> ScanResult<()> {
        let mut conn = self.db.connect()?;

        let root = PathBuf::from(root_path);
        if !root.exists() {
            // source offline
            sink.emit(AppEvent::Error(core_events::AppError {
                code: "SOURCE_NOT_FOUND".into(),
                message: format!("Source no existe: {root_path}"),
                context: None,
                timestamp: time::OffsetDateTime::now_utc().unix_timestamp(),
                severity: "error".into(),
            }));
            return Ok(());
        }

        // Preparación: TEMP table para registrar paths vistos (sin RAM).
        self.prepare_scan_seen_table(&conn)?;

        let total_files = self.count_candidate_files(&root);

        sink.emit(AppEvent::ScanProgress(ScanProgress {
            source_id,
            processed: 0,
            total: Some(total_files),
            phase: "walking".into(),
            current_path: Some(root_path.into()),
            progress_percent: Some(0.0),
        }));

        let mut processed: u64 = 0;
        let mut upserted_ids: Vec<i64> = Vec::with_capacity(DELTA_EVERY);

        // Inserción a TEMP table: statement preparado para máxima performance.
        {
        let tx = conn.transaction()?;
        let mut stmt_seen = tx.prepare_cached(&format!(
            "INSERT OR IGNORE INTO {TEMP_SEEN_TABLE}(path) VALUES (?1)"
        ))?;

        // Un scan grande conviene transaccionar para:
        // - acelerar inserts
        // - evitar fsync por cada operación

        for entry in WalkDir::new(&root).follow_links(true).into_iter() {
            if cancelled.load(Ordering::Relaxed) {
                return Err(ScanError::Cancelled);
            }

            let entry = match entry {
                Ok(e) => e,
                Err(e) => {
                    sink.emit(AppEvent::Toast(core_events::Toast {
                        level: "warn".into(),
                        message: format!("Walk warning: {e}"),
                        duration: None,
                        action: None,
                    }));
                    continue;
                }
            };

            if !entry.file_type().is_file() {
                continue;
            }

            let path = entry.path();

            if !is_candidate_media(path) {
                continue;
            }

            // Ojo: to_string_lossy puede allocar; solo lo hacemos si pasa filtros.
            let full_path = path.to_string_lossy().to_string();

            // Registrar "seen" en TEMP table (sin acumular en RAM).
            stmt_seen.execute(params![&full_path])?;

            let rel_path = path
                .strip_prefix(&root)
                .unwrap_or(path)
                .to_string_lossy()
                .to_string();

            let artist_folder = path
                .parent()
                .and_then(|p| p.file_name())
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_else(|| "Unknown Artist".into());

            let artist_key = normalize_artist_key(&artist_folder).0;

            let title = path
                .file_stem()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_else(|| "Untitled".into());

            let media_type = guess_media_type(path);

            let meta = match std::fs::metadata(path) {
                Ok(m) => m,
                Err(_) => continue,
            };

            let size_bytes = meta.len() as i64;
            let mtime_unix = meta
                .modified()
                .ok()
                .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                .map(|d| d.as_secs() as i64)
                .unwrap_or(0);

            // upsert artist + media_item
            let artist_id = core_db::Db::upsert_artist(&tx, &artist_folder, &artist_key)?;
            let item_id = core_db::Db::upsert_media_item(
                &tx,
                source_id,
                artist_id,
                &full_path,
                &rel_path,
                &title,
                media_type,
                None,      // duration_ms: luego con probe
                size_bytes,
                mtime_unix,
                "unknown", // playable: luego con probe
            )?;

            processed += 1;
            upserted_ids.push(item_id);

            if (processed % PROGRESS_EVERY) == 0 {
                sink.emit(AppEvent::ScanProgress(ScanProgress {
                    source_id,
                    processed,
                    total: Some(total_files),
                    phase: "walking".into(),
                    current_path: Some(rel_path.clone()),
                    progress_percent: Some(progress_percent(processed, total_files)),
                }));
            }

            if upserted_ids.len() >= DELTA_EVERY {
                sink.emit(AppEvent::LibraryDelta(LibraryDelta {
                    reason: "upsert".into(),
                    item_ids: upserted_ids.clone(),
                    affected_artists: vec![],
                    timestamp: time::OffsetDateTime::now_utc().unix_timestamp(),
                }));
                upserted_ids.clear();
            }
        }

        // Flush de delta final
        if !upserted_ids.is_empty() {
            sink.emit(AppEvent::LibraryDelta(LibraryDelta {
                reason: "upsert".into(),
                item_ids: upserted_ids,
                affected_artists: vec![],
                timestamp: time::OffsetDateTime::now_utc().unix_timestamp(),
            }));
        }

        // 1) Unmark missing (lo visto existe)
        // 2) Mark missing (lo que estaba en DB pero no se vio)
        //
        // IMPORTANTE: esto depende del esquema. Si tu DB usa otro nombre de tabla/columna,
        // ajusta las constantes arriba.
        let unmissing = tx.execute(
            &format!(
                "UPDATE {MEDIA_ITEMS_TABLE} \
                 SET {COL_MISSING}=0 \
                 WHERE {COL_SOURCE_ID}=?1 \
                   AND {COL_FULL_PATH} IN (SELECT path FROM {TEMP_SEEN_TABLE})"
            ),
            params![source_id],
        )?;

        let missing = tx.execute(
            &format!(
                "UPDATE {MEDIA_ITEMS_TABLE} \
                 SET {COL_MISSING}=1 \
                 WHERE {COL_SOURCE_ID}=?1 \
                   AND {COL_FULL_PATH} NOT IN (SELECT path FROM {TEMP_SEEN_TABLE})"
            ),
            params![source_id],
        )?;

        // commit transacción
        drop(stmt_seen);
        tx.commit()?;

        // Sólo avisamos delta missing si hubo cambios.
        if missing > 0 || unmissing > 0 {
            sink.emit(AppEvent::LibraryDelta(LibraryDelta {
                reason: "missing".into(),
                item_ids: vec![],
                affected_artists: vec![],
                timestamp: time::OffsetDateTime::now_utc().unix_timestamp(),
            }));
        }

        }
        core_db::Db::mark_source_scan_done(&conn, source_id)?;

        sink.emit(AppEvent::ScanProgress(ScanProgress {
            source_id,
            processed,
            total: Some(total_files),
            phase: "done".into(),
            current_path: None,
            progress_percent: Some(100.0),
        }));

        Ok(())
    }

    /// Crea/limpia la TEMP table usada para marcar vistos.
    fn prepare_scan_seen_table(&self, conn: &Connection) -> ScanResult<()> {
        // TEMP => vive sólo para esta conexión; se destruye al cerrar.
        conn.execute_batch(&format!(
            "DROP TABLE IF EXISTS {TEMP_SEEN_TABLE}; \
             CREATE TEMP TABLE {TEMP_SEEN_TABLE} ( \
                path TEXT PRIMARY KEY \
             );"
        ))?;
        Ok(())
    }

    /// Cuenta candidatos para progreso.
    /// Nota: es un segundo walk; si quieres aún menos IO, puedes omitir total y reportar sólo processed.
    fn count_candidate_files(&self, root: &Path) -> u64 {
        let mut count = 0u64;
        for entry in WalkDir::new(root).follow_links(true).into_iter().flatten() {
            if entry.file_type().is_file() && is_candidate_media(entry.path()) {
                count += 1;
            }
        }
        count
    }
}

#[inline]
fn progress_percent(processed: u64, total: u64) -> f64 {
    if total == 0 {
        0.0
    } else {
        (processed as f64 / total as f64) * 100.0
    }
}

/// Determina si el archivo es candidato por extensión.
/// Implementación sin asignaciones grandes:
/// - no usa una lista gigante de `matches!`
/// - evita `to_lowercase()` (alloc) usando `to_ascii_lowercase()` sólo para la extensión (muy chica).
fn is_candidate_media(path: &Path) -> bool {
    let ext = match path.extension().and_then(|e| e.to_str()) {
        Some(e) if !e.is_empty() => e,
        _ => return false,
    };

    // Extensiones suelen ser cortas (<= 5), esta alloc es marginal.
    let ext_lc = ext.to_ascii_lowercase();

    AUDIO_EXTS.binary_search(&ext_lc.as_str()).is_ok() || VIDEO_EXTS.binary_search(&ext_lc.as_str()).is_ok()
}

/// Clasificador simple audio/video por extensión.
fn guess_media_type(path: &Path) -> &'static str {
    let ext = match path.extension().and_then(|e| e.to_str()) {
        Some(e) if !e.is_empty() => e,
        _ => return "unknown",
    };

    let ext_lc = ext.to_ascii_lowercase();

    if AUDIO_EXTS.binary_search(&ext_lc.as_str()).is_ok() {
        "audio"
    } else if VIDEO_EXTS.binary_search(&ext_lc.as_str()).is_ok() {
        "video"
    } else {
        "unknown"
    }
}
