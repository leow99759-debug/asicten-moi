//! Voice output front: phrase pack clips (§6.1) with a text fallback. The UI always gets
//! the line as a `Say` event; silent mode (§2.3) keeps it text-only.
//! TTS for arbitrary text arrives in T041; until then text lines are UI-only.

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use jarvis_core::brain::Line;
use jarvis_core::ipc::{CoreEvent, EventSink};
use jarvis_core::voice::{Player, VoicePack};
use jarvis_core::Config;

pub struct Speaker {
    pack: Option<VoicePack>,
    player: Option<Player>,
    sink: Arc<dyn EventSink>,
    config: Arc<Mutex<Config>>,
}

/// Preferred voice pack under assets: our curated Jarvis pack, else Priler's original.
fn pack_dir(assets: &Path) -> Option<PathBuf> {
    ["voices/jarvis", "voices-priler/jarvis-og"]
        .iter()
        .map(|rel| assets.join(rel))
        .find(|p| p.exists())
}

impl Speaker {
    pub fn new(assets: &Path, sink: Arc<dyn EventSink>, config: Arc<Mutex<Config>>) -> Self {
        let pack = pack_dir(assets).and_then(|d| match VoicePack::load(&d, "ru") {
            Ok(p) => {
                tracing::info!(dir = %d.display(), categories = ?p.categories().collect::<Vec<_>>(), "voice pack");
                Some(p)
            }
            Err(e) => {
                tracing::warn!("voice pack: {e}");
                None
            }
        });
        let player = Player::new()
            .map_err(|e| tracing::warn!("speaker: {e}"))
            .ok();
        Self {
            pack,
            player,
            sink,
            config,
        }
    }

    pub fn say(&self, line: &Line) {
        let clip = self.pack.as_ref().and_then(|p| p.pick(&line.clips));
        let shown = line.text.clone().or_else(|| {
            clip.and_then(|c| c.file_stem())
                .map(|s| s.to_string_lossy().into_owned())
        });
        if let Some(t) = shown {
            self.sink.emit(CoreEvent::Say(t));
        }
        let (silent, volume) = {
            let c = self.config.lock().unwrap_or_else(|e| e.into_inner());
            (c.silent_mode, c.voice_volume)
        };
        let (Some(player), Some(clip), false) = (&self.player, clip, silent) else {
            return;
        };
        player.set_volume(f32::from(volume) / 100.0);
        if let Err(e) = player.play_wav(clip) {
            tracing::warn!("play {}: {e}", clip.display());
        }
    }

    pub fn say_text(&self, clips: &[&str], text: &str) {
        self.say(&Line {
            clips: clips.iter().map(|c| (*c).to_owned()).collect(),
            text: Some(text.to_owned()),
        });
    }

    /// Stop talking (barge-in, «отмена»).
    pub fn stop(&self) {
        if let Some(p) = &self.player {
            p.stop();
        }
    }

    /// Output level 0..1 for the orb.
    pub fn level(&self) -> f32 {
        self.player.as_ref().map_or(0.0, Player::level)
    }
}
