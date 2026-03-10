use rusqlite::OptionalExtension;
use serde::{Deserialize, Serialize};
use tauri::command;

use crate::database::{boost_item_rank, get_setting, record_history, set_setting, DB};
use crate::error::AuraError;
use crate::indexer::build_index;
use crate::intent::parse_intent;
use crate::search::{fuzzy_search, SearchResult};

/// Response type for search queries.
#[derive(Debug, Serialize, Deserialize)]
pub struct SearchResponse {
    pub results: Vec<SearchResult>,
    pub intent: Option<crate::intent::Intent>,
    pub query: String,
}

/// Response type for settings.
#[derive(Debug, Serialize, Deserialize)]
pub struct SettingsResponse {
    pub hotkey: String,
    pub theme: String,
    pub max_results: u32,
}

/// Perform a fuzzy search and optional intent classification.
#[command]
pub fn search(query: String) -> Result<SearchResponse, AuraError> {
    let max_results = get_setting("max_results")?
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or(10);

    let results = fuzzy_search(&query, max_results)?;
    let intent = parse_intent(&query);

    Ok(SearchResponse {
        results,
        intent,
        query,
    })
}

/// Execute an action identified by item ID.
#[command]
pub fn execute_action(id: i64, query: String) -> Result<(), AuraError> {
    // Record history
    let _ = record_history(&query, Some(id));
    // Boost rank of selected item
    let _ = boost_item_rank(id);

    // Look up the item path from the DB
    let path = {
        let conn = DB.lock().map_err(|_| AuraError::Search("db lock".into()))?;
        let mut stmt = conn
            .prepare("SELECT path, kind FROM search_index WHERE id = ?1")
            .map_err(AuraError::Database)?;
        let result: Option<(String, String)> = stmt
            .query_row(rusqlite::params![id], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
            })
            .optional()
            .map_err(AuraError::Database)?;
        result
    };

    if let Some((p, _kind)) = path {
        open_path(&p)?;
    }

    Ok(())
}

/// Execute a raw system path (open file/app).
fn open_path(path: &str) -> Result<(), AuraError> {
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(path)
            .spawn()
            .map_err(AuraError::Io)?;
    }
    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(path)
            .spawn()
            .map_err(AuraError::Io)?;
    }
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("explorer")
            .arg(path)
            .spawn()
            .map_err(AuraError::Io)?;
    }
    Ok(())
}

/// Execute an intent action (system commands, etc.)
#[command]
pub fn execute_intent(action: String, payload: serde_json::Value) -> Result<(), AuraError> {
    match action.as_str() {
        "open_mail" => {
            let recipient = payload["recipient"].as_str().unwrap_or("");
            let url = format!("mailto:{}", recipient);
            open_path(&url)?;
        }
        "open_facetime" => {
            let contact = payload["contact"].as_str().unwrap_or("");
            let url = format!("facetime:{}", contact);
            open_path(&url)?;
        }
        "open_browser" => {
            let url = payload["url"].as_str().unwrap_or("https://google.com");
            open_path(url)?;
        }
        "open_app" => {
            let name = payload["name"].as_str().unwrap_or("");
            open_path(name)?;
        }
        #[cfg(target_os = "macos")]
        "sleep" => {
            std::process::Command::new("pmset")
                .args(["sleepnow"])
                .spawn()
                .map_err(AuraError::Io)?;
        }
        #[cfg(target_os = "macos")]
        "empty_trash" => {
            std::process::Command::new("osascript")
                .args(["-e", "tell app \"Finder\" to empty trash"])
                .spawn()
                .map_err(AuraError::Io)?;
        }
        #[cfg(target_os = "macos")]
        "set_volume" => {
            let v = payload["volume"].as_u64().unwrap_or(50);
            std::process::Command::new("osascript")
                .args(["-e", &format!("set volume output volume {}", v)])
                .spawn()
                .map_err(AuraError::Io)?;
        }
        _ => {}
    }
    Ok(())
}

/// Retrieve app settings.
#[command]
pub fn get_settings() -> Result<SettingsResponse, AuraError> {
    Ok(SettingsResponse {
        hotkey: get_setting("hotkey")?.unwrap_or_else(|| "CmdOrCtrl+Space".into()),
        theme: get_setting("theme")?.unwrap_or_else(|| "dark".into()),
        max_results: get_setting("max_results")?
            .and_then(|v| v.parse().ok())
            .unwrap_or(10),
    })
}

/// Update a single setting.
#[command]
pub fn update_setting(key: String, value: String) -> Result<(), AuraError> {
    set_setting(&key, &value)
}

/// Trigger a re-index of the filesystem.
#[command]
pub fn reindex() -> Result<usize, AuraError> {
    build_index()
}

/// Retrieve all registered plugins.
#[command]
pub fn get_plugins() -> Result<Vec<serde_json::Value>, AuraError> {
    let conn = DB.lock().map_err(|_| AuraError::Search("db lock".into()))?;
    let mut stmt = conn
        .prepare("SELECT id, name, script_path, enabled FROM plugins")
        .map_err(AuraError::Database)?;
    let plugins: Vec<serde_json::Value> = stmt
        .query_map([], |row| {
            Ok(serde_json::json!({
                "id": row.get::<_, i64>(0)?,
                "name": row.get::<_, String>(1)?,
                "script_path": row.get::<_, String>(2)?,
                "enabled": row.get::<_, i64>(3)? != 0,
            }))
        })
        .map_err(AuraError::Database)?
        .filter_map(|r| r.ok())
        .collect();
    Ok(plugins)
}

/// Register a new plugin.
#[command]
pub fn register_plugin(name: String, script_path: String) -> Result<(), AuraError> {
    let conn = DB.lock().map_err(|_| AuraError::Search("db lock".into()))?;
    conn.execute(
        "INSERT OR REPLACE INTO plugins (name, script_path, enabled) VALUES (?1, ?2, 1)",
        rusqlite::params![name, script_path],
    )
    .map_err(AuraError::Database)?;
    Ok(())
}
