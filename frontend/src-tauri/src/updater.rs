//! Módulo de actualización: estado, comandos y eventos.
//! Solo el mantenedor (admin) puede disparar check/download/install desde la UI.
//! Incluye modo seguro (crash-loop recovery) tras actualizaciones.

use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_updater::UpdaterExt;
use url::Url;

const RECOVERY_FILE_NAME: &str = "rockola-update-recovery.json";
const CRASH_LOOP_THRESHOLD: u32 = 3;
const CLEAR_RESTARTS_AFTER_SECS: u64 = 60;

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct RecoveryFileState {
    pub restarts_after_update: u32,
    pub recovery_mode: bool,
}

/// Estado interno del updater.
pub struct UpdaterStateInner {
    pub phase: String,
    pub progress: f64,
    pub last_error: Option<String>,
    pub current_version: String,
    pub available_version: Option<String>,
    pub available_notes: Option<String>,
    pub available_date: Option<String>,
    pub pending_update: Option<tauri_plugin_updater::Update>,
    pub downloaded_bytes: Option<Vec<u8>>,
    /// Solo se ejecuta la lógica de startup recovery una vez por sesión.
    pub startup_recovery_checked: bool,
}

pub struct UpdaterState(pub Arc<tokio::sync::Mutex<UpdaterStateInner>>);

impl UpdaterState {
    pub fn new(current_version: String) -> Self {
        Self(Arc::new(tokio::sync::Mutex::new(UpdaterStateInner {
            phase: "idle".to_string(),
            progress: 0.0,
            last_error: None,
            current_version,
            available_version: None,
            available_notes: None,
            available_date: None,
            pending_update: None,
            downloaded_bytes: None,
            startup_recovery_checked: false,
        })))
    }
}

fn emit(app: &AppHandle, event: &str, payload: Option<serde_json::Value>) {
    let _ = app.emit(
        "update",
        payload.unwrap_or_else(|| serde_json::json!({ "event": event })),
    );
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CheckResult {
    pub has_update: bool,
    pub current_version: String,
    pub available_version: Option<String>,
    pub notes: Option<String>,
    pub pub_date: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProgressResult {
    pub status: String,
    pub progress: Option<f64>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StatusResult {
    pub phase: String,
    pub progress: f64,
    pub last_error: Option<String>,
    pub current_version: String,
    pub available_version: Option<String>,
}

#[tauri::command]
pub async fn updater_check(
    app: AppHandle,
    state: tauri::State<'_, UpdaterState>,
    channel: Option<String>,
    endpoint_override: Option<String>,
) -> Result<CheckResult, String> {
    let current = app.package_info().version.to_string();
    {
        let mut inner = state.0.lock().await;
        inner.phase = "checking".to_string();
        inner.progress = 0.0;
        inner.last_error = None;
    }
    emit(&app, "checking", None);

    let endpoints: Vec<Url> = if let Some(ref url) = endpoint_override {
        let u = url.trim();
        if u.is_empty() {
            build_endpoints(channel.as_deref().unwrap_or("stable"))
        } else {
            vec![u.to_string()]
        }
    } else {
        build_endpoints(channel.as_deref().unwrap_or("stable"))
    }
    .into_iter()
    .filter_map(|s| Url::parse(&s).ok())
    .collect();
    if endpoints.is_empty() {
        return Err("No hay endpoints de actualización configurados".to_string());
    }
    let update = match app
        .updater_builder()
        .endpoints(endpoints)
        .map_err(|e: tauri_plugin_updater::Error| e.to_string())?
        .build()
        .map_err(|e: tauri_plugin_updater::Error| e.to_string())?
        .check()
        .await
        .map_err(|e: tauri_plugin_updater::Error| e.to_string())?
    {
        Some(u) => u,
        None => {
            let mut inner = state.0.lock().await;
            inner.phase = "idle".to_string();
            inner.available_version = None;
            inner.pending_update = None;
            emit(&app, "done", Some(serde_json::json!({ "event": "done", "hasUpdate": false })));
            return Ok(CheckResult {
                has_update: false,
                current_version: current.clone(),
                available_version: None,
                notes: None,
                pub_date: None,
            });
        }
    };

    let version = update.version.clone();
    let notes = update.body.clone();
    let pub_date = update.date.as_ref().map(|d| d.to_string());

    {
        let mut inner = state.0.lock().await;
        inner.phase = "available".to_string();
        inner.available_version = Some(version.clone());
        inner.available_notes = notes.clone();
        inner.available_date = pub_date.clone();
        inner.pending_update = Some(update);
    }
    emit(
        &app,
        "available",
        Some(serde_json::json!({
            "event": "available",
            "version": version,
            "notes": notes,
            "pubDate": pub_date
        })),
    );

    Ok(CheckResult {
        has_update: true,
        current_version: current,
        available_version: Some(version),
        notes,
        pub_date,
    })
}

fn build_endpoints(channel: &str) -> Vec<String> {
    // Endpoints por canal; en producción leer de tauri.conf o settings.
    let base = "https://github.com/USER/REPO/releases";
    if channel == "beta" {
        vec![format!("{}/latest/download/latest-beta.json", base)]
    } else {
        vec![format!("{}/latest/download/latest.json", base)]
    }
}

#[tauri::command]
pub async fn updater_download(
    app: AppHandle,
    state: tauri::State<'_, UpdaterState>,
) -> Result<ProgressResult, String> {
    let update = {
        let mut inner = state.0.lock().await;
        let u = inner
            .pending_update
            .clone()
            .ok_or("No hay actualización pendiente. Ejecuta Buscar actualización primero.")?;
        inner.phase = "downloading".to_string();
        inner.progress = 0.0;
        drop(inner);
        u
    };

    emit(&app, "progress", Some(serde_json::json!({ "event": "progress", "progress": 0 })));

    let downloaded = std::sync::Arc::new(AtomicU64::new(0));
    let total = std::sync::Arc::new(std::sync::atomic::AtomicU64::new(0));
    let app_emit = app.clone();

    let on_chunk = {
        let d = downloaded.clone();
        let t = total.clone();
        let ap = app_emit.clone();
        move |chunk_len: usize, content_length: Option<u64>| {
            if let Some(c) = content_length {
                t.store(c, Ordering::Relaxed);
            }
            d.fetch_add(chunk_len as u64, Ordering::Relaxed);
            let down = d.load(Ordering::Relaxed);
            let tot = t.load(Ordering::Relaxed);
            let progress = if tot > 0 {
                (down as f64 / tot as f64).min(1.0)
            } else {
                0.0
            };
            let _ = ap.emit(
                "update",
                serde_json::json!({ "event": "progress", "progress": progress }),
            );
        }
    };

    let bytes = update
        .download(on_chunk, || {})
        .await
        .map_err(|e: tauri_plugin_updater::Error| e.to_string())?;

    {
        let mut inner = state.0.lock().await;
        inner.phase = "downloaded".to_string();
        inner.progress = 1.0;
        inner.downloaded_bytes = Some(bytes);
    }
    emit(&app, "downloaded", Some(serde_json::json!({ "event": "downloaded" })));

    Ok(ProgressResult {
        status: "downloaded".to_string(),
        progress: Some(1.0),
    })
}

#[tauri::command]
pub async fn updater_install(
    app: AppHandle,
    state: tauri::State<'_, UpdaterState>,
) -> Result<ProgressResult, String> {
    let (update, bytes) = {
        let mut inner = state.0.lock().await;
        let u = inner
            .pending_update
            .clone()
            .ok_or("No hay actualización pendiente.")?;
        let b = inner
            .downloaded_bytes
            .clone()
            .ok_or("No hay archivo descargado. Ejecuta Descargar primero.")?;
        inner.phase = "installing".to_string();
        drop(inner);
        (u, b)
    };

    emit(&app, "installing", None);

    write_install_marker(&app).ok();

    update
        .install(&bytes)
        .map_err(|e: tauri_plugin_updater::Error| e.to_string())?;

    {
        let mut inner = state.0.lock().await;
        inner.phase = "done".to_string();
        inner.pending_update = None;
        inner.downloaded_bytes = None;
    }
    emit(
        &app,
        "done",
        Some(serde_json::json!({ "event": "done", "willRelaunch": true })),
    );

    std::process::exit(0);
}

// ---------- Modo seguro / crash-loop recovery ----------

fn recovery_file_path(app: &AppHandle) -> Result<std::path::PathBuf, String> {
    let dir = app
        .path()
        .app_data_dir()
        .map_err(|e| e.to_string())?;
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    Ok(dir.join(RECOVERY_FILE_NAME))
}

fn read_recovery_state(app: &AppHandle) -> RecoveryFileState {
    let path = match recovery_file_path(app) {
        Ok(p) => p,
        Err(_) => return RecoveryFileState::default(),
    };
    let s = match std::fs::read_to_string(&path) {
        Ok(s) => s,
        Err(_) => return RecoveryFileState::default(),
    };
    serde_json::from_str(&s).unwrap_or_default()
}

fn write_recovery_state(app: &AppHandle, state: &RecoveryFileState) -> Result<(), String> {
    let path = recovery_file_path(app)?;
    let s = serde_json::to_string_pretty(state).map_err(|e| e.to_string())?;
    std::fs::write(&path, s).map_err(|e| e.to_string())?;
    Ok(())
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RecoveryStatusResult {
    pub in_recovery_mode: bool,
    pub restarts_after_update: u32,
}

/// Devuelve el estado de recuperación. La primera vez que se llama en esta sesión
/// ejecuta la lógica de startup: incrementa restarts_after_update, y si >= 3
/// activa recovery_mode. Tras 60s sin crash se puede limpiar el contador.
#[tauri::command]
pub async fn updater_get_recovery_status(
    app: AppHandle,
    state: tauri::State<'_, UpdaterState>,
) -> Result<RecoveryStatusResult, String> {
    let mut run_startup_check = false;
    {
        let mut inner = state.0.lock().await;
        if !inner.startup_recovery_checked {
            inner.startup_recovery_checked = true;
            run_startup_check = true;
        }
    }
    if run_startup_check {
        let mut file_state = read_recovery_state(&app);
        file_state.restarts_after_update = file_state.restarts_after_update.saturating_add(1);
        if file_state.restarts_after_update >= CRASH_LOOP_THRESHOLD {
            file_state.recovery_mode = true;
            file_state.restarts_after_update = 0;
            let _ = app.emit("update", serde_json::json!({ "event": "recovery_mode_activated" }));
        }
        write_recovery_state(&app, &file_state)?;
        if file_state.restarts_after_update > 0 && !file_state.recovery_mode {
            let app_clear = app.clone();
            tokio::spawn(async move {
                tokio::time::sleep(std::time::Duration::from_secs(CLEAR_RESTARTS_AFTER_SECS)).await;
                let mut s = read_recovery_state(&app_clear);
                s.restarts_after_update = 0;
                let _ = write_recovery_state(&app_clear, &s);
            });
        }
    }
    let file_state = read_recovery_state(&app);
    Ok(RecoveryStatusResult {
        in_recovery_mode: file_state.recovery_mode,
        restarts_after_update: file_state.restarts_after_update,
    })
}

/// Limpia el modo recuperación (admin lo desactiva desde la UI).
#[tauri::command]
pub async fn updater_clear_recovery(app: AppHandle) -> Result<(), String> {
    let mut s = read_recovery_state(&app);
    s.recovery_mode = false;
    s.restarts_after_update = 0;
    write_recovery_state(&app, &s)
}

/// Limpia solo el contador de reinicios (tras 60s de ejecución estable).
#[tauri::command]
pub async fn updater_clear_restart_marker(app: AppHandle) -> Result<(), String> {
    let mut s = read_recovery_state(&app);
    s.restarts_after_update = 0;
    write_recovery_state(&app, &s)
}

/// Escribe el marker de "próximo reinicio es tras update" antes de instalar.
fn write_install_marker(app: &AppHandle) -> Result<(), String> {
    let mut s = read_recovery_state(app);
    s.restarts_after_update = 1;
    write_recovery_state(app, &s)
}

#[tauri::command]
pub async fn updater_get_status(
    state: tauri::State<'_, UpdaterState>,
) -> Result<StatusResult, String> {
    let inner = state.0.lock().await;
    Ok(StatusResult {
        phase: inner.phase.clone(),
        progress: inner.progress,
        last_error: inner.last_error.clone(),
        current_version: inner.current_version.clone(),
        available_version: inner.available_version.clone(),
    })
}
