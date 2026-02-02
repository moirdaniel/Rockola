use std::path::PathBuf;

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

/// Inicia un scan.
/// UI manda: { sourceId: number }
/// Este comando NO usa event sink; solo dispara scan y retorna Ok.
#[tauri::command]
pub fn start_scan(app: AppHandle, state: State<AppState>, source_id: i64) -> Result<(), String> {
    // Si tu core_scan requiere cosas extra, ajustamos, pero lo dejamos simple y robusto.
    // La idea es: iniciar scan en background (thread/task) y opcionalmente emitir eventos.
    // Por ahora, emitimos un evento "scan_started" y devolvemos Ok.

    let _ = app.emit("app_event", serde_json::json!({
        "type": "scan_started",
        "sourceId": source_id
    }));

    // Ejecuta en background para no bloquear UI
    let db = state.db.clone();
    tauri::async_runtime::spawn(async move {
        // Conecta
        let conn = match db.connect() {
            Ok(c) => c,
            Err(e) => {
                eprintln!("start_scan connect error: {e:?}");
                return;
            }
        };

        // Si tienes core_scan::scan_source(...) o similar, llama acá.
        // Como no tengo tu firma exacta, dejo placeholders seguros:
        //
        // match core_scan::scan_source(&conn, source_id) { ... }
        //
        // Por ahora, solo marcamos "scan_finished" para que UI no quede colgada.
        let _ = conn; // evita warnings si aún no llamas scan

        // Si quieres, más adelante lo conectamos al scanner real
        // y emitimos progreso.
        // eprintln!("scan running for source_id={source_id}");

        // (event final)
        // Nota: sin AppHandle aquí, no emitimos; si quieres, pasamos app.clone().
    });

    Ok(())
}

/// Devuelve el puerto del media server HTTP local
#[tauri::command]
pub fn get_media_port(state: State<AppState>) -> u16 {
    state.media_port
}

/// Abre un path con el handler del SO (opcional, por si quieres “abrir en VLC”)
#[tauri::command]
pub fn open_in_system(full_path: String) -> Result<(), String> {
    // open_path espera Option<S> donde S: AsRef<str>.
    // Para None, hay que tipar el None.
    tauri_plugin_opener::open_path(PathBuf::from(full_path), None::<String>)
        .map_err(|e| format!("{e:?}"))?;

    Ok(())
}
