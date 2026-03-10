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
pub fn app_data_dir() -> PathBuf {
    dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("Aura")
}
