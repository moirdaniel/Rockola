#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod updater;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let context = tauri::generate_context!();
    let version = context.package_info().version.to_string();

    tauri::Builder::default()
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_shell::init())
        .manage(updater::UpdaterState::new(version))
        .invoke_handler(tauri::generate_handler![
            updater::updater_check,
            updater::updater_download,
            updater::updater_install,
            updater::updater_get_status,
            updater::updater_get_recovery_status,
            updater::updater_clear_recovery,
            updater::updater_clear_restart_marker,
        ])
        .run(context)
        .expect("error while running Rockola application");
}
