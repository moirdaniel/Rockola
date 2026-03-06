//! Acceso a datos (repositorio).

use sqlx::SqlitePool;
use crate::models::{MediaItem, QueueItemResponse, DownloadQueueItem, AdminAuditLogItem};
use uuid::Uuid;

pub async fn get_queue(pool: &SqlitePool) -> Result<Vec<QueueItemResponse>, sqlx::Error> {
    let rows = sqlx::query_as::<_, (String, String, String, String, Option<String>, Option<String>, i64, Option<String>, String, Option<String>, i64, String, Option<String>, Option<String>)>(
        r#"
        SELECT queue_id, media_id, source, title, artist, album, duration_seconds, thumbnail_url, type, stream_id, "order", added_at,
               download_id, (SELECT status FROM downloads_queue WHERE id = play_queue.download_id) AS download_status
        FROM play_queue
        WHERE played_at IS NULL
        ORDER BY "order" ASC
        "#
    )
    .fetch_all(pool)
    .await?;

    let items = rows
        .into_iter()
        .map(|(queue_id, id, source, title, artist, album, duration_seconds, thumbnail_url, media_type, stream_id, order, added_at, download_id, download_status)| {
            QueueItemResponse {
                queue_id,
                added_at,
                order,
                id,
                source,
                title,
                artist,
                album,
                duration_seconds,
                thumbnail_url,
                media_type,
                stream_id,
                download_id,
                download_status,
            }
        })
        .collect();
    Ok(items)
}

pub async fn add_to_queue(pool: &SqlitePool, media: &MediaItem, download_id: Option<&str>) -> Result<Vec<QueueItemResponse>, sqlx::Error> {
    let max_order: Option<(i64,)> = sqlx::query_as("SELECT COALESCE(MAX(\"order\"), 0) FROM play_queue WHERE played_at IS NULL")
        .fetch_optional(pool)
        .await?;
    let next_order = max_order.map(|(o,)| o + 1).unwrap_or(1);
    let queue_id = Uuid::new_v4().to_string();

    sqlx::query(
        r#"
        INSERT INTO play_queue (queue_id, media_id, source, title, artist, album, duration_seconds, thumbnail_url, type, stream_id, "order", added_at, download_id)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, datetime('now'), ?)
        "#
    )
    .bind(&queue_id)
    .bind(&media.id)
    .bind(&media.source)
    .bind(&media.title)
    .bind(&media.artist)
    .bind(&media.album)
    .bind(media.duration_seconds)
    .bind(&media.thumbnail_url)
    .bind(&media.media_type)
    .bind(&media.stream_id)
    .bind(next_order)
    .bind(download_id)
    .execute(pool)
    .await?;

    get_queue(pool).await
}

/// Marca el primer item como reproducido y devuelve la cola actualizada.
pub async fn mark_next_played(pool: &SqlitePool) -> Result<Vec<QueueItemResponse>, sqlx::Error> {
    sqlx::query(
        "UPDATE play_queue SET played_at = datetime('now') WHERE id IN (SELECT id FROM play_queue WHERE played_at IS NULL ORDER BY \"order\" ASC LIMIT 1)"
    )
    .execute(pool)
    .await?;
    get_queue(pool).await
}

pub async fn clear_queue(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM play_queue WHERE played_at IS NULL").execute(pool).await?;
    Ok(())
}

/// Resetea toda la data de la aplicación (cola, cache, biblioteca, playlists y créditos).
pub async fn reset_all(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    let mut tx = pool.begin().await?;

    // Vaciar tablas principales
    sqlx::query("DELETE FROM play_queue").execute(&mut *tx).await?;
    sqlx::query("DELETE FROM media_cache").execute(&mut *tx).await?;
    sqlx::query("DELETE FROM media_library").execute(&mut *tx).await?;
    sqlx::query("DELETE FROM playlists").execute(&mut *tx).await?;

    // Resetear créditos del usuario por defecto
    sqlx::query(
        "UPDATE user_credits SET balance = 1000, updated_at = datetime('now') WHERE id = 'default'",
    )
    .execute(&mut *tx)
    .await?;

    tx.commit().await
}

#[derive(Debug)]
pub struct UserCreditsRow {
    pub id: String,
    pub balance: i64,
    pub updated_at: String,
}

pub async fn get_credits(pool: &SqlitePool) -> Result<UserCreditsRow, sqlx::Error> {
    let row = sqlx::query_as::<_, (String, i64, String)>(
        "SELECT id, balance, updated_at FROM user_credits WHERE id = 'default'"
    )
    .fetch_optional(pool)
    .await?;

    let (id, balance, updated_at) = row.ok_or_else(|| sqlx::Error::RowNotFound)?;
    Ok(UserCreditsRow { id, balance, updated_at })
}

/// Asegura que exista la fila de créditos del usuario 'default' (por si la migración no la insertó).
pub async fn ensure_default_credits(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT OR IGNORE INTO user_credits (id, balance, updated_at) VALUES ('default', 1000, datetime('now'))",
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn add_credits(pool: &SqlitePool, amount: i64) -> Result<UserCreditsRow, sqlx::Error> {
    sqlx::query("UPDATE user_credits SET balance = balance + ?, updated_at = datetime('now') WHERE id = 'default'")
        .bind(amount)
        .execute(pool)
        .await?;
    get_credits(pool).await
}

pub async fn deduct_credits(pool: &SqlitePool, amount: i64) -> Result<bool, sqlx::Error> {
    let r = sqlx::query("UPDATE user_credits SET balance = balance - ?, updated_at = datetime('now') WHERE id = 'default' AND balance >= ?")
        .bind(amount)
        .bind(amount)
        .execute(pool)
        .await?;
    Ok(r.rows_affected() > 0)
}

// ---------- media_library ----------

#[derive(Debug)]
pub struct MediaLibraryRow {
    pub id: String,
    pub title: String,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub source: String,
    pub local_path: Option<String>,
    pub duration_seconds: Option<i64>,
    pub thumbnail_url: Option<String>,
    pub media_type: String,
    pub external_id: Option<String>,
}

pub async fn search_media_library(pool: &SqlitePool, query: &str) -> Result<Vec<MediaLibraryRow>, sqlx::Error> {
    let pattern = format!("%{}%", query.trim());
    let rows = sqlx::query_as::<_, (String, String, Option<String>, Option<String>, String, Option<String>, Option<i64>, Option<String>, String, Option<String>)>(
        r#"
        SELECT id, title, artist, album, source, local_path, duration_seconds, thumbnail_url, media_type, external_id
        FROM media_library
        WHERE (title LIKE ?1 OR artist LIKE ?1 OR album LIKE ?1) AND local_path IS NOT NULL
        ORDER BY title ASC
        LIMIT 20
        "#
    )
    .bind(&pattern)
    .fetch_all(pool)
    .await?;

    Ok(rows.into_iter().map(|(id, title, artist, album, source, local_path, duration_seconds, thumbnail_url, media_type, external_id)| {
        MediaLibraryRow {
            id,
            title,
            artist,
            album,
            source,
            local_path,
            duration_seconds,
            thumbnail_url,
            media_type,
            external_id,
        }
    }).collect())
}

pub async fn get_media_by_id(pool: &SqlitePool, id: &str) -> Result<Option<MediaLibraryRow>, sqlx::Error> {
    let row = sqlx::query_as::<_, (String, String, Option<String>, Option<String>, String, Option<String>, Option<i64>, Option<String>, String, Option<String>)>(
        "SELECT id, title, artist, album, source, local_path, duration_seconds, thumbnail_url, media_type, external_id FROM media_library WHERE id = ?"
    )
    .bind(id)
    .fetch_optional(pool)
    .await?;

    Ok(row.map(|(id, title, artist, album, source, local_path, duration_seconds, thumbnail_url, media_type, external_id)| {
        MediaLibraryRow {
            id,
            title,
            artist,
            album,
            source,
            local_path,
            duration_seconds,
            thumbnail_url,
            media_type,
            external_id,
        }
    }))
}

pub async fn get_media_by_external_id(pool: &SqlitePool, external_id: &str) -> Result<Option<MediaLibraryRow>, sqlx::Error> {
    let row = sqlx::query_as::<_, (String, String, Option<String>, Option<String>, String, Option<String>, Option<i64>, Option<String>, String, Option<String>)>(
        "SELECT id, title, artist, album, source, local_path, duration_seconds, thumbnail_url, media_type, external_id FROM media_library WHERE external_id = ? AND local_path IS NOT NULL"
    )
    .bind(external_id)
    .fetch_optional(pool)
    .await?;

    Ok(row.map(|(id, title, artist, album, source, local_path, duration_seconds, thumbnail_url, media_type, external_id)| {
        MediaLibraryRow {
            id,
            title,
            artist,
            album,
            source,
            local_path,
            duration_seconds,
            thumbnail_url,
            media_type,
            external_id,
        }
    }))
}

#[allow(clippy::too_many_arguments)]
pub async fn insert_media_library(
    pool: &SqlitePool,
    id: &str,
    title: &str,
    artist: Option<&str>,
    album: Option<&str>,
    source: &str,
    local_path: &str,
    duration_seconds: Option<i64>,
    thumbnail_url: Option<&str>,
    media_type: &str,
    external_id: Option<&str>,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO media_library (id, title, artist, album, source, local_path, duration_seconds, thumbnail_url, media_type, external_id)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#
    )
    .bind(id)
    .bind(title)
    .bind(artist)
    .bind(album)
    .bind(source)
    .bind(local_path)
    .bind(duration_seconds)
    .bind(thumbnail_url)
    .bind(media_type)
    .bind(external_id)
    .execute(pool)
    .await?;
    Ok(())
}

/// Lista entradas recientes de media_library (para vista mantenedores).
pub async fn list_media_library_recent(pool: &SqlitePool, limit: i64) -> Result<Vec<MediaLibraryRow>, sqlx::Error> {
    let rows = sqlx::query_as::<_, (String, String, Option<String>, Option<String>, String, Option<String>, Option<i64>, Option<String>, String, Option<String>)>(
        r#"
        SELECT id, title, artist, album, source, local_path, duration_seconds, thumbnail_url, media_type, external_id
        FROM media_library
        ORDER BY rowid DESC
        LIMIT ?
        "#
    )
    .bind(limit)
    .fetch_all(pool)
    .await?;

    Ok(rows.into_iter().map(|(id, title, artist, album, source, local_path, duration_seconds, thumbnail_url, media_type, external_id)| {
        MediaLibraryRow {
            id,
            title,
            artist,
            album,
            source,
            local_path,
            duration_seconds,
            thumbnail_url,
            media_type,
            external_id,
        }
    }).collect())
}

/// Conteos para vista de mantenedores.
pub async fn get_maintenance_counts(pool: &SqlitePool) -> Result<(i64, i64, i64), sqlx::Error> {
    let queue: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM play_queue WHERE played_at IS NULL")
        .fetch_one(pool)
        .await?;
    let library: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM media_library")
        .fetch_one(pool)
        .await?;
    let cache: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM media_cache")
        .fetch_one(pool)
        .await?;
    Ok((queue.0, library.0, cache.0))
}

// ---------- downloads_queue ----------

#[derive(Debug)]
pub struct DownloadQueueRow {
    pub id: String,
    pub youtube_video_id: String,
    pub requested_media_type: String,
    pub status: String,
    pub progress: Option<f64>,
    pub target_path: Option<String>,
    pub error_message: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

pub async fn insert_download_job(
    pool: &SqlitePool,
    youtube_video_id: &str,
    requested_media_type: &str,
) -> Result<DownloadQueueRow, sqlx::Error> {
    let id = Uuid::new_v4().to_string();
    sqlx::query(
        r#"
        INSERT INTO downloads_queue (id, youtube_video_id, requested_media_type, status, created_at, updated_at)
        VALUES (?, ?, ?, 'queued', datetime('now'), datetime('now'))
        "#,
    )
    .bind(&id)
    .bind(youtube_video_id)
    .bind(requested_media_type)
    .execute(pool)
    .await?;

    get_download_by_id(pool, &id).await?.ok_or(sqlx::Error::RowNotFound)
}

pub async fn get_download_by_id(
    pool: &SqlitePool,
    id: &str,
) -> Result<Option<DownloadQueueRow>, sqlx::Error> {
    let row = sqlx::query_as::<_, (String, String, String, String, Option<f64>, Option<String>, Option<String>, String, String)>(
        r#"
        SELECT id, youtube_video_id, requested_media_type, status, progress, target_path, error_message, created_at, updated_at
        FROM downloads_queue
        WHERE id = ?
        "#
    )
    .bind(id)
    .fetch_optional(pool)
    .await?;

    Ok(row.map(
        |(id, youtube_video_id, requested_media_type, status, progress, target_path, error_message, created_at, updated_at)| {
            DownloadQueueRow {
                id,
                youtube_video_id,
                requested_media_type,
                status,
                progress,
                target_path,
                error_message,
                created_at,
                updated_at,
            }
        },
    ))
}

pub async fn list_downloads(
    pool: &SqlitePool,
    limit: i64,
) -> Result<Vec<DownloadQueueItem>, sqlx::Error> {
    let rows = sqlx::query_as::<_, (String, String, String, String, Option<f64>, Option<String>, Option<String>, String, String)>(
        r#"
        SELECT id, youtube_video_id, requested_media_type, status, progress, target_path, error_message, created_at, updated_at
        FROM downloads_queue
        ORDER BY created_at DESC
        LIMIT ?
        "#
    )
    .bind(limit)
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(
            |(id, youtube_video_id, requested_media_type, status, progress, target_path, error_message, created_at, updated_at)| {
                DownloadQueueItem {
                    id,
                    youtube_video_id,
                    requested_media_type,
                    status,
                    progress,
                    target_path,
                    error_message,
                    created_at,
                    updated_at,
                }
            },
        )
        .collect())
}

pub async fn update_download_status(
    pool: &SqlitePool,
    id: &str,
    status: &str,
    progress: Option<f64>,
    target_path: Option<&str>,
    error_message: Option<&str>,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        UPDATE downloads_queue
        SET status = ?, progress = ?, target_path = COALESCE(?, target_path),
            error_message = COALESCE(?, error_message),
            updated_at = datetime('now')
        WHERE id = ?
        "#,
    )
    .bind(status)
    .bind(progress)
    .bind(target_path)
    .bind(error_message)
    .bind(id)
    .execute(pool)
    .await?;
    Ok(())
}

/// Actualiza stream_id del ítem de cola vinculado a este download (cuando la descarga termina).
pub async fn update_queue_stream_by_download_id(
    pool: &SqlitePool,
    download_id: &str,
    stream_id: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE play_queue SET stream_id = ?, download_id = NULL WHERE download_id = ?",
    )
    .bind(stream_id)
    .bind(download_id)
    .execute(pool)
    .await?;
    Ok(())
}

/// Obtiene el siguiente job en cola (status = 'queued') para procesar.
pub async fn get_next_download_job(pool: &SqlitePool) -> Result<Option<DownloadQueueRow>, sqlx::Error> {
    let row = sqlx::query_as::<_, (String, String, String, String, Option<f64>, Option<String>, Option<String>, String, String)>(
        r#"
        SELECT id, youtube_video_id, requested_media_type, status, progress, target_path, error_message, created_at, updated_at
        FROM downloads_queue
        WHERE status = 'queued'
        ORDER BY created_at ASC
        LIMIT 1
        "#
    )
    .fetch_optional(pool)
    .await?;

    Ok(row.map(
        |(id, youtube_video_id, requested_media_type, status, progress, target_path, error_message, created_at, updated_at)| {
            DownloadQueueRow {
                id,
                youtube_video_id,
                requested_media_type,
                status,
                progress,
                target_path,
                error_message,
                created_at,
                updated_at,
            }
        },
    ))
}

// ---------- admin_audit_log ----------

pub async fn insert_admin_audit_log(
    pool: &SqlitePool,
    action: &str,
    entity_type: Option<&str>,
    entity_id: Option<&str>,
    payload_json: Option<&str>,
) -> Result<(), sqlx::Error> {
    let id = Uuid::new_v4().to_string();
    sqlx::query(
        r#"
        INSERT INTO admin_audit_log (id, action, entity_type, entity_id, payload_json, created_at)
        VALUES (?, ?, ?, ?, ?, datetime('now'))
        "#,
    )
    .bind(id)
    .bind(action)
    .bind(entity_type)
    .bind(entity_id)
    .bind(payload_json)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn list_admin_audit_log(
    pool: &SqlitePool,
    limit: i64,
) -> Result<Vec<AdminAuditLogItem>, sqlx::Error> {
    let rows = sqlx::query_as::<_, (String, String, Option<String>, Option<String>, Option<String>, String)>(
        r#"
        SELECT id, action, entity_type, entity_id, payload_json, created_at
        FROM admin_audit_log
        ORDER BY created_at DESC
        LIMIT ?
        "#
    )
    .bind(limit)
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|(id, action, entity_type, entity_id, payload_json, created_at)| AdminAuditLogItem {
            id,
            action,
            entity_type,
            entity_id,
            payload_json,
            created_at,
        })
        .collect())
}

// ---------- settings ----------

/// Obtiene el valor de una clave de settings, si existe.
pub async fn get_setting(pool: &SqlitePool, key: &str) -> Result<Option<String>, sqlx::Error> {
    let row = sqlx::query_as::<_, (String,)>("SELECT value FROM settings WHERE key = ?")
        .bind(key)
        .fetch_optional(pool)
        .await?;
    Ok(row.map(|(v,)| v))
}

/// Crea o actualiza una clave en settings.
pub async fn set_setting(pool: &SqlitePool, key: &str, value: &str) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO settings (key, value, updated_at) VALUES (?, ?, datetime('now'))
        ON CONFLICT(key) DO UPDATE SET value = excluded.value, updated_at = datetime('now')
        "#,
    )
    .bind(key)
    .bind(value)
    .execute(pool)
    .await?;
    Ok(())
}
