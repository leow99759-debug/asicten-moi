//! Action types (SPEC §4.4). JSON: `{"type": "Launch.File", "path": "%CHROME%"}`.
//! Numeric params accept a number or a slot placeholder like `"{число}"`.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// Number or slot placeholder (`"{число}"`, `"{время}"`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(untagged)]
#[ts(export)]
pub enum Num {
    Value(f64),
    Slot(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "lowercase")]
#[ts(export)]
pub enum Side {
    Left,
    Right,
    Top,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "lowercase")]
#[ts(export)]
pub enum PowerPlan {
    Balanced,
    High,
    Saver,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum AssistantMode {
    Prefix,
    Silent,
    MicOff,
}

fn zero() -> Num {
    Num::Value(0.0)
}

fn step() -> Num {
    Num::Value(10.0)
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(tag = "type")]
#[ts(export)]
pub enum Action {
    // Launch
    #[serde(rename = "Launch.File")]
    LaunchFile {
        path: String,
        #[serde(default)]
        args: String,
        #[serde(default)]
        workdir: Option<String>,
        #[serde(default)]
        admin: bool,
    },
    #[serde(rename = "Launch.Url")]
    LaunchUrl {
        url: String,
    },
    #[serde(rename = "Launch.Uwp")]
    LaunchUwp {
        aumid: String,
    },
    #[serde(rename = "Launch.Find")]
    LaunchFind {
        name: String,
    },
    #[serde(rename = "Process.Kill")]
    ProcessKill {
        name: String,
    },

    // Windows
    #[serde(rename = "Window.Close")]
    WindowClose,
    #[serde(rename = "Window.Minimize")]
    WindowMinimize,
    #[serde(rename = "Window.Maximize")]
    WindowMaximize,
    #[serde(rename = "Window.Restore")]
    WindowRestore,
    #[serde(rename = "Window.MinimizeAll")]
    WindowMinimizeAll,
    /// Politely close every app window (WM_CLOSE: apps may still ask to save).
    #[serde(rename = "Window.CloseAll")]
    WindowCloseAll,
    #[serde(rename = "Window.Focus")]
    WindowFocus {
        target: String,
    },
    #[serde(rename = "Window.Snap")]
    WindowSnap {
        side: Side,
    },
    #[serde(rename = "Window.MoveToMonitor")]
    WindowMoveToMonitor {
        n: Num,
    },
    #[serde(rename = "Window.Fullscreen")]
    WindowFullscreen,

    // Input
    #[serde(rename = "Keys.Press")]
    KeysPress {
        keys: String,
    },
    #[serde(rename = "Keys.Type")]
    KeysType {
        text: String,
    },
    #[serde(rename = "Keys.Hold")]
    KeysHold {
        key: String,
        ms: Num,
    },
    #[serde(rename = "Mouse.ToCoords")]
    MouseToCoords {
        x: Num,
        y: Num,
    },
    #[serde(rename = "Mouse.ClickLeft")]
    MouseClickLeft {
        #[serde(default)]
        x: Option<Num>,
        #[serde(default)]
        y: Option<Num>,
    },
    #[serde(rename = "Mouse.ClickRight")]
    MouseClickRight {
        #[serde(default)]
        x: Option<Num>,
        #[serde(default)]
        y: Option<Num>,
    },
    #[serde(rename = "Mouse.Double")]
    MouseDouble {
        #[serde(default)]
        x: Option<Num>,
        #[serde(default)]
        y: Option<Num>,
    },
    #[serde(rename = "Mouse.LC")]
    MouseLc {
        n: Num,
    },
    #[serde(rename = "Mouse.Scroll")]
    MouseScroll {
        dy: Num,
    },

    // Audio / media
    #[serde(rename = "System.VolumeSet")]
    VolumeSet {
        level: Num,
    },
    #[serde(rename = "System.VolumeUp")]
    VolumeUp {
        #[serde(default = "step")]
        step: Num,
    },
    #[serde(rename = "System.VolumeDown")]
    VolumeDown {
        #[serde(default = "step")]
        step: Num,
    },
    #[serde(rename = "System.Mute")]
    Mute,
    #[serde(rename = "System.Unmute")]
    Unmute,
    #[serde(rename = "Media.PlayPause")]
    MediaPlayPause,
    #[serde(rename = "Media.Next")]
    MediaNext,
    #[serde(rename = "Media.Prev")]
    MediaPrev,
    #[serde(rename = "Media.Stop")]
    MediaStop,
    #[serde(rename = "Audio.SwitchDevice")]
    AudioSwitchDevice {
        name: String,
    },

    // System
    #[serde(rename = "System.Shutdown")]
    Shutdown {
        #[serde(default = "zero")]
        delay_sec: Num,
    },
    #[serde(rename = "System.Restart")]
    Restart {
        #[serde(default = "zero")]
        delay_sec: Num,
    },
    #[serde(rename = "System.Sleep")]
    Sleep,
    #[serde(rename = "System.Hibernate")]
    Hibernate,
    #[serde(rename = "System.Lock")]
    Lock,
    #[serde(rename = "System.Logoff")]
    Logoff,
    #[serde(rename = "System.CancelShutdown")]
    CancelShutdown,
    #[serde(rename = "System.PowerPlan")]
    SetPowerPlan {
        plan: PowerPlan,
    },
    #[serde(rename = "System.Brightness")]
    Brightness {
        level: Num,
    },
    #[serde(rename = "System.Screenshot")]
    Screenshot {
        #[serde(default)]
        region: bool,
    },
    #[serde(rename = "System.OpenSettings")]
    OpenSettings {
        uri: String,
    },
    #[serde(rename = "System.EmptyRecycleBin")]
    EmptyRecycleBin,
    #[serde(rename = "System.WiFi")]
    WiFi {
        on: bool,
    },
    #[serde(rename = "System.Bluetooth")]
    Bluetooth {
        on: bool,
    },
    #[serde(rename = "System.DND")]
    Dnd {
        on: bool,
    },
    #[serde(rename = "System.GameMode")]
    GameMode {
        on: bool,
    },
    #[serde(rename = "System.Clipboard")]
    Clipboard {
        #[serde(default)]
        text: Option<String>,
    },
    #[serde(rename = "System.Info")]
    Info {
        what: String,
    },

    // Service
    #[serde(rename = "Lux.PauseMS")]
    PauseMs {
        ms: Num,
    },
    #[serde(rename = "Lux.PauseSec")]
    PauseSec {
        s: Num,
    },
    #[serde(rename = "Sound.PlayWav")]
    PlayWav {
        path: String,
    },
    Speak {
        #[serde(default)]
        text: Option<String>,
        #[serde(default)]
        clip: Option<String>,
    },
    Ask {
        question: String,
    },
    #[serde(rename = "Run.Command")]
    RunCommand {
        cmd: String,
        #[serde(default)]
        powershell: bool,
        #[serde(default)]
        trusted: bool,
    },
    #[serde(rename = "Run.Command.Hidden")]
    RunCommandHidden {
        cmd: String,
        #[serde(default)]
        powershell: bool,
        #[serde(default)]
        trusted: bool,
    },
    Timer {
        sec: Num,
        then_command: String,
    },
    /// «Напомни через {время} {текст}»: seconds from now (§10.3).
    Reminder {
        sec: Num,
        text: String,
    },

    // Assistant
    #[serde(rename = "Assistant.Mode")]
    AssistantSetMode {
        mode: AssistantMode,
        on: bool,
    },
    #[serde(rename = "Assistant.SetTheme")]
    AssistantSetTheme {
        color: String,
    },
    #[serde(rename = "Assistant.OpenPage")]
    AssistantOpenPage {
        page: String,
    },
    #[serde(rename = "Assistant.Repeat")]
    AssistantRepeat,
    #[serde(rename = "Assistant.Cancel")]
    AssistantCancel,
    /// «Джарвис, выключи себя»: exit the app (not the PC).
    #[serde(rename = "Assistant.Quit")]
    AssistantQuit,

    // Office (COM)
    #[serde(rename = "PowerPoint.NewPresentation")]
    PptNew {
        topic: String,
        #[serde(default)]
        slides: Option<Num>,
    },
    #[serde(rename = "PowerPoint.ApplyTheme")]
    PptApplyTheme {
        name: String,
    },
    #[serde(rename = "PowerPoint.SetVariantColor")]
    PptSetVariantColor {
        color: String,
    },
    #[serde(rename = "Word.Bold")]
    WordBold,
    #[serde(rename = "Word.Italic")]
    WordItalic,
    #[serde(rename = "Word.NewParagraph")]
    WordNewParagraph,
    #[serde(rename = "Word.Print")]
    WordPrint,
    #[serde(rename = "Word.Save")]
    WordSave,
    #[serde(rename = "Word.Dictate")]
    WordDictate {
        text: String,
    },

    // UI Automation
    #[serde(rename = "UIA.Click")]
    UiaClick {
        name: String,
        #[serde(default)]
        window: Option<String>,
    },
    #[serde(rename = "UIA.SetText")]
    UiaSetText {
        name: String,
        text: String,
        #[serde(default)]
        window: Option<String>,
    },
}

impl Action {
    /// Dangerous by default → confirm flow (SPEC §4.6).
    pub fn needs_confirm(&self) -> bool {
        match self {
            Self::Shutdown { .. } | Self::Restart { .. } | Self::Logoff => true,
            Self::RunCommand { trusted, .. } | Self::RunCommandHidden { trusted, .. } => !trusted,
            _ => false,
        }
    }

    /// Every string/number param, for placeholder validation.
    pub fn params(&self) -> Vec<String> {
        let json = serde_json::to_value(self).unwrap_or_default();
        let mut out = Vec::new();
        if let serde_json::Value::Object(map) = json {
            for (k, v) in map {
                if k != "type" {
                    if let serde_json::Value::String(s) = v {
                        out.push(s);
                    }
                }
            }
        }
        out
    }
}
