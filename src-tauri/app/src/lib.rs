pub mod commands;
pub mod error;
pub mod git;
pub mod rebase_editor;
pub mod state;

use std::sync::Arc;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let bridge = Arc::new(rebase_editor::RebaseBridge::default());
    let bridge_for_setup = bridge.clone();
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(state::AppState::default())
        .manage(bridge)
        .setup(move |app| {
            let handle = app.handle().clone();
            let b = bridge_for_setup.clone();
            tauri::async_runtime::spawn(async move {
                if let Err(e) = rebase_editor::start_listener(handle, b).await {
                    eprintln!("rebase bridge listener failed to start: {e}");
                }
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::repo_add,
            commands::repo_refresh,
            commands::repo_open,
            commands::branch_checkout,
            commands::branch_create,
            commands::branch_delete,
            commands::commit,
            commands::push_branch,
            commands::fetch,
            commands::merge,
            commands::merge_abort_cmd,
            commands::merge_continue_cmd,
            commands::pull,
            commands::pull_into_current_rebase,
            commands::rebase_interactive_start,
            commands::rebase_reply,
            commands::rebase_continue_cmd,
            commands::rebase_abort_cmd,
            commands::commit_detail,
            commands::cherry_pick_cmd,
            commands::cherry_pick_continue_cmd,
            commands::cherry_pick_abort_cmd,
            commands::revert_cmd,
            commands::revert_continue_cmd,
            commands::revert_abort_cmd,
            commands::reset_to_commit,
            commands::amend_head_message,
            commands::reword_ancestor,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
