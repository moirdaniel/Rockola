//! # core-db
//!
//! Capa de persistencia SQLite usando **rusqlite**.
//!
//! ## Objetivos
//! - Inicializar la DB y aplicar migraciones desde `core/db/migrations/*.sql`.
//! - Proveer helpers de escritura (upsert) para `sources`, `artists`, `media_items`.
//! - Proveer queries mínimas para UI (listar artistas, listar items por artista).
//! - Soporte para búsquedas avanzadas, estadísticas y mantenimiento.
//!
//! ## Notas técnicas
//! - `rusqlite::Connection::transaction()` requiere `&mut Connection`.
//! - Para evitar conflictos de borrow (E0502), se materializan resultados antes de iniciar transacciones.
//!
//! ## Concurrencia
//! - Para MVP: se abre una conexión por operación (simple, confiable).
//! - Más adelante: se puede migrar a pool o a un `Connection` serializado si hace falta.

use rusqlite::{params, Connection, OptionalExtension};
use std::{
    fs,
    path::{Path, PathBuf},
};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DbError {
    #[error("sqlite error: {0}")]
    Sqlite(#[from] rusqlite::Error),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("migration error: {0}")]
    Migration(String),
}

pub type DbResult<T> = Result<T, DbError>;

#[derive(Debug, Clone)]
pub struct Db {
    path: PathBuf,
}

impl Db {
    /// Crea un wrapper DB apuntando a `db_path` (archivo SQLite).
    pub fn new(db_path: impl AsRef<Path>) -> Self {
        Self {
            path: db_path.as_ref().to_path_buf(),
        }
    }

    /// Abre conexión SQLite y habilita FK.
    pub fn connect(&self) -> DbResult<Connection> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent)?;
        }
        let conn = Connection::open(&self.path)?;
        conn.pragma_update(None, "foreign_keys", "ON")?;
        conn.pragma_update(None, "journal_mode", "WAL")?;
        conn.pragma_update(None, "synchronous", "NORMAL")?;
        conn.pragma_update(None, "cache_size", 10000)?;
        Ok(conn)
    }

    /// Aplica migraciones `.sql` ordenadas por nombre (ej: `001_init.sql`).
    ///
    /// Convención:
    /// - `001_init.sql` => version `1`
    /// - `010_xxx.sql`  => version `10`
    ///
    /// Requisitos:
    /// - El manifest raíz del workspace NO debe tener `[dependencies]` (virtual manifest).
    pub fn apply_migrations(
        conn: &mut Connection,
        migrations_dir: impl AsRef<Path>,
    ) -> DbResult<()> {
        conn.execute_batch(
            r#"
            PRAGMA foreign_keys = ON;
            CREATE TABLE IF NOT EXISTS schema_migrations (
              version INTEGER PRIMARY KEY,
              applied_at INTEGER NOT NULL
            );
            "#,
        )?;

        let mut files: Vec<PathBuf> = fs::read_dir(migrations_dir)?
            .filter_map(|e| e.ok().map(|e| e.path()))
            .filter(|p| p.extension().map(|x| x == "sql").unwrap_or(false))
            .collect();

        files.sort();

        for path in files {
            let fname = path.file_name().unwrap().to_string_lossy().to_string();

            let version_str = fname
                .split('_')
                .next()
                .ok_or_else(|| DbError::Migration(format!("invalid migration name: {fname}")))?;

            // "001" -> 1 ; "000" -> 0 (no recomendado, pero soportado)
            let version: i64 = version_str.trim_start_matches('0').parse().unwrap_or(0);

            let already: Option<i64> = conn
                .query_row(
                    "SELECT version FROM schema_migrations WHERE version = ?1",
                    [version],
                    |r| r.get(0),
                )
                .optional()?;

            if already.is_some() {
                continue;
            }

            let sql = fs::read_to_string(&path)?;

            // transacción para migración
            let tx = conn.transaction()?;
            tx.execute_batch(&sql)?;
            tx.execute(
                "INSERT INTO schema_migrations(version, applied_at) VALUES(?1, ?2)",
                params![version, unix_now()],
            )?;
            tx.commit()?;
        }

        Ok(())
    }

    /// Inicializa la base de datos y aplica migraciones.
    pub fn init(&self, migrations_dir: impl AsRef<Path>) -> DbResult<()> {
        let mut conn = self.connect()?;
        Self::apply_migrations(&mut conn, migrations_dir)?;
        Ok(())
    }

    // =========================================================
    // SOURCES
    // =========================================================

    /// Upsert de source por `root_path`. Retorna `source_id`.
    pub fn upsert_source(conn: &Connection, root_path: &str) -> DbResult<i64> {
        let now = unix_now();
        conn.execute(
            r#"
            INSERT INTO sources(root_path, enabled, last_seen_at, status)
            VALUES(?1, 1, ?2, 'ok')
            ON CONFLICT(root_path) DO UPDATE SET
              enabled=1,
              last_seen_at=excluded.last_seen_at,
              status='ok'
            "#,
            params![root_path, now],
        )?;

        let id: i64 = conn.query_row(
            "SELECT id FROM sources WHERE root_path = ?1",
            [root_path],
            |r| r.get(0),
        )?;
        Ok(id)
    }

    /// Marca el final de un scan (timestamps + status ok).
    pub fn mark_source_scan_done(conn: &Connection, source_id: i64) -> DbResult<()> {
        let now = unix_now();
        conn.execute(
            "UPDATE sources SET last_scan_at=?1, last_seen_at=?1, status='ok' WHERE id=?2",
            params![now, source_id],
        )?;
        Ok(())
    }

    /// Obtiene información detallada de una fuente
    pub fn get_source_info(conn: &Connection, source_id: i64) -> DbResult<Option<(String, String, Option<i64>, String)>> {
        let mut stmt = conn.prepare(
            "SELECT root_path, status, last_scan_at, last_seen_at FROM sources WHERE id = ?1"
        )?;
        let result = stmt.query_row([source_id], |row| {
            Ok((
                row.get(0)?,
                row.get(1)?,
                row.get(2)?,
                row.get::<_, Option<i64>>(3)?.unwrap_or_default(),
            ))
        }).optional()?;
        Ok(result)
    }

    /// Lista todas las fuentes
    pub fn list_sources(conn: &Connection) -> DbResult<Vec<(i64, String, String, Option<i64>)>> {
        let mut stmt = conn.prepare(
            "SELECT id, root_path, status, last_scan_at FROM sources ORDER BY root_path"
        )?;
        let rows = stmt.query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)))?;
        Ok(rows.filter_map(Result::ok).collect())
    }

    // =========================================================
    // ARTISTS
    // =========================================================

    /// Upsert de artista por `artist_key`. Retorna `artist_id`.
    pub fn upsert_artist(conn: &Connection, display_name: &str, artist_key: &str) -> DbResult<i64> {
        conn.execute(
            r#"
            INSERT INTO artists(display_name, artist_key)
            VALUES(?1, ?2)
            ON CONFLICT(artist_key) DO UPDATE SET
              display_name=excluded.display_name
            "#,
            params![display_name, artist_key],
        )?;

        let id: i64 = conn.query_row(
            "SELECT id FROM artists WHERE artist_key = ?1",
            [artist_key],
            |r| r.get(0),
        )?;
        Ok(id)
    }

    /// Obtiene estadísticas de un artista
    pub fn get_artist_stats(conn: &Connection, artist_id: i64) -> DbResult<(i64, i64, i64)> {
        let mut stmt = conn.prepare(
            r#"
            SELECT 
                COUNT(*) as total_items,
                SUM(CASE WHEN media_type = 'audio' THEN 1 ELSE 0 END) as audio_count,
                SUM(CASE WHEN media_type = 'video' THEN 1 ELSE 0 END) as video_count
            FROM media_items 
            WHERE artist_id = ?1 AND status = 'active'
            "#
        )?;
        let (total_items, audio_count, video_count): (i64, i64, i64) = stmt.query_row([artist_id], |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?))
        })?;
        Ok((total_items, audio_count, video_count))
    }

    // =========================================================
    // MEDIA ITEMS
    // =========================================================

    /// Upsert de media item. `full_path` es UNIQUE.
    #[allow(clippy::too_many_arguments)]
    pub fn upsert_media_item(
        conn: &Connection,
        source_id: i64,
        artist_id: i64,
        full_path: &str,
        relative_path: &str,
        title: &str,
        media_type: &str,      // audio|video|unknown
        duration_ms: Option<i64>,
        size_bytes: i64,
        mtime_unix: i64,
        playable: &str,        // unknown|playable|not_playable
    ) -> DbResult<i64> {
        let now = unix_now();
        conn.execute(
            r#"
            INSERT INTO media_items(
              source_id, artist_id, full_path, relative_path, title,
              media_type, duration_ms, size_bytes, mtime_unix,
              playable, status, created_at, updated_at
            )
            VALUES(?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, 'active', ?11, ?11)
            ON CONFLICT(full_path) DO UPDATE SET
              source_id=excluded.source_id,
              artist_id=excluded.artist_id,
              relative_path=excluded.relative_path,
              title=excluded.title,
              media_type=excluded.media_type,
              duration_ms=excluded.duration_ms,
              size_bytes=excluded.size_bytes,
              mtime_unix=excluded.mtime_unix,
              playable=excluded.playable,
              status='active',
              updated_at=excluded.updated_at
            "#,
            params![
                source_id,
                artist_id,
                full_path,
                relative_path,
                title,
                media_type,
                duration_ms,
                size_bytes,
                mtime_unix,
                playable,
                now
            ],
        )?;

        let id: i64 = conn.query_row(
            "SELECT id FROM media_items WHERE full_path = ?1",
            [full_path],
            |r| r.get(0),
        )?;
        Ok(id)
    }

    /// Marca como `missing` todo item activo del `source_id` que no esté en `seen_paths`.
    ///
    /// Importante: para evitar E0502, materializamos primero los `active_paths` y luego abrimos transacción.
    pub fn mark_missing_for_source(
        conn: &mut Connection,
        source_id: i64,
        seen_paths: &[String],
    ) -> DbResult<u64> {
        // 1) Materializar active_paths (stmt debe morir antes de iniciar tx)
        let mut stmt = conn.prepare(
            "SELECT full_path FROM media_items WHERE source_id=?1 AND status='active'",
        )?;
        let rows = stmt.query_map([source_id], |r| r.get::<_, String>(0))?;

        let mut active_paths: Vec<String> = Vec::new();
        for r in rows {
            active_paths.push(r?);
        }

        // Fuerza liberar borrow inmutable de `stmt` antes de pedir &mut con transaction()
        drop(stmt);

        // 2) Calcular missing
        let mut missing: Vec<String> = Vec::new();
        for p in active_paths {
            if !seen_paths.iter().any(|x| x == &p) {
                missing.push(p);
            }
        }

        // 3) Aplicar updates en transacción
        let tx = conn.transaction()?;
        let now = unix_now();
        let mut changed: u64 = 0;

        for p in missing {
            let n = tx.execute(
                "UPDATE media_items SET status='missing', updated_at=?1 WHERE full_path=?2",
                params![now, p],
            )?;
            changed += n as u64;
        }

        tx.commit()?;
        Ok(changed)
    }

    /// Busca items por título o artista
    pub fn search_items(conn: &Connection, query: &str) -> DbResult<Vec<(i64, String, String, String, String)>> {
        let search_term = format!("%{}%", query.to_lowercase());
        let mut stmt = conn.prepare(
            r#"
            SELECT 
                mi.id, 
                a.display_name, 
                mi.title, 
                mi.full_path, 
                mi.media_type
            FROM media_items mi
            JOIN artists a ON mi.artist_id = a.id
            WHERE mi.status = 'active'
            AND (
                LOWER(mi.title) LIKE ?1 
                OR LOWER(a.display_name) LIKE ?1
            )
            ORDER BY a.display_name, mi.title
            LIMIT 100
            "#
        )?;
        let rows = stmt.query_map([search_term], |r| {
            Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?))
        })?;

        Ok(rows.filter_map(Result::ok).collect())
    }

    /// Obtiene un item por ID
    pub fn get_item_by_id(conn: &Connection, item_id: i64) -> DbResult<Option<(i64, String, String, String, String, Option<i64>)>> {
        let mut stmt = conn.prepare(
            r#"
            SELECT 
                mi.id,
                a.display_name,
                mi.title,
                mi.full_path,
                mi.media_type,
                mi.duration_ms
            FROM media_items mi
            JOIN artists a ON mi.artist_id = a.id
            WHERE mi.id = ?1
            "#
        )?;
        let result = stmt.query_row([item_id], |row| {
            Ok((
                row.get(0)?,
                row.get(1)?,
                row.get(2)?,
                row.get(3)?,
                row.get(4)?,
                row.get(5)?,
            ))
        }).optional()?;
        Ok(result)
    }

    /// Obtiene estadísticas generales de la biblioteca
    pub fn get_library_stats(conn: &Connection) -> DbResult<(i64, i64, i64, i64, i64)> {
        let mut stmt = conn.prepare(
            r#"
            SELECT 
                COUNT(*) as total_items,
                COUNT(DISTINCT artist_id) as total_artists,
                SUM(CASE WHEN media_type = 'audio' THEN 1 ELSE 0 END) as audio_count,
                SUM(CASE WHEN media_type = 'video' THEN 1 ELSE 0 END) as video_count,
                SUM(size_bytes) as total_size
            FROM media_items 
            WHERE status = 'active'
            "#
        )?;
        let (total_items, total_artists, audio_count, video_count, total_size): (i64, i64, i64, i64, i64) = stmt.query_row([], |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?))
        })?;
        Ok((total_items, total_artists, audio_count, video_count, total_size))
    }

    /// Obtiene los últimos items agregados
    pub fn get_recent_items(conn: &Connection, limit: i64) -> DbResult<Vec<(i64, String, String, String, String)>> {
        let mut stmt = conn.prepare(
            r#"
            SELECT 
                mi.id,
                a.display_name,
                mi.title,
                mi.full_path,
                mi.media_type
            FROM media_items mi
            JOIN artists a ON mi.artist_id = a.id
            WHERE mi.status = 'active'
            ORDER BY mi.created_at DESC
            LIMIT ?1
            "#
        )?;
        let rows = stmt.query_map([limit], |r| {
            Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?))
        })?;

        Ok(rows.filter_map(Result::ok).collect())
    }

    /// Obtiene items aleatorios
    pub fn get_random_items(conn: &Connection, limit: i64) -> DbResult<Vec<(i64, String, String, String, String)>> {
        let mut stmt = conn.prepare(
            r#"
            SELECT 
                mi.id,
                a.display_name,
                mi.title,
                mi.full_path,
                mi.media_type
            FROM media_items mi
            JOIN artists a ON mi.artist_id = a.id
            WHERE mi.status = 'active'
            ORDER BY RANDOM()
            LIMIT ?1
            "#
        )?;
        let rows = stmt.query_map([limit], |r| {
            Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?))
        })?;

        Ok(rows.filter_map(Result::ok).collect())
    }

    // =========================================================
    // MANTENIMIENTO
    // =========================================================

    /// Elimina items marcados como 'missing' después de un período de tiempo
    pub fn cleanup_missing_items(conn: &Connection, days_threshold: i64) -> DbResult<u64> {
        let cutoff_time = unix_now() - (days_threshold * 24 * 60 * 60);
        let deleted_count = conn.execute(
            "DELETE FROM media_items WHERE status = 'missing' AND updated_at < ?1",
            [cutoff_time],
        )?;
        Ok(deleted_count as u64)
    }

    /// Optimiza la base de datos
    pub fn optimize_database(conn: &Connection) -> DbResult<()> {
        conn.execute_batch("VACUUM; ANALYZE;")?;
        Ok(())
    }

    // =========================================================
    // QUERIES MVP PARA UI
    // =========================================================

    /// Lista artistas: (id, display_name, artist_key)
    pub fn list_artists(conn: &Connection) -> DbResult<Vec<(i64, String, String)>> {
        let mut stmt = conn.prepare(
            "SELECT id, display_name, artist_key FROM artists ORDER BY display_name COLLATE NOCASE",
        )?;
        let rows = stmt.query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))?;

        Ok(rows.filter_map(Result::ok).collect())
    }

    /// Lista items activos por artista: (id, title, full_path, media_type)
    pub fn list_items_by_artist(
        conn: &Connection,
        artist_id: i64,
    ) -> DbResult<Vec<(i64, String, String, String)>> {
        let mut stmt = conn.prepare(
            r#"
            SELECT id, title, full_path, media_type
            FROM media_items
            WHERE artist_id=?1 AND status='active'
            ORDER BY title COLLATE NOCASE
            "#,
        )?;
        let rows = stmt.query_map([artist_id], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)))?;

        Ok(rows.filter_map(Result::ok).collect())
    }
}

/// Unix timestamp (seconds).
fn unix_now() -> i64 {
    use time::OffsetDateTime;
    OffsetDateTime::now_utc().unix_timestamp()
}