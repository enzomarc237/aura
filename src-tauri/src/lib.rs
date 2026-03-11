pub mod commands;
pub mod database;
pub mod error;
pub mod indexer;
pub mod intent;
pub mod search;
#[cfg(test)]
mod tests;

use tauri::{Emitter, Manager};

/// Entry point for the Tauri application library.
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            // System-tray icon so the user can show/hide the window after pressing Escape.
            let icon = app
                .default_window_icon()
                .cloned()
                .expect("app icon must be configured in tauri.conf.json");
            let handle = app.handle().clone();
            let tray = tauri::tray::TrayIconBuilder::new()
                .icon(icon)
                .tooltip("Aura – click to show / hide")
                .on_tray_icon_event(move |_tray, event| {
                    if let tauri::tray::TrayIconEvent::Click { .. } = event {
                        if let Some(win) = handle.get_webview_window("main") {
                            if win.is_visible().unwrap_or(false) {
                                let _ = win.hide();
                            } else {
                                let _ = win.show();
                                let _ = win.set_focus();
                            }
                        }
                    }
                })
                .build(app)?;
            // Keep the tray icon alive for the lifetime of the app.
            app.manage(tray);

            // Kick off background indexing with a short delay so the frontend
            // has time to register its event listeners before the completion
            // event is emitted.
            let handle2 = app.handle().clone();
            std::thread::spawn(move || {
                std::thread::sleep(std::time::Duration::from_millis(500));
                match indexer::build_index() {
                    Ok(n) => {
                        let _ = handle2.emit("index_complete", n);
                    }
                    Err(e) => {
                        let _ = handle2.emit("index_error", e.to_string());
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
