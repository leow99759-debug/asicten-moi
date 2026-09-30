//! User config: JSON in `%APPDATA%/Jarvis/config.json` (SPEC §1, §2, §12).
//! Unknown/missing fields fall back to defaults; a corrupt file is kept as `.bak`.

use std::path::Path;

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::Result;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(default)]
#[ts(export)]
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
    /// Look & feel (§3.1, §10.4 «Интерфейс»), applied by the UI.
    pub ui: UiPrefs,
    /// Voice phrases that switch modes (§2.2), editable in settings.
    pub mode_phrases: ModePhrases,
    /// Online keys (Settings → ИИ): stay in this PC's config.json, never in git.
    pub online: Online,
    /// The first-run greeting («первичная настройка… калибровка… все системы штатно») played.
    pub introduced: bool,
}

/// Who voices dynamic text online (see [`Online::cloud_voice`]).
#[derive(Debug, Clone, PartialEq)]
pub enum CloudVoice {
    Eleven { key: String, voice: String },
    Fish { key: String, voice: String },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(default)]
#[ts(export)]
pub struct Online {
    /// Google AI Studio keys for the news digest; the second is used when the first hits its quota.
    pub gemini_keys: Vec<String>,
    pub gemini_model: String,
    /// Fish Audio key: any text without a recording is spoken in the Jarvis voice online.
    pub fish_key: String,
    pub fish_voice: String,
    /// ElevenLabs key: when set, it voices text instead of Fish (studio-grade Russian).
    pub eleven_key: String,
    /// Voice ID from elevenlabs.io (premade or Voice Library).
    pub eleven_voice: String,
    /// Google Cloud OAuth «Desktop app» client for Classroom homework.
    pub classroom_id: String,
    pub classroom_secret: String,
    /// Refresh token from «Подключить» (set by the app, kept on UI saves).
    pub classroom_token: String,
}

impl Default for Online {
    fn default() -> Self {
        Self {
            gemini_keys: vec![String::new(), String::new()],
            gemini_model: "gemini-2.5-flash-lite".into(),
            fish_key: String::new(),
            // empty = the chosen pack's own voice ([`VoiceEngine::pack`])
            fish_voice: String::new(),
            // «Daniel»: calm British newsreader, the closest premade to Jarvis
            eleven_voice: "onwK4e9ZLuTAKqWW03F9".into(),
            eleven_key: String::new(),
            classroom_id: String::new(),
            classroom_secret: String::new(),
            classroom_token: String::new(),
        }
    }
}

impl Online {
    pub fn gemini_keys(&self) -> impl Iterator<Item = &str> {
        self.gemini_keys
            .iter()
            .map(|k| k.trim())
            .filter(|k| !k.is_empty())
    }

    /// Online voice for text without a recording: ElevenLabs first, then Fish Audio in the
    /// pack's own voice unless the user set another model.
    pub fn cloud_voice(&self, engine: VoiceEngine) -> Option<CloudVoice> {
        let own = engine.pack().map_or(JARVIS_FISH, |p| p.1);
        let fish = match self.fish_voice.trim() {
            "" | JARVIS_FISH | FILM_FISH => own,
            v => v,
        };
        let pick = |key: &str, voice: &str| {
            let k = key.trim();
            (!k.is_empty()).then(|| (k.to_owned(), voice.trim().to_owned()))
        };
        pick(&self.eleven_key, &self.eleven_voice)
            .map(|(key, voice)| CloudVoice::Eleven { key, voice })
            .or_else(|| {
                pick(&self.fish_key, fish).map(|(key, voice)| CloudVoice::Fish { key, voice })
            })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum VoiceEngine {
    /// Phrase pack + neural voice, offline (§6.1–6.2): lines in the Luxify-style Jarvis voice.
    #[default]
    Jarvis,
    /// Same, with the RU film dub originals.
    Film,
    /// Built-in Windows voice (§6.3).
    Windows,
}

/// Fish Audio voices of the built-in packs.
pub const JARVIS_FISH: &str = "78384906f98747bcab6ed67dd3ffa8aa";
pub const FILM_FISH: &str = "4c3eaacc1a0545cdb0295bfddf3e3785";

impl VoiceEngine {
    /// Phrase pack dir under assets + its Fish Audio voice; `None` = Windows voice.
    pub fn pack(self) -> Option<(&'static str, &'static str)> {
        match self {
            Self::Jarvis => Some(("voice-jarvis-v2", JARVIS_FISH)),
            Self::Film => Some(("voice-jarvis", FILM_FISH)),
            Self::Windows => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(default)]
#[ts(export)]
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
            ui: UiPrefs::default(),
            mode_phrases: ModePhrases::default(),
            online: Online::default(),
            introduced: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(default)]
#[ts(export)]
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

/// Theme swatches (Luxify palette), same order as the settings page.
pub const THEMES: [(&str, &str); 8] = [
    ("синий", "#4486ff"),
    ("фиолетовый", "#8b5cf6"),
    ("розовый", "#ec4899"),
    ("красный", "#ef4444"),
    ("оранжевый", "#f59e0b"),
    ("зелёный", "#22c55e"),
    ("голубой", "#06b6d4"),
    ("серый", "#94a3b8"),
];

/// «сделай тему фиолетовой» → `#8b5cf6`; also takes `#rrggbb`. Word stems, so any case ending works.
pub fn theme_hex(said: &str) -> Option<String> {
    let s = said.trim().to_lowercase().replace('ё', "е");
    if s.len() == 7 && s.starts_with('#') && s[1..].chars().all(|c| c.is_ascii_hexdigit()) {
        return Some(s);
    }
    const STEMS: [(&str, usize); 13] = [
        ("син", 0),
        ("фиолет", 1),
        ("сирен", 1),
        ("розов", 2),
        ("пурпур", 2),
        ("красн", 3),
        ("оранж", 4),
        ("желт", 4),
        ("зелен", 5),
        ("голуб", 6),
        ("бирюз", 6),
        ("сер", 7),
        ("бел", 7),
    ];
    s.split_whitespace().find_map(|w| {
        STEMS
            .iter()
            .find(|(stem, _)| w.starts_with(stem))
            .map(|&(_, i)| THEMES[i].1.to_owned())
    })
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(default)]
#[ts(export)]
pub struct UiPrefs {
    /// Accent `#rrggbb` (theme swatches or custom).
    pub accent: String,
    /// Window transparency, 0–100 % (Mica/CSS).
    pub transparency: u8,
    /// Glass blur, 0–100 %.
    pub blur: u8,
    pub animations: bool,
    pub avatar: bool,
    pub hud: bool,
    /// Listening pill at the top of the screen: «Слушаю…», live transcript, ✓ result.
    pub pill: bool,
    pub on_top: bool,
    /// Desktop avatar top-left corner, physical px; `None` = bottom-right corner (§3.6).
    pub avatar_pos: Option<[i32; 2]>,
}

impl Default for UiPrefs {
    fn default() -> Self {
        Self {
            accent: THEMES[0].1.into(),
            transparency: 30,
            blur: 90,
            animations: true,
            avatar: true,
            hud: false,
            pill: true,
            on_top: false,
            avatar_pos: None,
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
    fn theme_names_resolve() {
        assert_eq!(theme_hex("фиолетовую").as_deref(), Some("#8b5cf6"));
        assert_eq!(theme_hex("на зелёный").as_deref(), Some("#22c55e"));
        assert_eq!(theme_hex("Жёлтой").as_deref(), Some("#f59e0b"));
        assert_eq!(theme_hex("#AABBCC").as_deref(), Some("#aabbcc"));
        assert_eq!(theme_hex("квадратный"), None);
    }

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

    #[test]
    fn fish_follows_the_pack_unless_custom() {
        let fish = |o: &Online, e| match o.cloud_voice(e) {
            Some(CloudVoice::Fish { voice, .. }) => voice,
            v => panic!("{v:?}"),
        };
        let mut o = Online {
            fish_key: "k".into(),
            fish_voice: FILM_FISH.into(), // old saved default
            ..Online::default()
        };
        assert_eq!(fish(&o, VoiceEngine::Jarvis), JARVIS_FISH);
        assert_eq!(fish(&o, VoiceEngine::Film), FILM_FISH);
        o.fish_voice = "mine".into();
        assert_eq!(fish(&o, VoiceEngine::Film), "mine");
        o.eleven_key = "e".into();
        assert!(matches!(
            o.cloud_voice(VoiceEngine::Jarvis),
            Some(CloudVoice::Eleven { .. })
        ));
    }
}
