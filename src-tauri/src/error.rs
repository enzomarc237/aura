use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AuraError {
    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Search error: {0}")]
    Search(String),
    #[error("Intent error: {0}")]
    Intent(String),
}

impl serde::Serialize for AuraError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.to_string().as_ref())
    }
}

pub type AuraResult<T> = Result<T, AuraError>;

/// Returns the application data directory for Aura.
///
/// Checks the `AURA_DATA_DIR` environment variable first (used in tests to
/// avoid writing to the real user data directory).  Falls back to the OS data
/// directory.  Returns an error when the directory cannot be determined.
pub fn app_data_dir() -> Result<PathBuf, AuraError> {
    if let Ok(dir) = std::env::var("AURA_DATA_DIR") {
        return Ok(PathBuf::from(dir));
    }
    dirs::data_dir()
        .map(|d| d.join("Aura"))
        .ok_or_else(|| AuraError::Search("cannot determine OS data directory".into()))
}
