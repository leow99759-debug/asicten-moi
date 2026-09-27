//! User config: JSON in `%APPDATA%/Jarvis/config.json` (SPEC §1, §2, §12).
//! Unknown/missing fields fall back to defaults; a corrupt file is kept as `.bak`.

use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::Result;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    /// Commands only after "Джарвис" (§2.2).
    pub prefix_mode: bool,
    /// No voice replies, only sound + visuals (§2.2).
    pub silent_mode: bool,
    /// Microphone fully closed when false (§2.2).
    pub mic_enabled: bool,
    /// Capture device name; `None` = system default.
    pub mic_device: Option<String>,
    /// Wake word threshold slider, 0–100 (§2.2).
    pub wake_sensitivity: u8,
    /// Listening window after wake word, seconds (§2.1).
    pub listen_timeout_sec: u32,
    /// Follow-up window without wake word, seconds (§2.1).
    pub followup_sec: u32,
    /// Keep Vosk loaded after a command, seconds (§1).
    pub stt_keep_warm_sec: u32,
    /// Unload models on idle timers; off = keep Vosk warm always (§12).
    pub memory_saver: bool,
    /// Jarvis voice volume, 0–100 (§3.4).
    pub voice_volume: u8,
    /// «Сворачивать в трей при закрытии» (§10.4): close destroys the WebView, core keeps running.
    pub close_to_tray: bool,
    /// «Автозапуск программы» (§10.4).
    pub autostart: bool,
    /// Voice engine card on «Синтез речи» (§6.4).
    pub voice_engine: VoiceEngine,
    /// Neural voice speed, 0.5–2.0 (§6.4).
    pub voice_speed: f32,
    /// «Как в фильме» EQ + reverb on the neural voice (§6.2).
    pub voice_fx: bool,
    pub hotkeys: Hotkeys,
    /// Voice phrases that switch modes (§2.2), editable in settings.
    pub mode_phrases: ModePhrases,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VoiceEngine {
    /// Phrase pack + neural voice, offline (§6.1–6.2).
    #[default]
    Jarvis,
    /// Built-in Windows voice (§6.3).
    Windows,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Hotkeys {
    pub push_to_talk: String,
    pub toggle_window: String,
    pub toggle_mic: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            prefix_mode: true,
            silent_mode: false,
            mic_enabled: true,
            mic_device: None,
            wake_sensitivity: 50,
            listen_timeout_sec: 6,
            followup_sec: 5,
            stt_keep_warm_sec: 120,
            memory_saver: true,
            voice_volume: 80,
            close_to_tray: true,
            autostart: false,
            voice_engine: VoiceEngine::Jarvis,
            voice_speed: 1.0,
            voice_fx: true,
            hotkeys: Hotkeys::default(),
            mode_phrases: ModePhrases::default(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct ModePhrases {
    pub prefix_on: String,
    pub prefix_off: String,
    pub silent_on: String,
    pub silent_off: String,
    pub mic_off: String,
}

impl Default for ModePhrases {
    fn default() -> Self {
        Self {
            prefix_on: "перейди в режим префикса".into(),
            prefix_off: "выключи режим префикса".into(),
            silent_on: "перейди в тихий режим".into(),
            silent_off: "выключи тихий режим".into(),
            mic_off: "хватит слушать".into(),
        }
    }
}

impl Default for Hotkeys {
    fn default() -> Self {
        Self {
            push_to_talk: "Ctrl+Alt+J".into(),
            toggle_window: "Ctrl+Alt+H".into(),
            toggle_mic: "Ctrl+Alt+M".into(),
        }
    }
}

impl Config {
    /// Load config; missing file → defaults written to disk; corrupt file → `.bak` + defaults.
    pub fn load(path: &Path) -> Result<Self> {
        let cfg = match std::fs::read_to_string(path) {
            Ok(text) => match serde_json::from_str::<Self>(&text) {
                Ok(cfg) => return Ok(cfg.clamped()),
                Err(err) => {
                    tracing::warn!(%err, "config corrupt, backing up and using defaults");
                    std::fs::rename(path, path.with_extension("json.bak"))?;
                    Self::default()
                }
            },
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => Self::default(),
            Err(err) => return Err(err.into()),
        };
        cfg.save(path)?;
        Ok(cfg)
    }

    /// Atomic save: write temp file, then rename over the old one.
    pub fn save(&self, path: &Path) -> Result<()> {
        let tmp = path.with_extension("json.tmp");
        std::fs::write(&tmp, serde_json::to_vec_pretty(self)?)?;
        std::fs::rename(&tmp, path)?;
        Ok(())
    }

    fn clamped(mut self) -> Self {
        self.wake_sensitivity = self.wake_sensitivity.min(100);
        self.voice_volume = self.voice_volume.min(100);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_file_writes_defaults() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("config.json");
        let cfg = Config::load(&path).expect("load");
        assert_eq!(cfg, Config::default());
        assert!(path.exists());
    }

    #[test]
    fn roundtrip_and_partial_fields() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("config.json");
        std::fs::write(
            &path,
            r#"{"silent_mode":true,"voice_volume":250,"unknown":1}"#,
        )
        .expect("write");
        let cfg = Config::load(&path).expect("load");
        assert!(cfg.silent_mode);
        assert!(cfg.prefix_mode, "missing field falls back to default");
        assert_eq!(cfg.voice_volume, 100, "clamped");
        cfg.save(&path).expect("save");
        assert_eq!(Config::load(&path).expect("reload"), cfg);
    }

    #[test]
    fn corrupt_file_is_backed_up() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("config.json");
        std::fs::write(&path, "{not json").expect("write");
        assert_eq!(Config::load(&path).expect("load"), Config::default());
        assert!(path.with_extension("json.bak").exists());
    }
}
