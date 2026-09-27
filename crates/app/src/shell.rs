//! Desktop shell (SPEC §3.2, §3.5): tray with state icons, close-to-tray (the WebView is
//! destroyed to free RAM; the core keeps listening), reopen on demand, Mica on Windows 11.

use jarvis_core::ipc::AssistantState;
use jarvis_core::modes::ModeCommand;
use tauri::image::Image;
use tauri::menu::{Menu, MenuItem, PredefinedMenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Manager, Runtime, WebviewWindow, WebviewWindowBuilder};

use crate::engine::{self, Msg};
use crate::AppState;

const TRAY: &str = "main";
const MAIN: &str = "main";

fn icon(state: AssistantState) -> Option<Image<'static>> {
    let bytes: &'static [u8] = match state {
        AssistantState::Idle => include_bytes!("../icons/tray-idle.png"),
        AssistantState::Listening => include_bytes!("../icons/tray-listen.png"),
        AssistantState::Processing | AssistantState::Speaking => {
            include_bytes!("../icons/tray-busy.png")
        }
        AssistantState::MicOff => include_bytes!("../icons/tray-off.png"),
    };
    Image::from_bytes(bytes).ok()
}

fn tooltip(state: AssistantState) -> &'static str {
    match state {
        AssistantState::Idle => "Jarvis — ожидает «Джарвис»",
        AssistantState::Listening => "Jarvis — слушает",
        AssistantState::Processing | AssistantState::Speaking => "Jarvis — выполняет",
        AssistantState::MicOff => "Jarvis — микрофон выключен",
    }
}

/// Tray icon follows the assistant state (§3.5).
pub fn tray_state<R: Runtime>(app: &AppHandle<R>, state: AssistantState) {
    if let Some(tray) = app.tray_by_id(TRAY) {
        let _ = tray.set_icon(icon(state));
        let _ = tray.set_tooltip(Some(tooltip(state)));
    }
}

/// Show the main window, recreating it if it was closed to tray.
pub fn show_main<R: Runtime>(app: &AppHandle<R>) {
    if let Some(w) = app.get_webview_window(MAIN) {
        let _ = w.unminimize();
        let _ = w.show();
        let _ = w.set_focus();
        return;
    }
    let Some(cfg) = app.config().app.windows.first().cloned() else {
        return;
    };
    match WebviewWindowBuilder::from_config(app, &cfg).and_then(|b| b.visible(true).build()) {
        Ok(w) => {
            apply_material(&w);
            let _ = w.set_focus();
        }
        Err(e) => tracing::warn!("reopen window: {e}"),
    }
}

/// Hotkey «показать/скрыть окно».
pub fn toggle_main<R: Runtime>(app: &AppHandle<R>) {
    match app.get_webview_window(MAIN) {
        Some(w) if w.is_visible().unwrap_or(false) => {
            let _ = w.hide();
        }
        _ => show_main(app),
    }
}

/// Mica on Windows 11; Windows 10 keeps the opaque CSS background (§3.2).
pub fn apply_material<R: Runtime>(w: &WebviewWindow<R>) {
    #[cfg(windows)]
    {
        let ok = window_vibrancy::apply_mica(w, Some(true)).is_ok();
        let state = w.state::<AppState>();
        state.mica.store(ok, std::sync::atomic::Ordering::Relaxed);
    }
    #[cfg(not(windows))]
    let _ = w;
}

fn mode<R: Runtime>(app: &AppHandle<R>, cmd: ModeCommand) {
    let state = app.state::<AppState>();
    match state.engine() {
        Some(e) => e.send(Msg::Mode(cmd)),
        None => {
            engine::apply_mode(&state.config, &state.paths.config(), cmd);
        }
    }
}

pub fn setup_tray<R: Runtime>(app: &AppHandle<R>) -> tauri::Result<()> {
    let open = MenuItem::with_id(app, "open", "Открыть", true, None::<&str>)?;
    let mic = MenuItem::with_id(app, "mic", "Микрофон вкл/выкл", true, None::<&str>)?;
    let silent = MenuItem::with_id(app, "silent", "Тихий режим вкл/выкл", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "Выход", true, None::<&str>)?;
    let menu = Menu::with_items(
        app,
        &[
            &open,
            &PredefinedMenuItem::separator(app)?,
            &mic,
            &silent,
            &PredefinedMenuItem::separator(app)?,
            &quit,
        ],
    )?;
    let mut builder = TrayIconBuilder::with_id(TRAY)
        .tooltip(tooltip(AssistantState::Idle))
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, ev| match ev.id().as_ref() {
            "open" => show_main(app),
            "mic" => {
                let on = app.state::<AppState>().config_snapshot().mic_enabled;
                mode(
                    app,
                    if on {
                        ModeCommand::MicOff
                    } else {
                        ModeCommand::MicOn
                    },
                );
            }
            "silent" => {
                let on = app.state::<AppState>().config_snapshot().silent_mode;
                mode(
                    app,
                    if on {
                        ModeCommand::SilentOff
                    } else {
                        ModeCommand::SilentOn
                    },
                );
            }
            "quit" => app.exit(0),
            _ => {}
        })
        .on_tray_icon_event(|tray, ev| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = ev
            {
                show_main(tray.app_handle());
            }
        });
    if let Some(i) = icon(AssistantState::Idle) {
        builder = builder.icon(i);
    }
    builder.build(app)?;
    Ok(())
}
