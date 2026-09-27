//! Voice engine thread: mic capture → Listener → core events; modes from UI/hotkeys/voice.

use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Receiver, RecvTimeoutError, Sender};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use anyhow::Context;
use jarvis_core::audio;
use jarvis_core::brain::Line;
use jarvis_core::ipc::{AssistantState, CoreEvent, EventSink, Level, Transcript};
use jarvis_core::listener::{ListenCfg, Listener, Output};
use jarvis_core::modes::{self, ModeCommand};
use jarvis_core::nlu::chain::yes_no;
use jarvis_core::stt::{LazyStt, MODEL_DIR};
use jarvis_core::vad::Vad;
use jarvis_core::wake::WakeWord;
use jarvis_core::Config;

use crate::brain_worker::{Confirm, Work};

/// Level events at most this often (orb/listening bar).
const LEVEL_EVERY: Duration = Duration::from_millis(66);

pub enum Msg {
    Audio(Vec<i16>),
    /// Push-to-talk / mic button.
    Activate,
    Mode(ModeCommand),
    /// TTS started/stopped (from the voice output, M3).
    Speaking(bool),
    /// Settings saved: re-read wake sensitivity, mic device, prefix/memory options.
    Reconfigure,
}

/// Handle to the engine thread.
#[derive(Clone)]
pub struct Engine {
    tx: Sender<Msg>,
}

impl Engine {
    pub fn send(&self, msg: Msg) {
        if self.tx.send(msg).is_err() {
            tracing::warn!("engine thread is gone");
        }
    }
}

/// Apply a mode switch to the shared config and persist it.
pub fn apply_mode(config: &Mutex<Config>, path: &Path, cmd: ModeCommand) -> Config {
    let mut cfg = config.lock().unwrap_or_else(|e| e.into_inner());
    cmd.apply(&mut cfg);
    if let Err(err) = cfg.save(path) {
        tracing::warn!(%err, "config save failed");
    }
    cfg.clone()
}

/// Where final utterances go.
pub struct Route {
    pub work: Sender<Work>,
    pub confirm: Arc<Confirm>,
    pub speaker: Arc<crate::speaker::Speaker>,
}

pub fn spawn(
    assets: PathBuf,
    config: Arc<Mutex<Config>>,
    config_path: PathBuf,
    sink: Arc<dyn EventSink>,
    route: Route,
) -> Engine {
    let (tx, rx) = mpsc::channel();
    let engine = Engine { tx: tx.clone() };
    let result = std::thread::Builder::new()
        .name("jarvis-engine".into())
        .spawn(move || {
            if let Err(err) = run(&assets, &config, &config_path, &*sink, &route, tx, rx) {
                tracing::error!("engine stopped: {err:#}");
                sink.emit(CoreEvent::State(AssistantState::MicOff));
            }
        });
    if let Err(err) = result {
        tracing::error!(%err, "engine thread spawn failed");
    }
    engine
}

fn run(
    assets: &Path,
    config: &Mutex<Config>,
    config_path: &Path,
    sink: &dyn EventSink,
    route: &Route,
    tx: Sender<Msg>,
    rx: Receiver<Msg>,
) -> anyhow::Result<()> {
    let cfg = config.lock().unwrap_or_else(|e| e.into_inner()).clone();
    let listen = ListenCfg {
        listen_timeout: Duration::from_secs(cfg.listen_timeout_sec.into()),
        followup: Duration::from_secs(cfg.followup_sec.into()),
    };
    let mut listener = Listener::new(
        WakeWord::new(
            &assets.join("rustpotter-jarvis/jarvis-default.rpw"),
            cfg.wake_sensitivity,
        )
        .context("wake word")?,
        Vad::new(
            &assets.join("silero_vad.onnx"),
            listen.listen_timeout.as_secs_f32(),
        )
        .context("vad")?,
        LazyStt::new(
            assets.join(MODEL_DIR),
            Duration::from_secs(cfg.stt_keep_warm_sec.into()),
        ),
        listen,
    );
    listener.set_prefix(cfg.prefix_mode);
    listener
        .stt_mut()
        .set_always_warm(!cfg.memory_saver || !cfg.prefix_mode);

    let mut mic = Mic::new(tx);
    mic.set(cfg.mic_enabled, cfg.mic_device.as_deref());
    let mut state = AssistantState::Idle;
    let mut last_level = Instant::now();
    let set_state = |s: AssistantState, state: &mut AssistantState| {
        if *state != s {
            *state = s;
            sink.emit(CoreEvent::State(s));
        }
    };
    set_state(
        if cfg.mic_enabled {
            AssistantState::Idle
        } else {
            AssistantState::MicOff
        },
        &mut state,
    );

    loop {
        let msg = match rx.recv_timeout(Duration::from_secs(1)) {
            Ok(m) => m,
            Err(RecvTimeoutError::Timeout) => {
                listener.stt_mut().tick(Instant::now());
                continue;
            }
            Err(RecvTimeoutError::Disconnected) => return Ok(()),
        };
        let now = Instant::now();
        let outputs = match msg {
            Msg::Audio(chunk) => {
                if now.duration_since(last_level) >= LEVEL_EVERY {
                    last_level = now;
                    sink.emit(CoreEvent::Level(Level {
                        mic: audio::level(&chunk),
                        tts: route.speaker.level(),
                    }));
                }
                listener.push(&chunk, now)?
            }
            Msg::Activate => {
                listener.activate(now);
                vec![]
            }
            Msg::Speaking(on) => {
                listener.set_speaking(on, now);
                set_state(
                    if on {
                        AssistantState::Speaking
                    } else {
                        AssistantState::Listening
                    },
                    &mut state,
                );
                vec![]
            }
            Msg::Reconfigure => {
                let cfg = config.lock().unwrap_or_else(|e| e.into_inner()).clone();
                listener.wake_mut().set_sensitivity(cfg.wake_sensitivity);
                apply_to_runtime(&cfg, &mut listener, &mut mic);
                vec![]
            }
            Msg::Mode(cmd) => {
                let cfg = apply_mode(config, config_path, cmd);
                apply_to_runtime(&cfg, &mut listener, &mut mic);
                vec![]
            }
        };
        for out in outputs {
            match out {
                Output::Wake(score) => {
                    tracing::info!(score, "wake word");
                    route.speaker.stop();
                    route.speaker.say(&Line {
                        clips: vec!["reply".into()],
                        text: None,
                    });
                    set_state(AssistantState::Listening, &mut state);
                }
                Output::BargeIn => {
                    route.speaker.stop();
                    set_state(AssistantState::Listening, &mut state);
                }
                Output::Partial(text) => sink.emit(CoreEvent::Transcript(Transcript {
                    text,
                    is_final: false,
                })),
                Output::Timeout => {}
                Output::Final(text) => {
                    tracing::info!(%text, "utterance");
                    sink.emit(CoreEvent::Transcript(Transcript {
                        text: text.clone(),
                        is_final: true,
                    }));
                    let phrases = config
                        .lock()
                        .unwrap_or_else(|e| e.into_inner())
                        .mode_phrases
                        .clone();
                    if route.confirm.is_pending() {
                        if let Some(yes) = yes_no(&text) {
                            route.confirm.answer(yes);
                        }
                    } else if let Some(cmd) = modes::from_phrase(&text, &phrases) {
                        let cfg = apply_mode(config, config_path, cmd);
                        apply_to_runtime(&cfg, &mut listener, &mut mic);
                        route.speaker.say(&Line {
                            clips: vec!["ok".into()],
                            text: None,
                        });
                    } else if route.work.send(Work::Utterance(text)).is_err() {
                        tracing::warn!("command worker is gone");
                    }
                }
            }
        }
        if !listener.is_listening() && state == AssistantState::Listening {
            set_state(AssistantState::Idle, &mut state);
        }
        if !mic.is_on() {
            set_state(AssistantState::MicOff, &mut state);
        } else if state == AssistantState::MicOff {
            set_state(AssistantState::Idle, &mut state);
        }
    }
}

fn apply_to_runtime(cfg: &Config, listener: &mut Listener, mic: &mut Mic) {
    listener.set_prefix(cfg.prefix_mode);
    listener
        .stt_mut()
        .set_always_warm(!cfg.memory_saver || !cfg.prefix_mode);
    mic.set(cfg.mic_enabled, cfg.mic_device.as_deref());
}

/// Microphone on/off: off = stream closed completely (SPEC §2.2 «Выключить микро»).
struct Mic {
    tx: Sender<Msg>,
    #[cfg(windows)]
    capture: Option<audio::Capture>,
}

impl Mic {
    fn new(tx: Sender<Msg>) -> Self {
        Self {
            tx,
            #[cfg(windows)]
            capture: None,
        }
    }

    #[cfg(windows)]
    fn set(&mut self, on: bool, device: Option<&str>) {
        if !on {
            self.capture = None;
            return;
        }
        if self.capture.is_some() {
            return;
        }
        let tx = self.tx.clone();
        match audio::Capture::start(device, move |chunk| {
            let _ = tx.send(Msg::Audio(chunk.to_vec()));
        }) {
            Ok(c) => self.capture = Some(c),
            Err(err) => tracing::error!(%err, "mic start failed"),
        }
    }

    #[cfg(windows)]
    fn is_on(&self) -> bool {
        self.capture.is_some()
    }

    #[cfg(not(windows))]
    fn set(&mut self, _on: bool, _device: Option<&str>) {
        let _ = &self.tx;
        tracing::error!("mic capture is Windows-only");
    }

    #[cfg(not(windows))]
    fn is_on(&self) -> bool {
        false
    }
}
