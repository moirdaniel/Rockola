#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod media_server;
mod state;

use tauri::Manager;
use core_db::Db;
use crate::state::AppState;

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            // -------------------------
            // App data dir
            // -------------------------
            let data_dir = app
                .path()
                .app_data_dir()
                .expect("no app_data_dir");

            std::fs::create_dir_all(&data_dir).unwrap();

            let db_path = data_dir.join("rockola.db");
            let migrations_dir = data_dir.join("migrations");

            // -------------------------
            // DB (core_db)
            // -------------------------
            let db = Db::new(db_path);
            
            // Inicializar la base de datos con migraciones
            if let Err(e) = db.init(&migrations_dir) {
                eprintln!("Error inicializando la base de datos: {:?}", e);
            }

            // -------------------------
            // Media server HTTP
            // -------------------------
            let media_port =
                tauri::async_runtime::block_on(media_server::start_media_server())
                    .expect("failed to start media server");

            println!("🎬 Media server http://127.0.0.1:{media_port}");

            // -------------------------
            // Global state
            // -------------------------
            app.manage(AppState {
                db,
                migrations_dir,
                media_port,
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::add_source,
            commands::list_artists,
            commands::list_items_by_artist,
            commands::start_scan,
            commands::get_media_port,
            commands::search_items,
            commands::get_library_stats
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}