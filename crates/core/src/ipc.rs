//! Core → UI events (SPEC §1). One Tauri channel `core` carrying a tagged union,
//! so the TS side gets a discriminated `CoreEvent` type from ts-rs.

use serde::Serialize;
use ts_rs::TS;

use crate::brain::Outcome;
use crate::db::HistoryEntry;

/// Tauri event name all core events are emitted on.
pub const CHANNEL: &str = "core";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum AssistantState {
    Idle,
    Listening,
    Processing,
    Speaking,
    MicOff,
}

#[derive(Debug, Clone, PartialEq, Serialize, TS)]
#[ts(export)]
pub struct Transcript {
    pub text: String,
    /// false = partial result (listening bar), true = final utterance.
    pub is_final: bool,
}

/// Audio levels 0.0–1.0 for orb/listening bar.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, TS)]
#[ts(export)]
pub struct Level {
    pub mic: f32,
    pub tts: f32,
}

/// Confirmation dialog (SPEC §4.6).
#[derive(Debug, Clone, PartialEq, Serialize, TS)]
#[ts(export)]
pub struct ConfirmRequest {
    pub question: String,
    pub timeout_sec: u32,
}

/// Initial state for the main window (control panel, dashboard counter).
#[derive(Debug, Clone, PartialEq, Serialize, TS)]
#[ts(export)]
pub struct UiSnapshot {
    pub prefix_mode: bool,
    pub silent_mode: bool,
    pub mic_enabled: bool,
    pub voice_volume: u8,
    #[ts(type = "number")]
    pub commands: usize,
}

/// Assistant actions that drive the UI.
#[derive(Debug, Clone, PartialEq, Serialize, TS)]
#[serde(tag = "kind", content = "value", rename_all = "snake_case")]
#[ts(export)]
pub enum UiCommand {
    OpenPage(String),
    SetTheme(String),
}

#[derive(Debug, Clone, PartialEq, Serialize, TS)]
#[serde(tag = "event", content = "payload", rename_all = "snake_case")]
#[ts(export)]
pub enum CoreEvent {
    State(AssistantState),
    Transcript(Transcript),
    History(HistoryEntry),
    Level(Level),
    Confirm(ConfirmRequest),
    ConfirmClosed,
    /// Result cards for the last utterance (§4.3).
    Outcome(Outcome),
    /// What Jarvis says (also spoken once voice output exists).
    Say(String),
    Ui(UiCommand),
}

/// Where the core publishes events; the app implements it over Tauri, tests collect.
pub trait EventSink: Send + Sync {
    fn emit(&self, event: CoreEvent);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::Status;

    #[test]
    fn wire_format_is_tagged() {
        let json = |e: &CoreEvent| serde_json::to_string(e).expect("json");
        assert_eq!(
            json(&CoreEvent::State(AssistantState::MicOff)),
            r#"{"event":"state","payload":"mic_off"}"#
        );
        assert_eq!(
            json(&CoreEvent::Transcript(Transcript {
                text: "джарвис".into(),
                is_final: false
            })),
            r#"{"event":"transcript","payload":{"text":"джарвис","is_final":false}}"#
        );
        let h = HistoryEntry {
            id: 1,
            ts: 2,
            phrase: "p".into(),
            command_id: None,
            status: Status::NoInternet,
        };
        assert_eq!(
            json(&CoreEvent::History(h)),
            r#"{"event":"history","payload":{"id":1,"ts":2,"phrase":"p","command_id":null,"status":"no_internet"}}"#
        );
    }
}
