//! Desktop avatar (SPEC §3.6) and HUD skin (§3.7): small transparent click-through
//! windows above everything. One thread owns them: it follows the config and the
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
/// Logical px. The orb fills ~70 %, the rest is glow.
const AVATAR_SIZE: f64 = 176.0;
const HUD_SIZE: f64 = 460.0;
/// Gap between the avatar and the screen edge (logical px).
const MARGIN: f64 = 24.0;
const POLL: Duration = Duration::from_millis(1500);
/// HUD fade-out time before the window hides.
const HUD_LINGER: Duration = Duration::from_millis(700);
const HUD_PREVIEW: Duration = Duration::from_millis(3500);

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
}

fn run<R: Runtime>(app: AppHandle<R>, rx: Receiver<Msg>) {
    let mut c = Ctl {
        app,
        fullscreen: fullscreen(),
        editing: false,
        hud_hide_at: None,
    };
    c.sync();
    loop {
        let wait = c.hud_hide_at.map_or(POLL, |t| {
            t.saturating_duration_since(Instant::now()).min(POLL)
        });
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
    }

    fn state(&mut self, s: AssistantState) {
        let active = matches!(
            s,
            AssistantState::Listening | AssistantState::Processing | AssistantState::Speaking
        );
        if active {
            if self.ui().hud && !self.fullscreen {
                self.hud_show(None);
            }
        } else {
            self.hud_fade();
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
