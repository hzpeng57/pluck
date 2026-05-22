pub mod commands;
pub mod error;
pub mod git;
pub mod state;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(state::AppState::default())
        .invoke_handler(tauri::generate_handler![
            commands::repo_add,
            commands::repo_refresh,
            commands::repo_open,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
