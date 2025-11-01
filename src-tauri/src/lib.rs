mod commands;
mod types;

use commands::AppState;

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .manage(AppState::default())
        .invoke_handler(tauri::generate_handler![
            greet,
            commands::select_folder,
            commands::save_config,
            commands::load_configs,
            commands::delete_config,
            commands::run_backup_now,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
