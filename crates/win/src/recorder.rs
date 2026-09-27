//! Action recorder (SPEC §5.4): global low-level hooks → editable steps
//! (`Mouse.ClickLeft 642 518`, `Lux.PauseMS 850`, `Keys.Type`, `Keys.Press Ctrl+S`).
//! The event → action logic is pure and tested; the hooks are Windows-only.
//! Input on Jarvis's own windows and injected input (our own playback) is ignored.

use jarvis_core::commands::{Action, Num};

use crate::keys;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Button {
    Left,
    Right,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Mods {
    pub ctrl: bool,
    pub shift: bool,
    pub alt: bool,
    pub win: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Event {
    Click {
        x: i32,
        y: i32,
        button: Button,
    },
    /// Raw wheel delta, 120 per notch (touchpads send less).
    Wheel {
        delta: i32,
    },
    /// Key down with the modifiers held and the character it types in the current layout.
    Key {
        vk: u16,
        ch: Option<char>,
        mods: Mods,
    },
}

/// Shorter gaps are hand jitter, not worth a pause step.
const MIN_PAUSE_MS: u32 = 200;
const DOUBLE_MS: u32 = 500;
const DOUBLE_PX: i32 = 4;
const WHEEL_MERGE_MS: u32 = 400;
const BACKSPACE: u16 = 0x08;

/// Folds timed events into steps: typed characters → one `Keys.Type`, chords → `Keys.Press`,
/// two quick clicks → `Mouse.Double`, a wheel burst → one `Mouse.Scroll`, gaps → pauses.
#[derive(Default)]
pub struct Recorder {
    steps: Vec<Action>,
    /// Time of the last event that produced or extended a step.
    last: Option<u32>,
    text: String,
    wheel: i32,
    last_click: Option<(u32, i32, i32)>,
}

impl Recorder {
    pub fn push(&mut self, t: u32, ev: Event) {
        if !matches!(ev, Event::Wheel { .. }) {
            self.flush_wheel();
        }
        match ev {
            Event::Key { vk, ch, mods } => self.key(t, vk, ch, mods),
            Event::Click { x, y, button } => {
                self.flush_text();
                if button == Button::Left {
                    let double = self.last_click.is_some_and(|(pt, px, py)| {
                        t.wrapping_sub(pt) <= DOUBLE_MS
                            && (x - px).abs() <= DOUBLE_PX
                            && (y - py).abs() <= DOUBLE_PX
                    });
                    if double {
                        if let Some(last @ Action::MouseClickLeft { .. }) = self.steps.last_mut() {
                            *last = Action::MouseDouble {
                                x: Some(Num::Value(x.into())),
                                y: Some(Num::Value(y.into())),
                            };
                            self.last_click = None;
                            self.last = Some(t);
                            return;
                        }
                    }
                    self.last_click = Some((t, x, y));
                }
                self.gap(t);
                let (x, y) = (Some(Num::Value(x.into())), Some(Num::Value(y.into())));
                self.steps.push(match button {
                    Button::Left => Action::MouseClickLeft { x, y },
                    Button::Right => Action::MouseClickRight { x, y },
                });
            }
            Event::Wheel { delta } => {
                self.flush_text();
                let burst = self.wheel != 0
                    && self
                        .last
                        .is_some_and(|l| t.wrapping_sub(l) <= WHEEL_MERGE_MS);
                if burst {
                    self.last = Some(t);
                } else {
                    self.flush_wheel();
                    self.gap(t);
                }
                self.wheel += delta;
            }
        }
    }

    fn key(&mut self, t: u32, vk: u16, ch: Option<char>, mods: Mods) {
        let chord = mods.ctrl || mods.alt || mods.win;
        match ch.filter(|c| !c.is_control()) {
            Some(c) if !chord => {
                if self.text.is_empty() {
                    self.gap(t);
                } else {
                    self.last = Some(t);
                }
                self.text.push(c);
            }
            _ if vk == BACKSPACE && !chord && self.text.pop().is_some() => self.last = Some(t),
            _ => {
                // lone modifiers and keys without a name (OEM punctuation in chords) are skipped
                let Some(key) = keys::name(vk)
                    .filter(|n| !matches!(n.as_str(), "Ctrl" | "Shift" | "Alt" | "Win"))
                else {
                    return;
                };
                self.flush_text();
                self.gap(t);
                let held = [
                    (mods.ctrl, "Ctrl"),
                    (mods.shift, "Shift"),
                    (mods.alt, "Alt"),
                    (mods.win, "Win"),
                ];
                let mut combo: Vec<&str> = held.iter().filter(|m| m.0).map(|m| m.1).collect();
                combo.push(&key);
                self.steps.push(Action::KeysPress {
                    keys: combo.join("+"),
                });
            }
        }
    }

    fn gap(&mut self, t: u32) {
        if let Some(l) = self.last {
            let ms = t.wrapping_sub(l);
            if ms >= MIN_PAUSE_MS {
                let ms = (f64::from(ms) / 50.0).round() * 50.0;
                self.steps.push(Action::PauseMs { ms: Num::Value(ms) });
            }
        }
        self.last = Some(t);
    }

    fn flush_text(&mut self) {
        if !self.text.is_empty() {
            self.steps.push(Action::KeysType {
                text: std::mem::take(&mut self.text),
            });
        }
    }

    fn flush_wheel(&mut self) {
        let notches = (f64::from(self.wheel) / 120.0).round();
        self.wheel = 0;
        if notches != 0.0 {
            self.steps.push(Action::MouseScroll {
                dy: Num::Value(notches),
            });
        }
    }

    pub fn finish(mut self) -> Vec<Action> {
        self.flush_text();
        self.flush_wheel();
        // a pause right before the end (moving to «Стоп») is useless
        while matches!(self.steps.last(), Some(Action::PauseMs { .. })) {
            self.steps.pop();
        }
        self.steps
    }
}

#[cfg(windows)]
pub use hooks::{start, Recording};

#[cfg(not(windows))]
pub struct Recording;

#[cfg(not(windows))]
impl Recording {
    pub fn stop(self) -> Vec<Action> {
        Vec::new()
    }
}

#[cfg(not(windows))]
pub fn start() -> Result<Recording, String> {
    Err("запись действий работает только в Windows".into())
}

#[cfg(windows)]
mod hooks {
    use std::cell::RefCell;
    use std::sync::mpsc;
    use std::thread::JoinHandle;

    use jarvis_core::commands::Action;
    use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, POINT, WPARAM};
    use windows::Win32::System::Threading::{GetCurrentProcessId, GetCurrentThreadId};
    use windows::Win32::UI::Input::KeyboardAndMouse::{
        GetAsyncKeyState, GetKeyState, GetKeyboardLayout, ToUnicodeEx, VK_CAPITAL, VK_CONTROL,
        VK_LWIN, VK_MENU, VK_RWIN, VK_SHIFT,
    };
    use windows::Win32::UI::WindowsAndMessaging::{
        CallNextHookEx, GetAncestor, GetForegroundWindow, GetMessageW, GetWindowThreadProcessId,
        PostThreadMessageW, SetWindowsHookExW, UnhookWindowsHookEx, WindowFromPoint, GA_ROOT,
        KBDLLHOOKSTRUCT, LLKHF_INJECTED, LLMHF_INJECTED, MSG, MSLLHOOKSTRUCT, WH_KEYBOARD_LL,
        WH_MOUSE_LL, WM_KEYDOWN, WM_LBUTTONDOWN, WM_MOUSEWHEEL, WM_QUIT, WM_RBUTTONDOWN,
        WM_SYSKEYDOWN,
    };

    use super::{Button, Event, Mods, Recorder};

    thread_local! {
        static REC: RefCell<Option<Recorder>> = const { RefCell::new(None) };
    }

    pub struct Recording {
        thread: u32,
        join: JoinHandle<Vec<Action>>,
    }

    impl Recording {
        /// Stop the hooks and return the recorded steps.
        pub fn stop(self) -> Vec<Action> {
            // SAFETY: posting to a thread id we own; failure only means it already exited
            unsafe {
                let _ = PostThreadMessageW(self.thread, WM_QUIT, WPARAM(0), LPARAM(0));
            }
            self.join.join().unwrap_or_default()
        }
    }

    /// Install the hooks on a dedicated message-loop thread.
    pub fn start() -> Result<Recording, String> {
        let (ready_tx, ready_rx) = mpsc::channel();
        let join = std::thread::Builder::new()
            .name("jarvis-recorder".into())
            .spawn(move || {
                REC.with(|r| *r.borrow_mut() = Some(Recorder::default()));
                // SAFETY: plain Win32 calls; hook procs run on this thread's message loop
                unsafe {
                    let mouse = SetWindowsHookExW(WH_MOUSE_LL, Some(mouse_proc), None, 0);
                    let key = SetWindowsHookExW(WH_KEYBOARD_LL, Some(key_proc), None, 0);
                    match (mouse, key) {
                        (Ok(m), Ok(k)) => {
                            let _ = ready_tx.send(Ok(GetCurrentThreadId()));
                            let mut msg = MSG::default();
                            while GetMessageW(&mut msg, None, 0, 0).as_bool() {}
                            let _ = UnhookWindowsHookEx(m);
                            let _ = UnhookWindowsHookEx(k);
                        }
                        (m, k) => {
                            let err = m
                                .as_ref()
                                .err()
                                .or(k.as_ref().err())
                                .map(ToString::to_string);
                            for h in [m, k].into_iter().flatten() {
                                let _ = UnhookWindowsHookEx(h);
                            }
                            let _ = ready_tx.send(Err(err.unwrap_or_default()));
                        }
                    }
                }
                REC.with(|r| r.borrow_mut().take())
                    .map(Recorder::finish)
                    .unwrap_or_default()
            })
            .map_err(|e| e.to_string())?;
        let thread = ready_rx
            .recv()
            .map_err(|e| e.to_string())?
            .map_err(|e| format!("не удалось включить запись: {e}"))?;
        Ok(Recording { thread, join })
    }

    fn record(t: u32, ev: Event) {
        REC.with(|r| {
            if let Some(r) = r.borrow_mut().as_mut() {
                r.push(t, ev);
            }
        });
    }

    fn own(hwnd: HWND) -> bool {
        let mut pid = 0;
        // SAFETY: pid is a valid out pointer
        unsafe {
            GetWindowThreadProcessId(hwnd, Some(&mut pid));
            pid == GetCurrentProcessId()
        }
    }

    fn down(vk: u16) -> bool {
        // SAFETY: no pointers involved
        unsafe { GetAsyncKeyState(vk.into()) < 0 }
    }

    unsafe extern "system" fn mouse_proc(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
        if code >= 0 {
            // SAFETY: for WH_MOUSE_LL, lparam points to an MSLLHOOKSTRUCT
            let info = unsafe { &*(lparam.0 as *const MSLLHOOKSTRUCT) };
            let POINT { x, y } = info.pt;
            // SAFETY: plain window queries
            let mine = unsafe { own(GetAncestor(WindowFromPoint(info.pt), GA_ROOT)) };
            if info.flags & LLMHF_INJECTED == 0 && !mine {
                let ev = match wparam.0 as u32 {
                    WM_LBUTTONDOWN => Some(Event::Click {
                        x,
                        y,
                        button: Button::Left,
                    }),
                    WM_RBUTTONDOWN => Some(Event::Click {
                        x,
                        y,
                        button: Button::Right,
                    }),
                    WM_MOUSEWHEEL => Some(Event::Wheel {
                        delta: i32::from((info.mouseData >> 16) as i16),
                    }),
                    _ => None,
                };
                if let Some(ev) = ev {
                    record(info.time, ev);
                }
            }
        }
        // SAFETY: forwarding the hook chain unchanged
        unsafe { CallNextHookEx(None, code, wparam, lparam) }
    }

    unsafe extern "system" fn key_proc(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
        let is_down = matches!(wparam.0 as u32, WM_KEYDOWN | WM_SYSKEYDOWN);
        if code >= 0 && is_down {
            // SAFETY: for WH_KEYBOARD_LL, lparam points to a KBDLLHOOKSTRUCT
            let info = unsafe { &*(lparam.0 as *const KBDLLHOOKSTRUCT) };
            // SAFETY: plain window query
            let fg = unsafe { GetForegroundWindow() };
            if (info.flags & LLKHF_INJECTED).0 == 0 && !own(fg) {
                let mods = Mods {
                    ctrl: down(VK_CONTROL.0),
                    shift: down(VK_SHIFT.0),
                    alt: down(VK_MENU.0),
                    win: down(VK_LWIN.0) || down(VK_RWIN.0),
                };
                let vk = info.vkCode as u16;
                record(
                    info.time,
                    Event::Key {
                        vk,
                        ch: typed(vk, info.scanCode, mods, fg),
                        mods,
                    },
                );
            }
        }
        // SAFETY: forwarding the hook chain unchanged
        unsafe { CallNextHookEx(None, code, wparam, lparam) }
    }

    /// Character the key types in the foreground window's layout (Cyrillic too).
    fn typed(vk: u16, scan: u32, mods: Mods, fg: HWND) -> Option<char> {
        if mods.ctrl || mods.alt || mods.win {
            return None;
        }
        let mut state = [0u8; 256];
        if mods.shift {
            state[usize::from(VK_SHIFT.0)] = 0x80;
        }
        // SAFETY: plain Win32 queries; buffers outlive the calls
        unsafe {
            if GetKeyState(VK_CAPITAL.0.into()) & 1 == 1 {
                state[usize::from(VK_CAPITAL.0)] = 0x01;
            }
            let layout = GetKeyboardLayout(GetWindowThreadProcessId(fg, None));
            let mut buf = [0u16; 4];
            // flag 4: don't touch the keyboard state (keeps dead keys working for the user)
            let n = ToUnicodeEx(vk.into(), scan, &state, &mut buf, 4, Some(layout));
            (n == 1).then(|| char::from_u32(buf[0].into())).flatten()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key(vk: u16, ch: Option<char>) -> Event {
        Event::Key {
            vk,
            ch,
            mods: Mods::default(),
        }
    }

    #[test]
    fn folds_events_into_steps() {
        let ctrl = Mods {
            ctrl: true,
            ..Mods::default()
        };
        let mut r = Recorder::default();
        r.push(
            1000,
            Event::Click {
                x: 642,
                y: 518,
                button: Button::Left,
            },
        );
        r.push(1850, key(0x41, Some('п')));
        r.push(1900, key(0x42, Some('р')));
        r.push(1950, key(0x43, Some('х')));
        r.push(2000, key(BACKSPACE, None));
        r.push(2050, key(0x44, Some('и')));
        r.push(
            2100,
            Event::Key {
                vk: 0xA2,
                ch: None,
                mods: ctrl,
            },
        );
        r.push(
            2150,
            Event::Key {
                vk: 0x53,
                ch: None,
                mods: ctrl,
            },
        );
        r.push(3000, key(0x0D, Some('\r')));
        r.push(
            4000,
            Event::Click {
                x: 10,
                y: 10,
                button: Button::Left,
            },
        );
        r.push(
            4200,
            Event::Click {
                x: 11,
                y: 12,
                button: Button::Left,
            },
        );
        r.push(5000, Event::Wheel { delta: -120 });
        r.push(5100, Event::Wheel { delta: -60 });
        r.push(5200, Event::Wheel { delta: -60 });
        r.push(
            5300,
            Event::Click {
                x: 5,
                y: 5,
                button: Button::Right,
            },
        );
        r.push(9000, Event::Wheel { delta: 30 });
        let v = |n: f64| Some(Num::Value(n));
        let pause = |ms: f64| Action::PauseMs { ms: Num::Value(ms) };
        assert_eq!(
            r.finish(),
            vec![
                Action::MouseClickLeft {
                    x: v(642.0),
                    y: v(518.0)
                },
                pause(850.0),
                Action::KeysType {
                    text: "при".into()
                },
                Action::KeysPress {
                    keys: "Ctrl+S".into()
                },
                pause(850.0),
                Action::KeysPress {
                    keys: "Enter".into()
                },
                pause(1000.0),
                Action::MouseDouble {
                    x: v(11.0),
                    y: v(12.0)
                },
                pause(800.0),
                Action::MouseScroll {
                    dy: Num::Value(-2.0)
                },
                Action::MouseClickRight {
                    x: v(5.0),
                    y: v(5.0)
                },
            ]
        );
    }
}
