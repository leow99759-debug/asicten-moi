//! Global hotkeys (SPEC §2.3): push-to-talk, show/hide window, mic mute.

use tauri::{AppHandle, Manager, Runtime};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState};

use crate::engine::{Engine, Msg};
use crate::AppState;
use jarvis_core::modes::ModeCommand;

/// (Re)register all three from the config. Each one on its own: a combo taken by another
/// program must not cost the other two. Failures (`[config key, why]`) go to Settings.
pub fn register<R: Runtime>(app: &AppHandle<R>, engine: Engine) -> Vec<[String; 2]> {
    let hk = app.state::<AppState>().config_snapshot().hotkeys;
    let gs = app.global_shortcut();
    if let Err(e) = gs.unregister_all() {
        tracing::warn!("hotkeys unregister: {e}");
    }
    let mut failed = Vec::new();
    let mut note =
        |name: &str, keys: &str, r: Result<(), tauri_plugin_global_shortcut::Error>| match r {
            Ok(()) => tracing::info!(%name, %keys, "hotkey"),
            Err(e) => {
                tracing::warn!(%name, %keys, "hotkey: {e}");
                failed.push([name.to_owned(), why(&e.to_string())]);
            }
        };

    let e = engine.clone();
    note(
        "push_to_talk",
        &hk.push_to_talk,
        gs.on_shortcut(hk.push_to_talk.as_str(), move |_, _, ev| {
            if ev.state() == ShortcutState::Pressed {
                e.send(Msg::Activate);
            }
        }),
    );
    note(
        "toggle_window",
        &hk.toggle_window,
        gs.on_shortcut(hk.toggle_window.as_str(), |app, _, ev| {
            if ev.state() == ShortcutState::Pressed {
                crate::shell::toggle_main(app);
            }
        }),
    );
    note(
        "toggle_mic",
        &hk.toggle_mic,
        gs.on_shortcut(hk.toggle_mic.as_str(), move |app, _, ev| {
            if ev.state() == ShortcutState::Pressed {
                let on = app.state::<AppState>().config_snapshot().mic_enabled;
                engine.send(Msg::Mode(if on {
                    ModeCommand::MicOff
                } else {
                    ModeCommand::MicOn
                }));
            }
        }),
    );
    *app.state::<AppState>()
        .hotkey_errors
        .lock()
        .unwrap_or_else(|e| e.into_inner()) = failed.clone();
    failed
}

fn why(e: &str) -> String {
    if e.contains("already") || e.contains("1409") {
        "занято другой программой — выберите другое сочетание".into()
    } else if e.contains("recognize") || e.contains("format") || e.contains("empty token") {
        "не понял сочетание, нажмите его заново".into()
    } else {
        e.to_owned()
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn errors_read_as_advice() {
        assert!(super::why(
            "Unable to register hotkey: Hot key is already registered. (os error 1409)"
        )
        .contains("занято"));
        assert!(super::why("Couldn't recognize \"О\" as a valid HotKey Code").contains("не понял"));
    }
}
