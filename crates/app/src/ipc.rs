//! Tauri implementation of the core event sink.

use jarvis_core::ipc::{CoreEvent, EventSink, UiCommand, CHANNEL};
use tauri::{AppHandle, Emitter, Manager};

pub struct TauriSink(pub AppHandle);

impl EventSink for TauriSink {
    fn emit(&self, mut event: CoreEvent) {
        if let CoreEvent::Ui(UiCommand::SetTheme(said)) = &mut event {
            // voice theme works with the window closed too: persist, then every window re-reads
            if let Some(hex) = jarvis_core::config::theme_hex(said) {
                let state = self.0.state::<crate::AppState>();
                let mut cfg = state.config.lock().unwrap_or_else(|e| e.into_inner());
                cfg.ui.accent = hex.clone();
                if let Err(e) = cfg.save(&state.paths.config()) {
                    tracing::warn!("config save: {e}");
                }
                drop(cfg);
                let _ = self.0.emit("config", ());
                *said = hex;
            }
        }
        if let CoreEvent::Ui(UiCommand::OpenPage(page)) = &event {
            crate::shell::open_main(&self.0, Some(page));
        }
        if let CoreEvent::State(s) = &event {
            crate::shell::tray_state(&self.0, *s);
            if let Some(o) = self.0.state::<crate::AppState>().overlay.get() {
                o.send(crate::overlay::Msg::State(*s));
            }
        }
        if let Err(err) = self.0.emit(CHANNEL, event) {
            tracing::warn!(%err, "ipc emit failed");
        }
    }
}
