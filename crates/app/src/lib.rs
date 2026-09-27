//! Tauri shell: wires the core to the UI windows.

pub mod engine;
mod hotkeys;
pub mod ipc;

use std::sync::{Arc, Mutex};

use anyhow::Context;
use jarvis_core::modes::ModeCommand;
use jarvis_core::{Config, Db, Paths};
use tauri::Manager;

use engine::{Engine, Msg};

/// Shared app state managed by Tauri.
pub struct AppState {
    pub paths: Paths,
    pub config: Arc<Mutex<Config>>,
    pub db: Mutex<Db>,
    pub engine: Mutex<Option<Engine>>,
}

impl AppState {
    pub fn config_snapshot(&self) -> Config {
        self.config
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .clone()
    }

    fn engine(&self) -> Option<Engine> {
        self.engine
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .clone()
    }
}

/// UI toggles (control panel, tray): switch a listening mode.
#[tauri::command]
fn set_mode(state: tauri::State<'_, AppState>, cmd: ModeCommand) {
    match state.engine() {
        Some(e) => e.send(Msg::Mode(cmd)),
        None => {
            engine::apply_mode(&state.config, &state.paths.config(), cmd);
        }
    }
}

/// Mic button: listen now without the wake word.
#[tauri::command]
fn activate(state: tauri::State<'_, AppState>) {
    if let Some(e) = state.engine() {
        e.send(Msg::Activate);
    }
}

/// Start the Tauri application and block until exit.
pub fn run() -> anyhow::Result<()> {
    let paths = Paths::from_env().context("data dir")?;
    let _log_guard = jarvis_core::logging::init(&paths.logs()).context("logging")?;
    let config = Config::load(&paths.config()).context("config")?;
    let db = Db::open(&paths.db()).context("database")?;
    tracing::info!(version = jarvis_core::VERSION, root = %paths.root.display(), "starting");

    tauri::Builder::default()
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .manage(AppState {
            paths,
            config: Arc::new(Mutex::new(config)),
            db: Mutex::new(db),
            engine: Mutex::new(None),
        })
        .invoke_handler(tauri::generate_handler![set_mode, activate])
        .setup(|app| {
            let handle = app.handle().clone();
            let state = app.state::<AppState>();
            let resources = app.path().resource_dir().ok();
            match jarvis_core::paths::find_assets(resources.as_deref()) {
                Some(assets) => {
                    let engine = engine::spawn(
                        assets,
                        state.config.clone(),
                        state.paths.config(),
                        Arc::new(ipc::TauriSink(handle.clone())),
                    );
                    if let Err(err) = hotkeys::register(&handle, engine.clone()) {
                        tracing::warn!("hotkeys: {err:#}");
                    }
                    *state.engine.lock().unwrap_or_else(|e| e.into_inner()) = Some(engine);
                }
                None => tracing::error!("assets not found; run tools/fetch-assets.ps1"),
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .context("tauri runtime failed")
}
