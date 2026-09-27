//! tracing → daily rotated files in `%APPDATA%/Jarvis/logs` (keeps 7) + stderr in debug.

use std::path::Path;

use tracing_appender::non_blocking::WorkerGuard;
use tracing_appender::rolling::{Builder, Rotation};
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

use crate::{Error, Result};

const KEEP_FILES: usize = 7;

/// Install the global subscriber. Keep the guard alive until exit so logs flush.
/// Level comes from `RUST_LOG`, default `info`.
pub fn init(dir: &Path) -> Result<WorkerGuard> {
    let appender = Builder::new()
        .rotation(Rotation::DAILY)
        .filename_prefix("jarvis")
        .filename_suffix("log")
        .max_log_files(KEEP_FILES)
        .build(dir)
        .map_err(|e| Error::Logging(e.to_string()))?;
    let (writer, guard) = tracing_appender::non_blocking(appender);
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    let stderr = cfg!(debug_assertions).then(|| fmt::layer().with_writer(std::io::stderr));
    tracing_subscriber::registry()
        .with(filter)
        .with(fmt::layer().with_ansi(false).with_writer(writer))
        .with(stderr)
        .try_init()
        .map_err(|e| Error::Logging(e.to_string()))?;
    Ok(guard)
}

#[cfg(test)]
mod tests {
    #[test]
    fn writes_log_file() {
        let dir = tempfile::tempdir().expect("tempdir");
        let guard = super::init(dir.path()).expect("init");
        tracing::info!("hello log");
        drop(guard);
        let files: Vec<_> = std::fs::read_dir(dir.path())
            .expect("read_dir")
            .filter_map(|e| e.ok())
            .collect();
        assert_eq!(files.len(), 1);
        let text = std::fs::read_to_string(files[0].path()).expect("read");
        assert!(text.contains("hello log"));
    }
}
