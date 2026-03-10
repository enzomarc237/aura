pub mod commands;
pub mod database;
pub mod error;
pub mod indexer;
pub mod intent;
pub mod search;
#[cfg(test)]
mod tests;

use tauri::Emitter;

/// Entry point for the Tauri application library.
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_fs::init())
        .setup(|app| {
            // Kick off background indexing after the window is ready.
            let handle = app.handle().clone();
            std::thread::spawn(move || {
                match indexer::build_index() {
                    Ok(n) => {
                        let _ = handle.emit("index_complete", n);
                    }
                    Err(e) => {
                        let _ = handle.emit("index_error", e.to_string());
                    }
                }
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::search,
            commands::execute_action,
            commands::execute_intent,
            commands::get_settings,
            commands::update_setting,
            commands::reindex,
            commands::get_plugins,
            commands::register_plugin,
        ])
        .run(tauri::generate_context!())
        .expect("error running Aura");
}
