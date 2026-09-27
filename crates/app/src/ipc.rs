//! Tauri implementation of the core event sink.

use jarvis_core::ipc::{CoreEvent, EventSink, CHANNEL};
use tauri::{AppHandle, Emitter, Manager};

pub struct TauriSink(pub AppHandle);

impl EventSink for TauriSink {
    fn emit(&self, event: CoreEvent) {
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
