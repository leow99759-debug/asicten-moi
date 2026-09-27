//! Tauri shell: wires the core to the UI windows.

pub mod ipc;

use std::sync::Mutex;

use anyhow::Context;
use jarvis_core::{Config, Db, Paths};

/// Shared app state managed by Tauri.
pub struct AppState {
    pub paths: Paths,
    pub config: Mutex<Config>,
    pub db: Mutex<Db>,
}

/// Start the Tauri application and block until exit.
pub fn run() -> anyhow::Result<()> {
    let paths = Paths::from_env().context("data dir")?;
    let _log_guard = jarvis_core::logging::init(&paths.logs()).context("logging")?;
    let config = Config::load(&paths.config()).context("config")?;
    let db = Db::open(&paths.db()).context("database")?;
    tracing::info!(version = jarvis_core::VERSION, root = %paths.root.display(), "starting");

    tauri::Builder::default()
        .manage(AppState {
            paths,
            config: Mutex::new(config),
            db: Mutex::new(db),
        })
        .run(tauri::generate_context!())
        .context("tauri runtime failed")
}
