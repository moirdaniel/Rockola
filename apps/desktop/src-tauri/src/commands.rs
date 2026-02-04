use std::path::PathBuf;
use std::sync::atomic::AtomicBool;
use serde_json::Value; 
use tauri::{AppHandle, Emitter, State};
use crate::state::AppState;


/// Agrega una source (ruta raíz) a la DB.
/// UI manda: { rootPath: string }
#[tauri::command]
pub fn add_source(state: State<AppState>, root_path: String) -> Result<i64, String> {
    let conn = state
        .db
        .connect()
        .map_err(|e| format!("{e:?}"))?;

    // core_db::Db::upsert_source retorna i64 (source_id)
    let source_id = core_db::Db::upsert_source(&conn, &root_path)
        .map_err(|e| format!("{e:?}"))?;

    Ok(source_id)
}

/// Lista artistas.
/// Retorno: Vec<(id, name, cover_path)>
#[tauri::command]
pub fn list_artists(state: State<AppState>) -> Result<Vec<(i64, String, String)>, String> {
    let conn = state
        .db
        .connect()
        .map_err(|e| format!("{e:?}"))?;

    // IMPORTANTE: esto debe retornar Vec<...>
    let rows = core_db::Db::list_artists(&conn)
        .map_err(|e| format!("{e:?}"))?;

    Ok(rows)
}

/// Lista items por artista.
/// Retorno: Vec<(id, title, full_path, media_type)>
#[tauri::command]
pub fn list_items_by_artist(
    state: State<AppState>,
    artist_id: i64,
) -> Result<Vec<(i64, String, String, String)>, String> {
    let conn = state
        .db
        .connect()
        .map_err(|e| format!("{e:?}"))?;

    let rows = core_db::Db::list_items_by_artist(&conn, artist_id)
        .map_err(|e| format!("{e:?}"))?;

    Ok(rows)
}

/// Busca items por título o artista
#[tauri::command]
pub fn search_items(state: State<AppState>, query: String) -> Result<Vec<(i64, String, String, String, String)>, String> {
    let conn = state
        .db
        .connect()
        .map_err(|e| format!("{e:?}"))?;

    let rows = core_db::Db::search_items(&conn, &query)
        .map_err(|e| format!("{e:?}"))?;

    Ok(rows)
}

/// Obtiene estadísticas generales de la biblioteca
#[tauri::command]
pub fn get_library_stats(state: State<AppState>) -> Result<(i64, i64, i64, i64, i64), String> {
    let conn = state
        .db
        .connect()
        .map_err(|e| format!("{e:?}"))?;

    let stats = core_db::Db::get_library_stats(&conn)
        .map_err(|e| format!("{e:?}"))?;

    Ok(stats)
}

/// Inicia un scan.
/// UI manda: { sourceId: number }
#[tauri::command]
pub fn start_scan(app: AppHandle, state: State<AppState>, source_id: i64) -> Result<(), String> {
    let _ = app.emit("app_event", serde_json::json!({
        "type": "scan_started",
        "sourceId": source_id
    }));

    // Obtener la ruta de la fuente
    let source_info = {
        let conn = state.db.connect().map_err(|e| format!("{:?}", e))?;
        core_db::Db::get_source_info(&conn, source_id).map_err(|e| format!("{:?}", e))?
    };

    if let Some((root_path, _, _, _)) = source_info {
        let db = state.db.clone();
        let app_handle = app.clone();
        
        tauri::async_runtime::spawn(async move {
            let cancelled = AtomicBool::new(false);
            let scanner = core_scan::Scanner::new(db);
            
            // Emitir evento de inicio
            let _ = app_handle.emit("app_event", serde_json::json!({
                "type": "scan_progress",
                "payload": {
                    "source_id": source_id,
                    "processed": 0,
                    "total": Value::Null,                 // <-- NULL JSON
                    "phase": "starting",
                    "current_path": "Iniciando escaneo...", // <-- string directo
                    "progress_percent": 0.0                 // <-- number directo
                }
            }));


            if let Err(e) = scanner.scan_source_with_cancel(
                source_id,
                &root_path,
                &EventSinkAdapter::new(app_handle),
                &cancelled
            ) {
                let _ = app_handle.emit("app_event", serde_json::json!({
                    "type": "error",
                    "payload": {
                        "code": "SCAN_ERROR",
                        "message": format!("Error durante el escaneo: {}", e),
                        "context": Value::Null,
                        "timestamp": time::OffsetDateTime::now_utc().unix_timestamp(),
                        "severity": "error"
                    }
                }));
            }

        });
    } else {
        return Err("Source no encontrada".to_string());
    }

    Ok(())
}

/// Devuelve el puerto del media server HTTP local
#[tauri::command]
pub fn get_media_port(state: State<AppState>) -> u16 {
    state.media_port
}

/// Abre un path con el handler del SO (opcional, por si quieres "abrir en VLC")
#[tauri::command]
pub fn open_in_system(full_path: String) -> Result<(), String> {
    // open_path espera Option<S> donde S: AsRef<str>.
    // Para None, hay que tipar el None.
    tauri_plugin_opener::open_path(PathBuf::from(full_path), None::<String>)
        .map_err(|e| format!("{e:?}"))?;

    Ok(())
}

/// Event sink adapter para conectar el scanner con los eventos de Tauri
struct EventSinkAdapter {
    app_handle: AppHandle,
}

impl EventSinkAdapter {
    fn new(app_handle: AppHandle) -> Self {
        Self { app_handle }
    }
}

impl core_scan::EventSink for EventSinkAdapter {
    fn emit(&self, event: core_events::AppEvent) {
        let _ = self.app_handle.emit("app_event", event);
    }
}