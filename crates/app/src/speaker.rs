//! Voice output front: phrase pack clips (§6.1) with a text fallback. The UI always gets
//! the line as a `Say` event; silent mode (§2.3) keeps it text-only.
//! Text without a clip goes to the neural voice (Piper via sherpa-onnx, §6.2), sentence by sentence.

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use jarvis_core::brain::Line;
use jarvis_core::config::VoiceEngine;
use jarvis_core::ipc::{CoreEvent, EventSink};
use jarvis_core::tts::{self, Tts};
use jarvis_core::voice::{self, Player, VoicePack};
use jarvis_core::Config;

pub struct Speaker {
    pack: Option<VoicePack>,
    player: Option<Player>,
    tts: Option<Arc<Tts>>,
    sink: Arc<dyn EventSink>,
    config: Arc<Mutex<Config>>,
    /// Words of the last line sent to the speakers (echo filter in the listener).
    said: Mutex<String>,
}

/// Preferred voice pack under assets: our curated Jarvis pack, else Priler's original.
fn pack_dir(assets: &Path) -> Option<PathBuf> {
    ["voice-jarvis", "voices-priler/jarvis-og"]
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
        let tts_dir = assets.join(tts::DEFAULT_MODEL_DIR);
        let tts = tts_dir.exists().then(|| Arc::new(Tts::new(tts_dir)));
        if let Some(t) = tts.clone() {
            // unload the neural voice after a minute of silence
            let _ = std::thread::Builder::new()
                .name("jarvis-tts-idle".into())
                .spawn(move || loop {
                    std::thread::sleep(Duration::from_secs(15));
                    t.unload_if_idle(Duration::from_secs(60));
                });
        }
        Self {
            pack,
            player,
            tts,
            sink,
            config,
            said: Mutex::default(),
        }
    }

    pub fn say(&self, line: &Line) {
        let (silent, volume, speed, fx, engine) = {
            let c = self.config.lock().unwrap_or_else(|e| e.into_inner());
            (
                c.silent_mode,
                c.voice_volume,
                c.voice_speed,
                c.voice_fx,
                c.voice_engine,
            )
        };
        // no recording (or Windows voice): speak the category's words
        let text = line
            .text
            .as_deref()
            .map(|t| voice::pick_variant(t).to_owned())
            .or_else(|| {
                line.clips
                    .iter()
                    .find_map(|c| voice::category_text(c))
                    .map(str::to_owned)
            });
        // category recording first (variety), then a recording of this exact text
        let clip = match (engine, &self.pack) {
            (VoiceEngine::Jarvis, Some(p)) => p
                .pick(&line.clips)
                .or_else(|| text.as_deref().and_then(|t| p.by_text(t))),
            _ => None,
        };
        if let Some(t) = text.clone().or_else(|| {
            clip.and_then(|c| c.file_stem())
                .map(|s| s.to_string_lossy().into_owned())
        }) {
            self.sink.emit(CoreEvent::Say(t));
        }
        let Some(player) = self.player.as_ref().filter(|_| !silent) else {
            return;
        };
        let spoken = clip
            .and_then(|c| self.pack.as_ref().and_then(|p| p.text_of(c)))
            .map(str::to_owned)
            .or_else(|| text.clone())
            .unwrap_or_default();
        *self.said.lock().unwrap_or_else(|e| e.into_inner()) = spoken;
        player.set_volume(f32::from(volume) / 100.0);
        if let Some(clip) = clip {
            if let Err(e) = player.play_wav(clip) {
                tracing::warn!("play {}: {e}", clip.display());
            }
            return;
        }
        let Some(text) = text else { return };
        for sentence in tts::sentences(&text) {
            let neural = match (engine, &self.tts) {
                (VoiceEngine::Jarvis, Some(t)) => t
                    .synth(&sentence, speed)
                    .map(|(s, rate)| {
                        if fx {
                            (tts::movie_fx(&s, rate), rate)
                        } else {
                            (s, rate)
                        }
                    })
                    .map_err(|e| tracing::warn!("tts: {e}"))
                    .ok(),
                _ => None,
            };
            // Windows voice: chosen engine or fallback when Piper is missing/broken (§6.3)
            let audio = neural.or_else(|| {
                jarvis_win::speech::synth_wav(&sentence, speed)
                    .map_err(|e| tracing::warn!("windows voice: {e}"))
                    .ok()
                    .and_then(|wav| voice::decode_wav(std::io::Cursor::new(wav)).ok())
            });
            match audio {
                Some((s, rate)) => player.play_samples(&s, rate),
                None => return,
            }
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

    /// Audio is queued for the speakers.
    pub fn is_playing(&self) -> bool {
        self.player.as_ref().is_some_and(Player::is_playing)
    }

    /// Words of the last line played.
    pub fn said(&self) -> String {
        self.said.lock().unwrap_or_else(|e| e.into_inner()).clone()
    }

    /// Output level 0..1 for the orb.
    pub fn level(&self) -> f32 {
        self.player.as_ref().map_or(0.0, Player::level)
    }
}
