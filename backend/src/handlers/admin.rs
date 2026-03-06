//! Endpoints de administración (login PIN, sesión, audit) y operaciones protegidas.

use axum::{
    extract::State,
    http::{header, HeaderMap, StatusCode},
    response::Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::models::AdminAuditLogItem;
use crate::{repository, AppState};

const SESSION_TTL_SECS: u64 = 15 * 60; // 15 min
const MAX_FAILED_ATTEMPTS: u32 = 5;
const LOCKOUT_SECS: u64 = 5 * 60; // 5 min

/// Obtiene el token Bearer del header Authorization.
fn bearer_token(headers: &HeaderMap) -> Option<String> {
    let v = headers.get(header::AUTHORIZATION)?.to_str().ok()?;
    let t = v.strip_prefix("Bearer ").unwrap_or(v).trim();
    if t.is_empty() {
        None
    } else {
        Some(t.to_string())
    }
}

/// Si admin_pin está configurado, exige sesión válida. Si no, permite.
pub async fn require_admin(
    state: &AppState,
    headers: &HeaderMap,
) -> Result<(), (StatusCode, String)> {
    if state.config.admin_pin.is_none() {
        return Ok(());
    }
    let token = bearer_token(headers).ok_or((
        StatusCode::UNAUTHORIZED,
        "Authorization: Bearer <token> requerido".to_string(),
    ))?;
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let mut sessions = state.admin_sessions.lock().await;
    let expires = *sessions.get(&token).ok_or((
        StatusCode::UNAUTHORIZED,
        "Sesión inválida o expirada".to_string(),
    ))?;
    if now >= expires {
        sessions.remove(&token);
        return Err((
            StatusCode::UNAUTHORIZED,
            "Sesión expirada".to_string(),
        ));
    }
    Ok(())
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AdminLoginRequest {
    pub pin: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AdminLoginResponse {
    pub token: String,
    pub expires_in_secs: u64,
}

/// POST /api/admin/login
pub async fn login(
    State(state): State<AppState>,
    Json(body): Json<AdminLoginRequest>,
) -> Result<Json<AdminLoginResponse>, (StatusCode, String)> {
    let pin = state.config.admin_pin.as_ref().ok_or((
        StatusCode::BAD_REQUEST,
        "Admin auth no configurado (ADMIN_PIN)".to_string(),
    ))?;

    let now_secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    {
        let mut failed = state.admin_failed.lock().await;
        if failed.locked_until_secs > now_secs {
            return Err((
                StatusCode::TOO_MANY_REQUESTS,
                "Demasiados intentos; espera unos minutos".to_string(),
            ));
        }
        if body.pin != *pin {
            failed.count += 1;
            if failed.count >= MAX_FAILED_ATTEMPTS {
                failed.locked_until_secs = now_secs + LOCKOUT_SECS;
            }
            return Err((
                StatusCode::UNAUTHORIZED,
                "PIN incorrecto".to_string(),
            ));
        }
        failed.count = 0;
        failed.locked_until_secs = 0;
    }

    let token = Uuid::new_v4().to_string();
    let expires_at = now_secs + SESSION_TTL_SECS;
    state.admin_sessions.lock().await.insert(token.clone(), expires_at);

    repository::insert_admin_audit_log(&state.pool, "admin.login", None, None, None)
        .await
        .ok();

    Ok(Json(AdminLoginResponse {
        token,
        expires_in_secs: SESSION_TTL_SECS,
    }))
}

/// POST /api/admin/logout
pub async fn logout(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    if state.config.admin_pin.is_none() {
        return Ok(Json(serde_json::json!({ "ok": true })));
    }
    if let Some(token) = bearer_token(&headers) {
        state.admin_sessions.lock().await.remove(&token);
        repository::insert_admin_audit_log(&state.pool, "admin.logout", None, None, None)
            .await
            .ok();
    }
    Ok(Json(serde_json::json!({ "ok": true })))
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AdminSessionResponse {
    pub valid: bool,
}

/// GET /api/admin/session
pub async fn session_status(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<AdminSessionResponse>, (StatusCode, String)> {
    if state.config.admin_pin.is_none() {
        return Ok(Json(AdminSessionResponse { valid: true }));
    }
    let token = bearer_token(&headers).ok_or((
        StatusCode::UNAUTHORIZED,
        "Authorization: Bearer <token> requerido".to_string(),
    ))?;
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let mut sessions = state.admin_sessions.lock().await;
    let valid = sessions
        .get(&token)
        .map(|&exp| now < exp)
        .unwrap_or(false);
    if !valid {
        sessions.remove(&token);
    }
    Ok(Json(AdminSessionResponse { valid }))
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ResetResponse {
    ok: bool,
}

/// POST /api/admin/reset — requiere sesión admin.
pub async fn reset(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<ResetResponse>, (StatusCode, String)> {
    require_admin(&state, &headers).await?;
    repository::reset_all(&state.pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    repository::insert_admin_audit_log(
        &state.pool,
        "admin.reset_all",
        None,
        None,
        Some("{}"),
    )
    .await
    .ok();
    Ok(Json(ResetResponse { ok: true }))
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaLibraryEntryDto {
    pub id: String,
    pub title: String,
    pub artist: Option<String>,
    pub source: String,
    pub local_path: Option<String>,
    pub media_type: String,
    pub external_id: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MaintenanceStatsDto {
    pub queue_count: i64,
    pub media_library_count: i64,
    pub media_cache_count: i64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MaintenanceResponse {
    pub queue: Vec<crate::models::QueueItemResponse>,
    pub media_library: Vec<MediaLibraryEntryDto>,
    pub credits: CreditsDto,
    pub stats: MaintenanceStatsDto,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreditsDto {
    pub id: String,
    pub balance: i64,
    pub updated_at: String,
}

/// GET /api/maintenance — requiere sesión admin.
/// Devuelve cola, biblioteca reciente, créditos y conteos para vista de mantenedores.
pub async fn maintenance(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<MaintenanceResponse>, (StatusCode, String)> {
    require_admin(&state, &headers).await?;
    let pool = &state.pool;
    let queue = repository::get_queue(pool)
        .await
        .map_err(|e| {
            tracing::error!("maintenance get_queue: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
        })?;
    let library_rows = repository::list_media_library_recent(pool, 50)
        .await
        .map_err(|e| {
            tracing::error!("maintenance list_media_library_recent: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
        })?;
    let credits = repository::get_credits(pool)
        .await
        .map_err(|e| {
            tracing::error!("maintenance get_credits: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
        })?;
    let (queue_count, library_count, cache_count) = repository::get_maintenance_counts(pool)
        .await
        .map_err(|e| {
            tracing::error!("maintenance get_maintenance_counts: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
        })?;

    let media_library: Vec<MediaLibraryEntryDto> = library_rows
        .into_iter()
        .map(|r| MediaLibraryEntryDto {
            id: r.id,
            title: r.title,
            artist: r.artist,
            source: r.source,
            local_path: r.local_path,
            media_type: r.media_type,
            external_id: r.external_id,
        })
        .collect();

    Ok(Json(MaintenanceResponse {
        queue,
        media_library,
        credits: CreditsDto {
            id: credits.id,
            balance: credits.balance,
            updated_at: credits.updated_at,
        },
        stats: MaintenanceStatsDto {
            queue_count,
            media_library_count: library_count,
            media_cache_count: cache_count,
        },
    }))
}

/// GET /api/admin/audit-log — lista el registro de auditoría. Requiere sesión admin.
pub async fn audit_log(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<Vec<AdminAuditLogItem>>, (StatusCode, String)> {
    require_admin(&state, &headers).await?;
    let list = repository::list_admin_audit_log(&state.pool, 200)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(list))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuditCreateRequest {
    pub action: String,
    pub entity_type: Option<String>,
    pub entity_id: Option<String>,
    pub payload_json: Option<String>,
}

/// POST /api/admin/audit — registra una entrada en el audit log (p. ej. actualizaciones). Requiere sesión admin.
pub async fn audit_create(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<AuditCreateRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    require_admin(&state, &headers).await?;
    repository::insert_admin_audit_log(
        &state.pool,
        &body.action,
        body.entity_type.as_deref(),
        body.entity_id.as_deref(),
        body.payload_json.as_deref(),
    )
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(serde_json::json!({ "ok": true })))
}

// ---------- Settings de actualizaciones (solo admin) ----------

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateSettingsDto {
    pub enabled: bool,
    pub channel: String,
    pub auto_check: bool,
    pub check_interval_minutes: u32,
    pub endpoint_override: Option<String>,
}

/// GET /api/admin/settings/updates — devuelve la configuración de actualizaciones. Requiere sesión admin.
pub async fn get_update_settings(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<UpdateSettingsDto>, (StatusCode, String)> {
    require_admin(&state, &headers).await?;
    let pool = &state.pool;
    let enabled = repository::get_setting(pool, "updates.enabled")
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .unwrap_or_else(|| "true".to_string());
    let channel = repository::get_setting(pool, "updates.channel")
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .unwrap_or_else(|| "stable".to_string());
    let auto_check = repository::get_setting(pool, "updates.autoCheck")
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .unwrap_or_else(|| "true".to_string());
    let check_interval_minutes = repository::get_setting(pool, "updates.checkIntervalMinutes")
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .unwrap_or_else(|| "720".to_string());
    let endpoint_override = repository::get_setting(pool, "updates.endpointOverride")
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .filter(|s| !s.is_empty());

    let parse_bool = |s: &str| s.eq_ignore_ascii_case("true") || s == "1";
    let parse_u32 = |s: &str| s.parse::<u32>().unwrap_or(720);

    Ok(Json(UpdateSettingsDto {
        enabled: parse_bool(&enabled),
        channel: if channel.is_empty() { "stable".to_string() } else { channel },
        auto_check: parse_bool(&auto_check),
        check_interval_minutes: parse_u32(&check_interval_minutes),
        endpoint_override,
    }))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateSettingsRequest {
    pub enabled: Option<bool>,
    pub channel: Option<String>,
    pub auto_check: Option<bool>,
    pub check_interval_minutes: Option<u32>,
    pub endpoint_override: Option<String>,
}

/// PUT /api/admin/settings/updates — actualiza la configuración de actualizaciones. Requiere sesión admin.
pub async fn put_update_settings(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<UpdateSettingsRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    require_admin(&state, &headers).await?;
    let pool = &state.pool;

    if let Some(v) = body.enabled {
        repository::set_setting(pool, "updates.enabled", if v { "true" } else { "false" })
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    }
    if let Some(ref c) = body.channel {
        let channel = if c.is_empty() { "stable" } else { c.as_str() };
        if channel == "stable" || channel == "beta" {
            repository::set_setting(pool, "updates.channel", channel)
                .await
                .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
        }
    }
    if let Some(v) = body.auto_check {
        repository::set_setting(pool, "updates.autoCheck", if v { "true" } else { "false" })
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    }
    if let Some(m) = body.check_interval_minutes {
        if m > 0 && m <= 60 * 24 * 7 {
            repository::set_setting(pool, "updates.checkIntervalMinutes", &m.to_string())
                .await
                .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
        }
    }
    if let Some(ref e) = body.endpoint_override {
        repository::set_setting(pool, "updates.endpointOverride", e.trim())
            .await
            .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))?;
    }

    repository::insert_admin_audit_log(
        &state.pool,
        "admin.settings_updates_updated",
        None,
        None,
        None,
    )
    .await
    .ok();

    Ok(Json(serde_json::json!({ "ok": true })))
}

