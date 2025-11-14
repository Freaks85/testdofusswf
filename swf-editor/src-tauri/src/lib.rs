// SWF Editor - Tauri Application Library
mod core;
mod commands;
mod state;

use state::{AppState, SharedState};
use std::sync::Mutex;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Initialize logger
    env_logger::init();

    // Create application state
    let app_state: SharedState = Mutex::new(AppState::new());

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            commands::open_swf,
            commands::get_resources,
            commands::get_resource_data,
            commands::export_resource,
            commands::decompile_script,
            commands::export_all_resources,
            commands::save_swf,
            commands::get_swf_info,
            commands::get_tags_debug,
            commands::search_swf,
            commands::update_text,
            commands::update_image,
            commands::update_script,
            commands::save_swf_as,
            commands::replace_all_text,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
