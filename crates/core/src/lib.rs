//! Jarvis core: audio, wake word, STT, NLU, executor, TTS, config, DB, IPC.
//! Platform-independent; Windows side effects live in `jarvis-win` behind traits.

pub mod audio;
pub mod brain;
pub mod commands;
pub mod config;
pub mod db;
pub mod executor;
pub mod info;
pub mod ipc;
pub mod listener;
pub mod logging;
pub mod modes;
mod msvc_compat;
pub mod news;
pub mod nlu;
pub mod paths;
#[cfg(test)]
mod s11_tests;
pub mod scheduler;
pub mod stt;
#[cfg(test)]
mod test_util;
pub mod text;
pub mod tts;
pub mod vad;
pub mod voice;
pub mod wake;

pub use config::Config;
pub use db::Db;
pub use paths::Paths;

/// Product name shown in UI, tray and logs.
pub const APP_NAME: &str = "Jarvis";

/// Error text an online action returns without network (→ status «Нет сети», no_internet clip).
pub const NO_INTERNET: &str = "NO_INTERNET";

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
    #[error("model: {0}")]
    Model(String),
    #[error("logging: {0}")]
    Logging(String),
    #[error("no config directory (APPDATA)")]
    NoConfigDir,
}

pub type Result<T> = std::result::Result<T, Error>;
