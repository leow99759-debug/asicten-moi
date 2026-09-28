//! Voice loop state machine (SPEC §2.1): wake word → STT → final phrase, single utterance
//! («Джарвис, включи музыку» in one breath), follow-up window without wake word, barge-in.
//! Pure logic: audio and time are injected, so it runs on WAV fixtures in tests.

use std::time::{Duration, Instant};

use crate::audio::{Ring, SAMPLE_RATE};
use crate::stt::LazyStt;
use crate::text;
use crate::vad::Vad;
use crate::wake::WakeWord;
use crate::Result;

/// Audio before the wake detection fed to STT, so the first command word isn't clipped.
const PRE_ROLL_SEC: f32 = 0.3;
/// Jarvis's last line still counts as echo this long after playback ended (room reverb, late STT).
const ECHO_KEEP: Duration = Duration::from_secs(4);

#[derive(Debug, Clone, PartialEq)]
pub enum Output {
    /// Wake word heard (play the short «слушаю» cue / animation now).
    Wake(f32),
    Partial(String),
    /// Complete utterance for the NLU.
    Final(String),
    /// Listening window passed with nothing said.
    Timeout,
    /// User interrupted Jarvis speaking («стоп»).
    BargeIn,
}

#[derive(Debug, Clone, Copy)]
pub struct ListenCfg {
    pub listen_timeout: Duration,
    pub followup: Duration,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum State {
    Idle,
    Listening { until: Instant, followup: bool },
    Speaking,
}

pub struct Listener {
    wake: WakeWord,
    vad: Vad,
    stt: LazyStt,
    cfg: ListenCfg,
    ring: Ring,
    state: State,
    prefix: bool,
    /// What Jarvis is saying / just said (normalized), stripped from what the mic hears.
    echo: String,
    /// None while he is still speaking.
    echo_until: Option<Instant>,
}

impl Listener {
    pub fn new(wake: WakeWord, vad: Vad, stt: LazyStt, cfg: ListenCfg) -> Self {
        Self {
            wake,
            vad,
            stt,
            cfg,
            ring: Ring::with_seconds(PRE_ROLL_SEC),
            state: State::Idle,
            prefix: true,
            echo: String::new(),
            echo_until: None,
        }
    }

    pub fn stt_mut(&mut self) -> &mut LazyStt {
        &mut self.stt
    }

    pub fn wake_mut(&mut self) -> &mut WakeWord {
        &mut self.wake
    }

    pub fn is_listening(&self) -> bool {
        matches!(self.state, State::Listening { .. })
    }

    /// Prefix mode off = every phrase is a command (STT stays loaded, SPEC §2.2).
    pub fn set_prefix(&mut self, on: bool) {
        self.prefix = on;
        self.stt.set_always_warm(!on);
        self.stt.reset();
        self.vad.flush();
    }

    /// Push-to-talk / mic button: listen now without the wake word.
    pub fn activate(&mut self, now: Instant) {
        self.start_listening(now, false);
    }

    /// Playback started (`said` = its words) or stopped. A reply after a command mutes the
    /// command path (only barge-in is heard); the «Да, сэр?» cue after the wake word keeps
    /// listening, so «Джарвис, включи музыку» in one breath still works. Either way the
    /// played words are stripped from what the mic hears. Stopping opens the follow-up window.
    pub fn set_speaking(&mut self, speaking: bool, said: &str, now: Instant) {
        if speaking {
            self.echo = text::normalize(said);
            self.echo_until = None;
            if self.state == State::Idle {
                self.state = State::Speaking;
                self.stt.begin();
                self.stt.reset();
            }
        } else {
            self.echo_until = Some(now + ECHO_KEEP);
            if self.state == State::Speaking {
                self.start_listening(now, true);
            }
        }
    }

    /// STT text minus Jarvis's own echo and a lone wake word; empty = nothing for the NLU.
    fn clean(&self, heard: &str, now: Instant) -> String {
        let echo_on = self.echo_until.is_none_or(|t| now < t);
        let norm = text::normalize(heard);
        let norm = if echo_on {
            text::strip_echo(&norm, &self.echo)
        } else {
            norm
        };
        if text::strip_wake(&norm).is_empty() {
            String::new()
        } else {
            norm
        }
    }

    /// Feed 16 kHz mono audio captured at `now`.
    pub fn push(&mut self, samples: &[i16], now: Instant) -> Result<Vec<Output>> {
        let mut out = Vec::new();
        match self.state {
            State::Idle if !self.prefix => {
                if self.feed(samples, &mut out)? {
                    let text = self.stt.finish()?;
                    let text = self.clean(&text, now);
                    if !text.is_empty() {
                        out.push(Output::Final(text));
                    }
                }
            }
            State::Idle => {
                self.ring.push(samples);
                if let Some(score) = self.wake.push(samples) {
                    out.push(Output::Wake(score));
                    self.start_listening(now, false);
                    let pre = self.ring.take();
                    self.feed(&pre, &mut out)?;
                }
            }
            State::Speaking => {
                // Only «стоп» interrupts: the wake model scores his own voice from the
                // speakers as high as a real «Джарвис» (0.57–0.62 on voice-jarvis clips).
                let stop = self
                    .stt
                    .push(samples)?
                    .is_some_and(|p| p.split_whitespace().any(|w| w == "стоп"));
                if stop {
                    out.push(Output::BargeIn);
                    self.start_listening(now, false);
                }
            }
            State::Listening { until, followup } => {
                let ended = self.feed(samples, &mut out)?;
                if ended || now >= until {
                    let heard = self.stt.finish()?;
                    let text = self.clean(&heard, now);
                    if text.is_empty() && now < until {
                        // only his echo, the wake word or a cough: keep waiting for the command
                        self.stt.begin();
                        self.stt.reset();
                        return Ok(out);
                    }
                    self.vad.flush();
                    self.state = State::Idle;
                    self.wake.reset();
                    if !text.is_empty() {
                        out.push(Output::Final(text));
                    } else if !followup {
                        out.push(Output::Timeout);
                    }
                }
            }
        }
        Ok(out)
    }

    /// Feed STT + VAD; true when the phrase ended (VAD closed a speech segment).
    fn feed(&mut self, samples: &[i16], out: &mut Vec<Output>) -> Result<bool> {
        if let Some(p) = self.stt.push(samples)? {
            out.push(Output::Partial(p));
        }
        Ok(!self.vad.push(samples).is_empty())
    }

    fn start_listening(&mut self, now: Instant, followup: bool) {
        let window = if followup {
            self.cfg.followup
        } else {
            self.cfg.listen_timeout
        };
        self.stt.begin();
        self.stt.reset();
        self.vad.flush();
        self.state = State::Listening {
            until: now + window,
            followup,
        };
    }
}

/// Duration of `n` samples at the pipeline rate.
pub fn samples_to_duration(n: usize) -> Duration {
    Duration::from_secs_f64(n as f64 / f64::from(SAMPLE_RATE))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::stt::MODEL_DIR;
    use crate::test_util::{asset, fixture, read_wav_16k};

    struct Rig {
        l: Listener,
        t0: Instant,
        fed: usize,
    }

    impl Rig {
        fn new() -> Option<Self> {
            let (Some(rpw), Some(vad), Some(stt)) = (
                asset("rustpotter-jarvis/jarvis-default.rpw"),
                asset("silero_vad.onnx"),
                asset(MODEL_DIR),
            ) else {
                eprintln!("assets missing; skipping");
                return None;
            };
            let cfg = ListenCfg {
                listen_timeout: Duration::from_secs(6),
                followup: Duration::from_secs(5),
            };
            let l = Listener::new(
                WakeWord::new(&rpw, 70).expect("wake"),
                Vad::new(&vad, 10.0).expect("vad"),
                LazyStt::new(stt, Duration::from_secs(120)),
                cfg,
            );
            Some(Self {
                l,
                t0: Instant::now(),
                fed: 0,
            })
        }

        fn now(&self) -> Instant {
            self.t0 + samples_to_duration(self.fed)
        }

        fn feed(&mut self, audio: &[i16]) -> Vec<Output> {
            let mut out = Vec::new();
            for c in audio.chunks(1600) {
                self.fed += c.len();
                let now = self.now();
                out.extend(self.l.push(c, now).expect("push"));
            }
            out
        }

        fn say(&mut self, name: &str) -> Vec<Output> {
            let mut a = read_wav_16k(&fixture(&format!("wake/{name}.wav")));
            a.extend(vec![0i16; SAMPLE_RATE as usize * 2]);
            self.feed(&a)
        }

        fn silence(&mut self, secs: usize) -> Vec<Output> {
            self.feed(&vec![0i16; SAMPLE_RATE as usize * secs])
        }
    }

    fn finals(out: &[Output]) -> Vec<String> {
        out.iter()
            .filter_map(|o| match o {
                Output::Final(t) => Some(t.clone()),
                _ => None,
            })
            .collect()
    }

    #[test]
    fn single_utterance_followup_and_bargein() {
        let Some(mut r) = Rig::new() else { return };

        // «Джарвис, включи музыку» in one breath
        let out = r.say("jarvis_music");
        assert!(matches!(out.first(), Some(Output::Wake(_))), "{out:?}");
        // cold start: STT still loading, so partials may be absent; the final must be complete
        let f = finals(&out);
        assert_eq!(f.len(), 1, "{out:?}");
        assert!(f[0].ends_with("музыку"), "{f:?}");

        // no wake word while idle → nothing
        assert!(finals(&r.say("neg_hello")).is_empty());

        // Jarvis answers, then follow-up without wake word
        let now = r.now();
        r.l.set_speaking(true, "", now);
        r.silence(1);
        let now = r.now();
        r.l.set_speaking(false, "", now);
        assert_eq!(finals(&r.say("neg_browser")), vec!["открой браузер"]);

        // follow-up window expires silently
        let now = r.now();
        r.l.set_speaking(true, "", now);
        let now = r.now();
        r.l.set_speaking(false, "", now);
        let out = r.silence(6);
        assert!(
            !out.contains(&Output::Timeout) && finals(&out).is_empty(),
            "{out:?}"
        );
        assert!(finals(&r.say("neg_hello")).is_empty());

        // the wake word doesn't cut him off: his own voice would trigger it too
        let now = r.now();
        r.l.set_speaking(true, "", now);
        let out = r.say("jarvis");
        assert!(!out.contains(&Output::BargeIn), "{out:?}");
    }

    /// Real bug: his «Да, сэр» cue came back through the mic, became a command («да сэр» →
    /// «Чего вы пытаетесь добиться?») and closed the listening window.
    #[test]
    fn own_voice_is_not_a_command() {
        let Some(mut r) = Rig::new() else { return };
        let Some(cue) = asset("voice-jarvis/ru/reply/js_33.wav") else {
            return;
        };
        let cue = read_wav_16k(&cue);
        let out = r.say("jarvis");
        assert!(matches!(out.first(), Some(Output::Wake(_))), "{out:?}");
        // «Да, сэр» plays while listening; the mic hears it
        let now = r.now();
        r.l.set_speaking(true, "Да, сэр", now);
        let mut out = r.feed(&cue);
        let now = r.now();
        r.l.set_speaking(false, "", now);
        out.extend(r.silence(1));
        assert!(finals(&out).is_empty(), "echo became a command: {out:?}");
        assert!(r.l.is_listening(), "window closed on echo");
        // the real command still lands in the same window
        assert_eq!(finals(&r.say("neg_browser")), vec!["открой браузер"]);
    }

    #[test]
    fn no_prefix_mode_takes_every_phrase() {
        let Some(mut r) = Rig::new() else { return };
        r.l.set_prefix(false);
        assert_eq!(finals(&r.say("neg_browser")), vec!["открой браузер"]);
        assert!(finals(&r.silence(3)).is_empty());
        r.l.set_prefix(true);
        assert!(finals(&r.say("neg_browser")).is_empty());
    }

    #[test]
    fn wake_then_silence_times_out() {
        let Some(mut r) = Rig::new() else { return };
        let now = r.now();
        r.l.activate(now);
        let out = r.silence(7);
        assert_eq!(out, vec![Output::Timeout]);
        assert!(!r.l.is_listening());
    }
}
