//! Jarvis core: audio, wake word, STT, NLU, executor, TTS, config, DB, IPC.
//! Platform-independent; Windows side effects live in `jarvis-win` behind traits.

pub mod audio;
pub mod config;
pub mod db;
pub mod ipc;
pub mod logging;
pub mod paths;

pub use config::Config;
pub use db::Db;
pub use paths::Paths;

/// Product name shown in UI, tray and logs.
pub const APP_NAME: &str = "Jarvis";

/// Crate version, shared by every workspace crate.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("json: {0}")]
    Json(#[from] serde_json::Error),
    #[error("sqlite: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("audio: {0}")]
    Audio(String),
    #[error("logging: {0}")]
    Logging(String),
    #[error("no config directory (APPDATA)")]
    NoConfigDir,
}

pub type Result<T> = std::result::Result<T, Error>;
