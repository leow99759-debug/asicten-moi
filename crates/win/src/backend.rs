//! Real Windows side effects for the executor (SPEC §4.4). Grows task by task (T023–T027).

use jarvis_core::commands::Action;
use jarvis_core::executor::Backend;

use crate::apps;

pub struct WinBackend;

impl Backend for WinBackend {
    fn perform(&self, action: &Action) -> Result<(), String> {
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
                let lnk = apps::find_app(name).ok_or_else(|| format!("«{name}» не найдено"))?;
                shell_open(&lnk.to_string_lossy(), "", None, false)
            }
            Action::ProcessKill { name } => kill(name),
            other => Err(format!("{other:?}: not supported yet")),
        }
    }
}

#[cfg(windows)]
fn shell_open(file: &str, args: &str, workdir: Option<&str>, admin: bool) -> Result<(), String> {
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
fn shell_open(file: &str, _args: &str, _workdir: Option<&str>, _admin: bool) -> Result<(), String> {
    Err(format!("launch «{file}»: Windows only"))
}

/// `taskkill /IM name.exe /F` without a console window.
fn kill(name: &str) -> Result<(), String> {
    let exe = if name.to_lowercase().ends_with(".exe") {
        name.to_owned()
    } else {
        format!("{name}.exe")
    };
    let mut cmd = std::process::Command::new("taskkill");
    cmd.args(["/IM", &exe, "/F"]);
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        cmd.creation_flags(CREATE_NO_WINDOW);
    }
    let out = cmd.output().map_err(|e| e.to_string())?;
    if out.status.success() {
        Ok(())
    } else {
        Err(format!("процесс «{exe}» не найден"))
    }
}

#[cfg(all(test, windows))]
mod tests {
    use super::*;

    #[test]
    fn kill_missing_process_reports_error() {
        let err = WinBackend
            .perform(&Action::ProcessKill {
                name: "definitely-not-running-jarvis-test".into(),
            })
            .expect_err("should fail");
        assert!(err.contains("не найден"));
    }
}
