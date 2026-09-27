//! Tauri implementation of the core event sink.

use jarvis_core::ipc::{CoreEvent, EventSink, CHANNEL};
use tauri::{AppHandle, Emitter};

pub struct TauriSink(pub AppHandle);

impl EventSink for TauriSink {
    fn emit(&self, event: CoreEvent) {
        if let Err(err) = self.0.emit(CHANNEL, event) {
            tracing::warn!(%err, "ipc emit failed");
        }
    }
}
