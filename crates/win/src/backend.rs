//! Real Windows side effects for the executor (SPEC §4.4). Grows task by task (T023–T027).

use jarvis_core::commands::{Action, Num, Side};
use jarvis_core::executor::Backend;

use std::sync::{Arc, Mutex};

use jarvis_core::Config;

use crate::{apps, classroom, online, system};

#[derive(Default)]
pub struct WinBackend {
    /// Live config for online keys; `None` = defaults (tests).
    pub config: Option<Arc<Mutex<Config>>>,
}

impl Backend for WinBackend {
    fn foreground_exe(&self) -> Option<String> {
        #[cfg(windows)]
        {
            crate::window::foreground_exe()
        }
        #[cfg(not(windows))]
        {
            None
        }
    }

    fn perform(&self, action: &Action) -> Result<Option<String>, String> {
        if let Action::Info { what } = action {
            if what == "news" || what.starts_with("homework") || what.starts_with("ask:") {
                let keys = self
                    .config
                    .as_ref()
                    .map(|c| c.lock().unwrap_or_else(|e| e.into_inner()).online.clone())
                    .unwrap_or_default();
                return if what == "news" {
                    online::digest(&keys)
                } else if let Some(q) = what.strip_prefix("ask:") {
                    online::ask(&keys, q.trim())
                } else {
                    classroom::homework(&keys, what)
                }
                .map(Some);
            }
        }
        if let Some(r) = system::perform(action) {
            return r;
        }
        match action {
            Action::LaunchFile {
                path,
                args,
                workdir,
                admin,
            } => shell_open(path, args, workdir.as_deref(), *admin),
            Action::LaunchUrl { url } => shell_open(url, "", None, false),
            Action::LaunchUwp { aumid } => shell_open(
                "explorer.exe",
                &format!(r"shell:AppsFolder\{aumid}"),
                None,
                false,
            ),
            Action::LaunchFind { name } => {
                let target =
                    apps::resolve(name).ok_or_else(|| format!("программа «{name}» не найдена"))?;
                shell_open(&target, "", None, false)
            }
            Action::ProcessKill { name } => kill(name),
            Action::PlayWav { path } => play_wav(path),
            Action::RunCommand {
                cmd, powershell, ..
            } => run_visible(cmd, *powershell),
            Action::RunCommandHidden {
                cmd, powershell, ..
            } => {
                let r = if *powershell {
                    run_hidden(
                        "powershell",
                        &["-NoProfile", "-NonInteractive", "-Command", cmd],
                    )
                } else {
                    run_hidden("cmd", &["/C", cmd])
                };
                return r.map(|out| {
                    tracing::info!(%cmd, %out, "hidden command");
                    None
                });
            }
            other => input_or_window(other),
        }
        .map(|()| None)
    }
}

/// Value of a numeric param; slots are already filled by the executor.
pub(crate) fn n(v: &Num) -> Result<f64, String> {
    match v {
        Num::Value(x) => Ok(*x),
        // «-{число}» fills to the string "-20"
        Num::Slot(s) => s
            .trim()
            .parse()
            .map_err(|_| format!("не заполнен слот {s}")),
    }
}

#[cfg(windows)]
fn input_or_window(action: &Action) -> Result<(), String> {
    use windows::Win32::UI::WindowsAndMessaging::{SW_MAXIMIZE, SW_MINIMIZE, SW_RESTORE};

    use crate::{audio, keys, window};
    let at = |x: &Option<Num>, y: &Option<Num>| -> Result<(), String> {
        match (x, y) {
            (Some(x), Some(y)) => keys::move_to(n(x)? as i32, n(y)? as i32),
            _ => Ok(()),
        }
    };
    match action {
        Action::WindowClose => window::close(),
        Action::WindowMinimize => window::show(SW_MINIMIZE),
        Action::WindowMaximize => window::show(SW_MAXIMIZE),
        Action::WindowRestore => window::show(SW_RESTORE),
        // Win+M, not Win+D: no toggle (a second «сверни всё» doesn't bring them back),
        // «верни все окна» = Win+Shift+M
        Action::WindowMinimizeAll => keys::press(&keys::parse_combo("win+m")?),
        Action::WindowCloseAll => window::close_all().map(|_| ()),
        Action::WindowFocus { target } => window::focus(target),
        Action::WindowSnap { side } => keys::press(&keys::parse_combo(match side {
            Side::Left => "win+left",
            Side::Right => "win+right",
            Side::Top => "win+up",
        })?),
        Action::WindowMoveToMonitor { n: m } => window::move_to_monitor(n(m)? as usize),
        Action::WindowFullscreen => keys::press(&keys::parse_combo("f11")?),
        Action::WindowCloseApp { name } => window::close_app(name).map(|_| ()),
        Action::WindowTopmost => window::toggle_topmost().map(|_| ()),
        Action::KeysPress { keys: combo } => keys::press(&keys::parse_combo(combo)?),
        Action::KeysType { text } => keys::type_text(text),
        Action::KeysHold { key, ms } => {
            let vks = keys::parse_combo(key)?;
            keys::down(&vks)?;
            std::thread::sleep(std::time::Duration::from_millis(
                n(ms)?.clamp(0.0, 30_000.0) as u64,
            ));
            keys::up(&vks)
        }
        Action::MouseToCoords { x, y } => keys::move_to(n(x)? as i32, n(y)? as i32),
        Action::MouseClickLeft { x, y } => at(x, y).and_then(|_| keys::click(false, 1)),
        Action::MouseClickRight { x, y } => at(x, y).and_then(|_| keys::click(true, 1)),
        Action::MouseDouble { x, y } => at(x, y).and_then(|_| keys::click(false, 2)),
        Action::MouseLc { n: times } => keys::click(false, n(times)? as u32),
        Action::MouseScroll { dy } => keys::scroll(n(dy)? as i32),
        Action::VolumeSet { level } => audio::set_volume(n(level)?),
        Action::VolumeUp { step } => audio::change_volume(n(step)?),
        Action::VolumeDown { step } => audio::change_volume(-n(step)?),
        Action::Mute => audio::set_mute(true),
        Action::Unmute => audio::set_mute(false),
        Action::MediaPlayPause => keys::press(&[0xB3]),
        Action::MediaNext => keys::press(&[0xB0]),
        Action::MediaPrev => keys::press(&[0xB1]),
        Action::MediaStop => keys::press(&[0xB2]),
        Action::AudioSwitchDevice { name } => audio::switch_device(name),
        other => Err(format!("{other:?}: not supported yet")),
    }
}

#[cfg(not(windows))]
fn input_or_window(action: &Action) -> Result<(), String> {
    let _ = n;
    let _ = Side::Left;
    Err(format!("{action:?}: Windows only"))
}

#[cfg(windows)]
pub(crate) fn shell_open(
    file: &str,
    args: &str,
    workdir: Option<&str>,
    admin: bool,
) -> Result<(), String> {
    use windows::core::HSTRING;
    use windows::Win32::UI::Shell::ShellExecuteW;
    use windows::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL;

    let op = HSTRING::from(if admin { "runas" } else { "open" });
    let file_h = HSTRING::from(file);
    let args_h = HSTRING::from(args);
    let dir_h = HSTRING::from(workdir.unwrap_or(""));
    // SAFETY: all pointers come from HSTRINGs alive for the duration of the call.
    let rc = unsafe { ShellExecuteW(None, &op, &file_h, &args_h, &dir_h, SW_SHOWNORMAL) };
    // ShellExecute returns a value > 32 on success
    if rc.0 as usize > 32 {
        Ok(())
    } else {
        Err(format!(
            "не удалось запустить «{file}» (код {})",
            rc.0 as usize
        ))
    }
}

#[cfg(not(windows))]
pub(crate) fn shell_open(
    file: &str,
    _args: &str,
    _workdir: Option<&str>,
    _admin: bool,
) -> Result<(), String> {
    Err(format!("launch «{file}»: Windows only"))
}

/// Open a console window running the command (user sees the output).
fn run_visible(cmd: &str, powershell: bool) -> Result<(), String> {
    let args = if powershell {
        format!("/C start \"Jarvis\" powershell -NoExit -Command {cmd}")
    } else {
        format!("/C start \"Jarvis\" cmd /K {cmd}")
    };
    shell_open("cmd.exe", &args, None, false)
}

/// Blocking WAV playback (voice clips have their own player; this is for `Sound.PlayWav`).
#[cfg(windows)]
fn play_wav(path: &str) -> Result<(), String> {
    use windows::core::HSTRING;
    use windows::Win32::Media::Audio::{PlaySoundW, SND_FILENAME, SND_NODEFAULT, SND_SYNC};
    if !std::path::Path::new(path).exists() {
        return Err(format!("файл «{path}» не найден"));
    }
    let p = HSTRING::from(path);
    // SAFETY: NUL-terminated path alive for the call.
    if unsafe { PlaySoundW(&p, None, SND_FILENAME | SND_SYNC | SND_NODEFAULT) }.as_bool() {
        Ok(())
    } else {
        Err(format!("не удалось воспроизвести «{path}»"))
    }
}

#[cfg(not(windows))]
fn play_wav(path: &str) -> Result<(), String> {
    Err(format!("play «{path}»: Windows only"))
}

/// Run a console tool without flashing a window; stdout on success, stderr/stdout on failure.
pub(crate) fn run_hidden(program: &str, args: &[&str]) -> Result<String, String> {
    let mut cmd = std::process::Command::new(program);
    cmd.args(args);
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        cmd.creation_flags(CREATE_NO_WINDOW);
    }
    let out = cmd.output().map_err(|e| format!("{program}: {e}"))?;
    let text = |b: &[u8]| String::from_utf8_lossy(b).trim().to_owned();
    if out.status.success() {
        Ok(text(&out.stdout))
    } else {
        let e = text(&out.stderr);
        Err(if e.is_empty() { text(&out.stdout) } else { e })
    }
}

/// `taskkill /IM name.exe /F`.
fn kill(name: &str) -> Result<(), String> {
    let exe = if name.to_lowercase().ends_with(".exe") {
        name.to_owned()
    } else {
        format!("{name}.exe")
    };
    run_hidden("taskkill", &["/IM", &exe, "/F"])
        .map(|_| ())
        .map_err(|_| format!("процесс «{exe}» не найден"))
}

#[cfg(all(test, windows))]
mod tests {
    use super::*;

    #[test]
    fn kill_missing_process_reports_error() {
        let err = WinBackend::default()
            .perform(&Action::ProcessKill {
                name: "definitely-not-running-jarvis-test".into(),
            })
            .expect_err("should fail");
        assert!(err.contains("не найден"));
    }
}

#[cfg(test)]
mod num_tests {
    use super::*;

    #[test]
    fn filled_negative_slot_parses() {
        assert_eq!(n(&Num::Slot("-20".into())), Ok(-20.0));
        assert!(n(&Num::Slot("{число}".into())).is_err());
    }
}
