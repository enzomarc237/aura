use rusqlite::params;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::time::SystemTime;
use walkdir::WalkDir;

use crate::database::DB;
use crate::error::AuraResult;

/// Represents a single indexed item.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct IndexedItem {
    pub id: i64,
    pub title: String,
    pub path: String,
    pub kind: String,
    pub last_modified: Option<i64>,
    pub rank: f64,
}

/// Returns directories to index based on the current platform.
fn index_dirs() -> Vec<std::path::PathBuf> {
    let mut dirs = Vec::new();

    // Applications
    #[cfg(target_os = "macos")]
    {
        dirs.push(std::path::PathBuf::from("/Applications"));
        if let Some(home) = dirs::home_dir() {
            dirs.push(home.join("Applications"));
        }
    }
    #[cfg(not(target_os = "macos"))]
    {
        // On Linux: /usr/bin, /usr/local/bin for testing
        dirs.push(std::path::PathBuf::from("/usr/bin"));
        dirs.push(std::path::PathBuf::from("/usr/local/bin"));
    }

    // User documents/downloads/desktop
    if let Some(home) = dirs::home_dir() {
        dirs.push(home.join("Documents"));
        dirs.push(home.join("Downloads"));
        dirs.push(home.join("Desktop"));
    }

    dirs
}

fn modified_timestamp(path: &Path) -> Option<i64> {
    path.metadata()
        .ok()
        .and_then(|m| m.modified().ok())
        .and_then(|t| t.duration_since(SystemTime::UNIX_EPOCH).ok())
        .map(|d| d.as_secs() as i64)
}

fn classify_kind(path: &Path) -> &'static str {
    #[cfg(target_os = "macos")]
    if path.extension().map_or(false, |e| e == "app") {
        return "application";
    }

    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
    match ext {
        "pdf" | "doc" | "docx" | "txt" | "md" | "rtf" => "document",
        "png" | "jpg" | "jpeg" | "gif" | "webp" | "svg" => "image",
        "mp4" | "mov" | "avi" | "mkv" => "video",
        "mp3" | "flac" | "wav" | "aac" => "audio",
        "zip" | "tar" | "gz" | "bz2" | "7z" => "archive",
        "rs" | "ts" | "js" | "py" | "go" | "java" | "c" | "cpp" | "swift" => "code",
        #[cfg(target_os = "macos")]
        "app" => "application",
        _ => {
            if path.is_dir() {
                "folder"
            } else {
                "file"
            }
        }
    }
}

/// Returns true if a directory-entry name starts with `.` (hidden on Unix).
fn is_hidden(entry: &walkdir::DirEntry) -> bool {
    entry
        .file_name()
        .to_str()
        .map_or(false, |n| n.starts_with('.'))
}

/// Crawl the filesystem and populate / update the search index.
///
/// The filesystem walk is performed *without* holding the DB mutex so that
/// IPC search queries can still be served concurrently.  The collected
/// entries are then upserted in a single locked section.
///
/// Returns the number of items upserted.
pub fn build_index() -> AuraResult<usize> {
    let dirs = index_dirs();

    // --- Phase 1: walk filesystem WITHOUT holding the DB lock ---
    struct Entry {
        title: String,
        path: String,
        kind: &'static str,
        modified: Option<i64>,
    }

    let mut entries: Vec<Entry> = Vec::new();

    for base in dirs {
        if !base.exists() {
            continue;
        }

        let max_depth = if base.to_string_lossy().contains("bin") {
            1
        } else {
            5
        };

        for entry in WalkDir::new(&base)
            .max_depth(max_depth)
            .follow_links(false)
            .into_iter()
            // Prune hidden directories during traversal to avoid descending into them
            .filter_entry(|e| !is_hidden(e))
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            let title = match path.file_name().and_then(|n| n.to_str()) {
                Some(t) if !t.is_empty() => t.to_string(),
                _ => continue,
            };

            // Strip .app extension for display
            let display_title = if title.ends_with(".app") {
                title.trim_end_matches(".app").to_string()
            } else {
                title
            };

            entries.push(Entry {
                title: display_title,
                path: path.to_string_lossy().into_owned(),
                kind: classify_kind(path),
                modified: modified_timestamp(path),
            });
        }
    }

    // --- Phase 2: batch-upsert while holding the DB lock ---
    let count = entries.len();
    let conn = DB.lock().map_err(|_| {
        crate::error::AuraError::Search("db lock poisoned".into())
    })?;

    for e in entries {
        conn.execute(
            "INSERT INTO search_index (title, path, kind, last_modified, rank)
             VALUES (?1, ?2, ?3, ?4, 1.0)
             ON CONFLICT(path) DO UPDATE SET
                 title = excluded.title,
                 kind  = excluded.kind,
                 last_modified = excluded.last_modified",
            params![e.title, e.path, e.kind, e.modified],
        )?;
    }

    Ok(count)
}

/// Fetch all indexed items for search.
pub fn get_all_items() -> AuraResult<Vec<IndexedItem>> {
    let conn = DB.lock().map_err(|_| {
        crate::error::AuraError::Search("db lock poisoned".into())
    })?;

    let mut stmt = conn.prepare(
        "SELECT id, title, path, kind, last_modified, rank FROM search_index ORDER BY rank DESC, title ASC",
    )?;

    let items: Result<Vec<IndexedItem>, _> = stmt
        .query_map([], |row| {
            Ok(IndexedItem {
                id: row.get(0)?,
                title: row.get(1)?,
                path: row.get(2)?,
                kind: row.get(3)?,
                last_modified: row.get(4)?,
                rank: row.get(5)?,
            })
        })?
        .collect();

    Ok(items?)
}

/// Insert a single item (used for plugins / custom commands).
pub fn upsert_item(title: &str, path: &str, kind: &str) -> AuraResult<()> {
    let conn = DB.lock().map_err(|_| {
        crate::error::AuraError::Search("db lock poisoned".into())
    })?;
    conn.execute(
        "INSERT INTO search_index (title, path, kind, rank) VALUES (?1, ?2, ?3, 1.0)
         ON CONFLICT(path) DO UPDATE SET title = excluded.title, kind = excluded.kind",
        params![title, path, kind],
    )?;
    Ok(())
}
