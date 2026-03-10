use once_cell::sync::Lazy;
use rusqlite::{Connection, OptionalExtension, params};
use std::sync::Mutex;
use crate::error::{AuraError, AuraResult, app_data_dir};

pub static DB: Lazy<Mutex<Connection>> = Lazy::new(|| {
    let path = app_data_dir().join("aura.db");
    std::fs::create_dir_all(path.parent().unwrap()).expect("create data dir");
    let conn = Connection::open(&path).expect("open database");
    init_db(&conn).expect("init database");
    Mutex::new(conn)
});

fn init_db(conn: &Connection) -> AuraResult<()> {
    conn.execute_batch("
        PRAGMA journal_mode=WAL;
        PRAGMA synchronous=NORMAL;
        CREATE TABLE IF NOT EXISTS search_index (
            id      INTEGER PRIMARY KEY AUTOINCREMENT,
            title   TEXT NOT NULL,
            path    TEXT NOT NULL UNIQUE,
            kind    TEXT NOT NULL,
            last_modified INTEGER,
            rank    REAL DEFAULT 1.0
        );
        CREATE INDEX IF NOT EXISTS idx_search_title ON search_index(title);
        CREATE INDEX IF NOT EXISTS idx_search_kind  ON search_index(kind);
        CREATE TABLE IF NOT EXISTS history (
            id           INTEGER PRIMARY KEY AUTOINCREMENT,
            query        TEXT NOT NULL,
            selection_id INTEGER,
            timestamp    INTEGER NOT NULL
        );
        CREATE TABLE IF NOT EXISTS settings (
            key   TEXT PRIMARY KEY,
            value TEXT NOT NULL
        );
        CREATE TABLE IF NOT EXISTS plugins (
            id          INTEGER PRIMARY KEY AUTOINCREMENT,
            name        TEXT NOT NULL,
            script_path TEXT NOT NULL,
            enabled     INTEGER NOT NULL DEFAULT 1
        );
    ")?;

    // Seed default settings
    conn.execute(
        "INSERT OR IGNORE INTO settings (key, value) VALUES (?1, ?2)",
        params!["hotkey", "CmdOrCtrl+Space"],
    )?;
    conn.execute(
        "INSERT OR IGNORE INTO settings (key, value) VALUES (?1, ?2)",
        params!["theme", "dark"],
    )?;
    conn.execute(
        "INSERT OR IGNORE INTO settings (key, value) VALUES (?1, ?2)",
        params!["max_results", "10"],
    )?;
    Ok(())
}

pub fn record_history(query: &str, selection_id: Option<i64>) -> AuraResult<()> {
    let conn = DB.lock().map_err(|_| AuraError::Search("db lock poisoned".into()))?;
    let now = chrono::Utc::now().timestamp();
    conn.execute(
        "INSERT INTO history (query, selection_id, timestamp) VALUES (?1, ?2, ?3)",
        params![query, selection_id, now],
    )?;
    Ok(())
}

pub fn get_setting(key: &str) -> AuraResult<Option<String>> {
    let conn = DB.lock().map_err(|_| AuraError::Search("lock poisoned".into()))?;
    let mut stmt = conn.prepare("SELECT value FROM settings WHERE key = ?1")?;
    let result = stmt.query_row(params![key], |row| row.get(0)).optional()?;
    Ok(result)
}

pub fn set_setting(key: &str, value: &str) -> AuraResult<()> {
    let conn = DB.lock().map_err(|_| AuraError::Search("lock poisoned".into()))?;
    conn.execute(
        "INSERT OR REPLACE INTO settings (key, value) VALUES (?1, ?2)",
        params![key, value],
    )?;
    Ok(())
}

pub fn boost_item_rank(item_id: i64) -> AuraResult<()> {
    let conn = DB.lock().map_err(|_| AuraError::Search("lock poisoned".into()))?;
    conn.execute(
        "UPDATE search_index SET rank = MIN(rank + 0.1, 5.0) WHERE id = ?1",
        params![item_id],
    )?;
    Ok(())
}
