//! Desktop avatar (SPEC §3.6), HUD skin (§3.7) and the listening pill (video 32): small
//! transparent click-through windows above everything, kept out of screenshots. One thread owns them: it follows the config and the
//! assistant state and polls for fullscreen apps, so nothing ever covers a game.
//! Windows are created only while enabled and destroyed when switched off (RAM, §1).

use std::sync::mpsc::{self, Receiver, RecvTimeoutError, Sender};
use std::time::{Duration, Instant};

use jarvis_core::ipc::AssistantState;
use tauri::{
    AppHandle, Emitter, LogicalSize, Manager, PhysicalPosition, Runtime, WebviewUrl, WebviewWindow,
    WebviewWindowBuilder,
};

use crate::AppState;

pub const AVATAR: &str = "avatar";
pub const HUD: &str = "hud";
pub const PILL: &str = "pill";
/// Logical px. The orb fills ~70 %, the rest is glow.
const AVATAR_SIZE: f64 = 176.0;
const HUD_SIZE: f64 = 460.0;
/// Gap between the avatar and the screen edge (logical px).
const MARGIN: f64 = 24.0;
const POLL: Duration = Duration::from_millis(1500);
/// HUD fade-out time before the window hides.
const HUD_LINGER: Duration = Duration::from_millis(700);
const HUD_PREVIEW: Duration = Duration::from_millis(3500);
/// Logical px: room for the pill (44 px) plus its shadow.
const PILL_W: f64 = 680.0;
const PILL_H: f64 = 80.0;
/// The ✓ result stays readable a moment after Jarvis goes idle.
const PILL_LINGER: Duration = Duration::from_millis(1600);
/// Hidden this long → the WebView is destroyed (RAM, §1); the next wake builds it again.
const PILL_FREE: Duration = Duration::from_secs(90);

pub enum Msg {
    /// Config changed (settings, control panel toggle).
    Sync,
    State(AssistantState),
    /// Avatar placement mode: clickable + draggable; `false` saves the position.
    Edit(bool),
    HudPreview,
}

#[derive(Clone)]
pub struct Overlay(Sender<Msg>);

impl Overlay {
    pub fn send(&self, msg: Msg) {
        let _ = self.0.send(msg);
    }
}

pub fn spawn<R: Runtime>(app: AppHandle<R>) -> Overlay {
    let (tx, rx) = mpsc::channel();
    let spawned = std::thread::Builder::new()
        .name("overlay".into())
        .spawn(move || run(app, rx));
    if let Err(e) = spawned {
        tracing::warn!("overlay thread: {e}");
    }
    Overlay(tx)
}

struct Ctl<R: Runtime> {
    app: AppHandle<R>,
    fullscreen: bool,
    editing: bool,
    /// HUD is fading out; hide it at this moment.
    hud_hide_at: Option<Instant>,
    pill_hide_at: Option<Instant>,
    pill_hidden_at: Option<Instant>,
}

fn run<R: Runtime>(app: AppHandle<R>, rx: Receiver<Msg>) {
    let mut c = Ctl {
        app,
        fullscreen: fullscreen(),
        editing: false,
        hud_hide_at: None,
        pill_hide_at: None,
        pill_hidden_at: None,
    };
    c.sync();
    loop {
        let wait = [c.hud_hide_at, c.pill_hide_at]
            .into_iter()
            .flatten()
            .map(|t| t.saturating_duration_since(Instant::now()))
            .fold(POLL, Duration::min);
        match rx.recv_timeout(wait) {
            Ok(Msg::Sync) => c.sync(),
            Ok(Msg::State(s)) => c.state(s),
            Ok(Msg::Edit(on)) => c.edit(on),
            Ok(Msg::HudPreview) => c.hud_show(Some(HUD_PREVIEW)),
            Err(RecvTimeoutError::Timeout) => {}
            Err(RecvTimeoutError::Disconnected) => return,
        }
        c.tick();
    }
}

fn fullscreen() -> bool {
    #[cfg(windows)]
    {
        jarvis_win::window::fullscreen_app()
    }
    #[cfg(not(windows))]
    {
        false
    }
}

impl<R: Runtime> Ctl<R> {
    fn ui(&self) -> jarvis_core::config::UiPrefs {
        self.app.state::<AppState>().config_snapshot().ui
    }

    fn tick(&mut self) {
        let now = Instant::now();
        if self.pill_hide_at.is_some_and(|t| now >= t) {
            self.pill_hide_at = None;
            self.pill_hidden_at = Some(now);
            if let Some(w) = self.app.get_webview_window(PILL) {
                let _ = w.hide();
            }
        }
        if self.pill_hidden_at.is_some_and(|t| now >= t + PILL_FREE) {
            self.pill_hidden_at = None;
            if let Some(w) = self.app.get_webview_window(PILL) {
                let _ = w.destroy();
            }
        }
        if self.hud_hide_at.is_some_and(|t| Instant::now() >= t) {
            self.hud_hide_at = None;
            if let Some(w) = self.app.get_webview_window(HUD) {
                let _ = w.hide();
            }
        }
        let fs = fullscreen();
        if fs != self.fullscreen {
            self.fullscreen = fs;
            tracing::debug!(fullscreen = fs, "overlay");
            if fs {
                self.hud_hide_now();
                self.pill_hide_at = Some(now);
            }
            self.sync();
        }
    }

    /// Bring the windows in line with the config and the fullscreen state.
    fn sync(&mut self) {
        let ui = self.ui();
        let show = self.editing || (ui.avatar && !self.fullscreen);
        match (show, self.app.get_webview_window(AVATAR)) {
            (true, Some(w)) => {
                let _ = w.show();
            }
            (true, None) => {
                if let Err(e) = self.create_avatar(ui.avatar_pos) {
                    tracing::warn!("avatar window: {e}");
                }
            }
            // hidden during fullscreen keeps it warm; switched off frees the WebView
            (false, Some(w)) if ui.avatar => {
                let _ = w.hide();
            }
            (false, Some(w)) => {
                let _ = w.destroy();
            }
            (false, None) => {}
        }
        if !ui.hud {
            if let Some(w) = self.app.get_webview_window(HUD) {
                let _ = w.destroy();
            }
            self.hud_hide_at = None;
        }
        if !ui.pill {
            if let Some(w) = self.app.get_webview_window(PILL) {
                let _ = w.destroy();
            }
            self.pill_hide_at = None;
            self.pill_hidden_at = None;
        }
    }

    fn state(&mut self, s: AssistantState) {
        let active = matches!(
            s,
            AssistantState::Listening | AssistantState::Processing | AssistantState::Speaking
        );
        let ui = self.ui();
        if active {
            if ui.hud && !self.fullscreen {
                self.hud_show(None);
            }
            if ui.pill && !self.fullscreen {
                self.pill_show();
            }
        } else {
            self.hud_fade();
            self.pill_fade();
        }
    }

    fn pill_show(&mut self) {
        self.pill_hide_at = None;
        self.pill_hidden_at = None;
        let w = match self.app.get_webview_window(PILL) {
            Some(w) => w,
            None => match self.create_pill() {
                Ok(w) => w,
                Err(e) => {
                    tracing::warn!("pill window: {e}");
                    return;
                }
            },
        };
        let _ = w.show();
        let _ = self.app.emit_to(PILL, "pill", true);
    }

    fn pill_fade(&mut self) {
        let visible = self
            .app
            .get_webview_window(PILL)
            .is_some_and(|w| w.is_visible().unwrap_or(false));
        if visible && self.pill_hide_at.is_none() {
            self.pill_hide_at = Some(Instant::now() + PILL_LINGER);
            // the out motion runs at the end of the linger
            let app = self.app.clone();
            std::thread::spawn(move || {
                std::thread::sleep(PILL_LINGER.saturating_sub(Duration::from_millis(200)));
                let _ = app.emit_to(PILL, "pill", false);
            });
        }
    }

    fn hud_show(&mut self, preview: Option<Duration>) {
        if self.fullscreen {
            return;
        }
        let w = match self.app.get_webview_window(HUD) {
            Some(w) => w,
            None => match self.create_hud() {
                Ok(w) => w,
                Err(e) => {
                    tracing::warn!("hud window: {e}");
                    return;
                }
            },
        };
        let _ = w.show();
        let _ = self.app.emit_to(HUD, "hud", true);
        self.hud_hide_at = preview.map(|d| Instant::now() + d);
        if preview.is_some() {
            // the preview fades out like a real answer does
            let app = self.app.clone();
            let fade_at = preview.unwrap_or_default().saturating_sub(HUD_LINGER);
            std::thread::spawn(move || {
                std::thread::sleep(fade_at);
                let _ = app.emit_to(HUD, "hud", false);
            });
        }
    }

    fn hud_fade(&mut self) {
        let visible = self
            .app
            .get_webview_window(HUD)
            .is_some_and(|w| w.is_visible().unwrap_or(false));
        if visible && self.hud_hide_at.is_none() {
            let _ = self.app.emit_to(HUD, "hud", false);
            self.hud_hide_at = Some(Instant::now() + HUD_LINGER);
        }
    }

    fn hud_hide_now(&mut self) {
        self.hud_hide_at = None;
        if let Some(w) = self.app.get_webview_window(HUD) {
            let _ = w.hide();
        }
    }

    fn edit(&mut self, on: bool) {
        if !on && self.editing {
            self.save_position();
        }
        self.editing = on;
        self.sync();
        if let Some(w) = self.app.get_webview_window(AVATAR) {
            let _ = w.set_ignore_cursor_events(!on);
            if on {
                let _ = w.set_focus();
            }
        }
        let _ = self.app.emit_to(AVATAR, "avatar-edit", on);
    }

    fn save_position(&self) {
        let Some(pos) = self
            .app
            .get_webview_window(AVATAR)
            .and_then(|w| w.outer_position().ok())
        else {
            return;
        };
        let state = self.app.state::<AppState>();
        let mut c = state.config.lock().unwrap_or_else(|e| e.into_inner());
        c.ui.avatar_pos = Some([pos.x, pos.y]);
        if let Err(e) = c.save(&state.paths.config()) {
            tracing::warn!("config save: {e}");
        }
    }

    fn builder(&self, label: &str, size: f64) -> WebviewWindowBuilder<'_, R, AppHandle<R>> {
        // hash, not query: WebviewUrl::App is a path
        WebviewWindowBuilder::new(
            &self.app,
            label,
            WebviewUrl::App(format!("index.html#{label}").into()),
        )
        .title(format!("Jarvis {label}"))
        .inner_size(size, size)
        .decorations(false)
        .transparent(true)
        .shadow(false)
        .always_on_top(true)
        .skip_taskbar(true)
        .resizable(false)
        .focused(false)
        .visible(false)
    }

    fn create_avatar(&self, pos: Option<[i32; 2]>) -> tauri::Result<()> {
        let w = self.builder(AVATAR, AVATAR_SIZE).build()?;
        no_capture(&w);
        let pos = match pos {
            Some([x, y]) if on_screen(&w, x, y) => PhysicalPosition::new(x, y),
            _ => corner(&w).unwrap_or_default(),
        };
        w.set_position(pos)?;
        w.set_ignore_cursor_events(!self.editing)?;
        w.show()?;
        Ok(())
    }

    fn create_hud(&self) -> tauri::Result<WebviewWindow<R>> {
        let w = self.builder(HUD, HUD_SIZE).build()?;
        w.set_ignore_cursor_events(true)?;
        no_capture(&w);
        // centre of the primary screen
        if let Ok(Some(m)) = w.primary_monitor() {
            let s = m.scale_factor();
            let side = (HUD_SIZE * s) as i32;
            let (mx, my) = (m.position().x, m.position().y);
            let (mw, mh) = (m.size().width as i32, m.size().height as i32);
            w.set_position(PhysicalPosition::new(
                mx + (mw - side) / 2,
                my + (mh - side) / 2,
            ))?;
            w.set_size(LogicalSize::new(HUD_SIZE, HUD_SIZE))?;
        }
        Ok(w)
    }

    /// Top centre of the primary monitor's work area.
    fn create_pill(&self) -> tauri::Result<WebviewWindow<R>> {
        let w = self
            .builder(PILL, PILL_H)
            .inner_size(PILL_W, PILL_H)
            .build()?;
        w.set_ignore_cursor_events(true)?;
        no_capture(&w);
        if let Ok(Some(m)) = w.primary_monitor() {
            let s = m.scale_factor();
            let wa = m.work_area();
            w.set_position(PhysicalPosition::new(
                wa.position.x + (wa.size.width as i32 - (PILL_W * s) as i32) / 2,
                wa.position.y,
            ))?;
        }
        Ok(w)
    }
}

/// Screenshots and recordings («скриншот», OBS) must not show Jarvis' own overlays.
fn no_capture<R: Runtime>(w: &WebviewWindow<R>) {
    #[cfg(windows)]
    if let Ok(h) = w.hwnd() {
        jarvis_win::window::hide_from_capture(h.0 as isize);
    }
    #[cfg(not(windows))]
    let _ = w;
}

/// Bottom-right of the primary monitor's work area (above the taskbar).
fn corner<R: Runtime>(w: &WebviewWindow<R>) -> Option<PhysicalPosition<i32>> {
    let m = w.primary_monitor().ok()??;
    let s = m.scale_factor();
    let wa = m.work_area();
    let side = (AVATAR_SIZE * s) as i32;
    let gap = (MARGIN * s) as i32;
    Some(PhysicalPosition::new(
        wa.position.x + wa.size.width as i32 - side - gap,
        wa.position.y + wa.size.height as i32 - side - gap,
    ))
}

/// A saved position may point at a monitor that is gone.
fn on_screen<R: Runtime>(w: &WebviewWindow<R>, x: i32, y: i32) -> bool {
    w.available_monitors().is_ok_and(|ms| {
        ms.iter().any(|m| {
            let (p, s) = (m.position(), m.size());
            x >= p.x - 40
                && y >= p.y - 40
                && x < p.x + s.width as i32 - 40
                && y < p.y + s.height as i32 - 40
        })
    })
}
