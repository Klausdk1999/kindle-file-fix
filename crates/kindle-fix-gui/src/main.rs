#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .invoke_handler(tauri::generate_handler![
            commands::process_files,
            commands::get_supported_languages,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
