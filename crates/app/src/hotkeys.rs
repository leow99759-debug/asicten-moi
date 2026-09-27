//! Global hotkeys (SPEC §2.3): push-to-talk, show/hide window, mic mute.

use tauri::{AppHandle, Manager, Runtime};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState};

use crate::engine::{Engine, Msg};
use crate::AppState;
use jarvis_core::modes::ModeCommand;

pub fn register<R: Runtime>(app: &AppHandle<R>, engine: Engine) -> anyhow::Result<()> {
    let hk = app.state::<AppState>().config_snapshot().hotkeys;
    let gs = app.global_shortcut();

    let e = engine.clone();
    gs.on_shortcut(hk.push_to_talk.as_str(), move |_, _, ev| {
        if ev.state() == ShortcutState::Pressed {
            e.send(Msg::Activate);
        }
    })?;

    gs.on_shortcut(hk.toggle_window.as_str(), |app, _, ev| {
        if ev.state() == ShortcutState::Pressed {
            crate::shell::toggle_main(app);
        }
    })?;

    gs.on_shortcut(hk.toggle_mic.as_str(), move |app, _, ev| {
        if ev.state() == ShortcutState::Pressed {
            let on = app.state::<AppState>().config_snapshot().mic_enabled;
            engine.send(Msg::Mode(if on {
                ModeCommand::MicOff
            } else {
                ModeCommand::MicOn
            }));
        }
    })?;
    Ok(())
}
