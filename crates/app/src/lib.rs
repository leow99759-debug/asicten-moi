//! Tauri shell: wires the core to the UI windows.

pub mod brain_worker;
mod editor;
pub mod engine;
mod hotkeys;
pub mod ipc;
mod overlay;
mod shell;
pub mod speaker;

use std::sync::mpsc::{self, Sender};
use std::sync::{Arc, Mutex};

use anyhow::Context;
use jarvis_core::modes::ModeCommand;
use jarvis_core::{Config, Db, Paths};
use tauri::Manager;

use brain_worker::{Confirm, Work};
use engine::{Engine, Msg};

/// Shared app state managed by Tauri.
pub struct AppState {
    pub paths: Paths,
    pub config: Arc<Mutex<Config>>,
    pub db: Arc<Mutex<Db>>,
    pub engine: Mutex<Option<Engine>>,
    pub confirm: Arc<Confirm>,
    pub work: Sender<Work>,
    /// Loaded commands (dashboard counter).
    pub commands: std::sync::atomic::AtomicUsize,
    /// Mica applied to the main window (UI then drops its opaque background).
    pub mica: std::sync::atomic::AtomicBool,
    /// Voice output, for «▶ Прослушать» on the voice page.
    pub speaker: std::sync::OnceLock<Arc<speaker::Speaker>>,
    /// Desktop avatar + HUD windows (§3.6, §3.7).
    pub overlay: std::sync::OnceLock<overlay::Overlay>,
    /// Built-in packs folder, for the command editor.
    pub packs: std::sync::OnceLock<Option<std::path::PathBuf>>,
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

/// Confirmation dialog buttons (§4.6).
#[tauri::command]
fn confirm_answer(state: tauri::State<'_, AppState>, yes: bool) {
    state.confirm.answer(yes);
}

/// Typed command (PWA/remote, editor «▶ Test», history repeat).
#[tauri::command]
fn run_text(state: tauri::State<'_, AppState>, text: String) {
    let _ = state.work.send(Work::Utterance(text));
}

/// Main window history list (§3.4), newest first.
#[tauri::command]
fn history(
    state: tauri::State<'_, AppState>,
    limit: u32,
) -> Result<Vec<jarvis_core::db::HistoryEntry>, String> {
    let db = state.db.lock().unwrap_or_else(|e| e.into_inner());
    db.history(limit.min(500)).map_err(|e| e.to_string())
}

/// True when Windows 11 Mica is behind the window (CSS goes translucent).
#[tauri::command]
fn window_material(state: tauri::State<'_, AppState>) -> bool {
    state.mica.load(std::sync::atomic::Ordering::Relaxed)
}

/// Settings page: the whole config (§10.4).
#[tauri::command]
fn get_config(state: tauri::State<'_, AppState>) -> Config {
    state.config_snapshot()
}

/// Settings page save. Mode flags stay as they are (they change by voice/toggles via
/// `set_mode`), everything else is replaced, saved and applied.
#[tauri::command]
fn set_config(
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
    mut config: Config,
) -> Result<(), String> {
    {
        let mut cur = state.config.lock().unwrap_or_else(|e| e.into_inner());
        config.prefix_mode = cur.prefix_mode;
        config.silent_mode = cur.silent_mode;
        config.mic_enabled = cur.mic_enabled;
        config.wake_sensitivity = config.wake_sensitivity.min(100);
        config.voice_volume = config.voice_volume.min(100);
        config.voice_speed = config.voice_speed.clamp(0.5, 2.0);
        // placement is saved by the avatar itself (overlay edit mode)
        config.ui.avatar_pos = cur.ui.avatar_pos;
        config
            .save(&state.paths.config())
            .map_err(|e| e.to_string())?;
        *cur = config.clone();
    }
    #[cfg(windows)]
    if let Ok(exe) = std::env::current_exe() {
        jarvis_win::autostart::set(config.autostart, &exe)?;
    }
    if let Some(e) = state.engine() {
        e.send(Msg::Reconfigure);
    }
    if let Some(o) = state.overlay.get() {
        o.send(overlay::Msg::Sync);
    }
    // avatar/HUD windows re-read the accent colour
    let _ = tauri::Emitter::emit(&app, "config", ());
    Ok(())
}

/// «Переместить аватар»: make the avatar draggable; `false` saves the spot (§3.6).
#[tauri::command]
fn avatar_edit(state: tauri::State<'_, AppState>, on: bool) {
    if let Some(o) = state.overlay.get() {
        o.send(overlay::Msg::Edit(on));
    }
}

/// «Показать HUD» in settings (§3.7).
#[tauri::command]
fn hud_preview(state: tauri::State<'_, AppState>) {
    if let Some(o) = state.overlay.get() {
        o.send(overlay::Msg::HudPreview);
    }
}

/// Microphone picker (§10.4 «Модель микрофона»).
#[tauri::command]
fn mic_devices() -> Vec<String> {
    #[cfg(windows)]
    {
        jarvis_core::audio::input_devices().unwrap_or_default()
    }
    #[cfg(not(windows))]
    {
        Vec::new()
    }
}

/// «▶» on the voice page: say a sample with the current voice settings (§6.4).
#[tauri::command]
fn preview_voice(state: tauri::State<'_, AppState>) {
    if let Some(s) = state.speaker.get().cloned() {
        std::thread::spawn(move || {
            s.stop();
            s.say_text(
                &[],
                "Добрый день, сэр. Все системы работают в штатном режиме.",
            );
        });
    }
}

/// Main window start-up state.
#[tauri::command]
fn ui_snapshot(state: tauri::State<'_, AppState>) -> jarvis_core::ipc::UiSnapshot {
    let c = state.config_snapshot();
    jarvis_core::ipc::UiSnapshot {
        prefix_mode: c.prefix_mode,
        silent_mode: c.silent_mode,
        mic_enabled: c.mic_enabled,
        voice_volume: c.voice_volume,
        commands: state.commands.load(std::sync::atomic::Ordering::Relaxed),
    }
}

/// Control panel «Громкость» (§3.4): Jarvis voice volume, 0–100.
#[tauri::command]
fn set_voice_volume(state: tauri::State<'_, AppState>, volume: u8) {
    let mut c = state.config.lock().unwrap_or_else(|e| e.into_inner());
    c.voice_volume = volume.min(100);
    if let Err(e) = c.save(&state.paths.config()) {
        tracing::warn!("config save: {e}");
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
    let (work_tx, work_rx) = mpsc::channel();

    tauri::Builder::default()
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_dialog::init())
        .manage(AppState {
            paths,
            config: Arc::new(Mutex::new(config)),
            db: Arc::new(Mutex::new(db)),
            engine: Mutex::new(None),
            confirm: Arc::new(Confirm::default()),
            work: work_tx,
            commands: Default::default(),
            mica: Default::default(),
            speaker: Default::default(),
            overlay: Default::default(),
            packs: Default::default(),
        })
        .invoke_handler(tauri::generate_handler![
            set_mode,
            activate,
            confirm_answer,
            run_text,
            history,
            ui_snapshot,
            set_voice_volume,
            window_material,
            get_config,
            set_config,
            mic_devices,
            preview_voice,
            avatar_edit,
            hud_preview,
            editor::editor_library,
            editor::editor_save,
            editor::editor_probe,
            editor::editor_test,
            editor::say_reply
        ])
        .setup(move |app| {
            let handle = app.handle().clone();
            let state = app.state::<AppState>();
            if let Some(w) = app.get_webview_window("main") {
                shell::apply_material(&w);
                // autostart passes --minimized: stay in the tray
                if !std::env::args().any(|a| a == "--minimized") {
                    let _ = w.show();
                }
            }
            if let Err(e) = shell::setup_tray(&handle) {
                tracing::warn!("tray: {e}");
            }
            let _ = state.overlay.set(overlay::spawn(handle.clone()));
            #[cfg(windows)]
            if let Ok(exe) = std::env::current_exe() {
                if let Err(e) = jarvis_win::autostart::set(state.config_snapshot().autostart, &exe)
                {
                    tracing::warn!("autostart: {e}");
                }
            }
            let resources = app.path().resource_dir().ok();
            let packs = jarvis_core::paths::find_packs(resources.as_deref());
            let _ = state.packs.set(packs.clone());
            let sink: Arc<dyn jarvis_core::ipc::EventSink> =
                Arc::new(ipc::TauriSink(handle.clone()));
            let Some(assets) = jarvis_core::paths::find_assets(resources.as_deref()) else {
                tracing::error!("assets not found; run tools/fetch-assets.ps1");
                return Ok(());
            };
            let speaker = Arc::new(speaker::Speaker::new(
                &assets,
                sink.clone(),
                state.config.clone(),
            ));
            let _ = state.speaker.set(speaker.clone());
            // Priler's pack calls the startup line `run`
            speaker.say(&jarvis_core::brain::Line {
                clips: vec!["greet".into(), "run".into()],
                text: None,
            });
            let engine = engine::spawn(
                assets.clone(),
                state.config.clone(),
                state.paths.config(),
                sink.clone(),
                engine::Route {
                    work: state.work.clone(),
                    confirm: state.confirm.clone(),
                    speaker: speaker.clone(),
                },
            );
            let commands =
                brain_worker::load_commands(packs.as_deref(), &state.paths.user_commands());
            state
                .commands
                .store(commands.len(), std::sync::atomic::Ordering::Relaxed);
            brain_worker::spawn(
                commands,
                brain_worker::Deps {
                    sink,
                    speaker,
                    db: state.db.clone(),
                    confirm: state.confirm.clone(),
                    engine: engine.clone(),
                },
                state.work.clone(),
                work_rx,
            );
            if let Err(err) = hotkeys::register(&handle, engine.clone()) {
                tracing::warn!("hotkeys: {err:#}");
            }
            *state.engine.lock().unwrap_or_else(|e| e.into_inner()) = Some(engine);
            Ok(())
        })
        .build(tauri::generate_context!())
        .context("tauri init failed")?
        .run(|app, event| {
            // overlay windows keep the app alive, so closing the main window exits by hand
            if let tauri::RunEvent::WindowEvent {
                label,
                event: tauri::WindowEvent::Destroyed,
                ..
            } = &event
            {
                if label == "main" && !app.state::<AppState>().config_snapshot().close_to_tray {
                    app.exit(0);
                }
            }
            // last window closed (not «Выход»): keep the core running in the tray (§3.5)
            if let tauri::RunEvent::ExitRequested {
                api, code: None, ..
            } = &event
            {
                if app.state::<AppState>().config_snapshot().close_to_tray {
                    api.prevent_exit();
                }
            }
        });
    Ok(())
}
