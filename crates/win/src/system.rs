//! System.* actions (SPEC §4.4): power, power plan, brightness, screenshot, settings,
//! recycle bin, Wi-Fi/Bluetooth radios, DND, game mode, clipboard, spoken info.

use jarvis_core::commands::{Action, PowerPlan};

use crate::backend::{n, run_hidden, shell_open};

type Out = Result<Option<String>, String>;

/// `None` = not a system action (the caller tries other groups).
pub fn perform(action: &Action) -> Option<Out> {
    let secs = |v| n(v).map(|s| (s.max(0.0) as u64).to_string());
    let done = |r: Result<(), String>| r.map(|()| None);
    Some(match action {
        Action::Shutdown { delay_sec } => {
            secs(delay_sec).and_then(|t| run_hidden("shutdown", &["/s", "/t", &t]).map(|_| None))
        }
        Action::Restart { delay_sec } => {
            secs(delay_sec).and_then(|t| run_hidden("shutdown", &["/r", "/t", &t]).map(|_| None))
        }
        Action::Logoff => run_hidden("shutdown", &["/l"]).map(|_| None),
        Action::Hibernate => run_hidden("shutdown", &["/h"]).map(|_| None),
        Action::CancelShutdown => run_hidden("shutdown", &["/a"])
            .map(|_| None)
            .map_err(|_| "запланированного выключения нет".into()),
        Action::Sleep => done(win::sleep()),
        Action::Lock => done(win::lock()),
        Action::SetPowerPlan { plan } => done(power_plan(*plan)),
        Action::Brightness { level } => n(level).and_then(|l| done(brightness(l))),
        Action::BrightnessStep { step } => n(step).and_then(|d| done(brightness_step(d))),
        Action::MonitorOff => done(win::monitor_off()),
        Action::DarkTheme { on } => done(win::dark_theme(*on)),
        Action::Screenshot { region: true } => done(shell_open("ms-screenclip:", "", None, false)),
        Action::Screenshot { region: false } => shot(),
        Action::OpenSettings { uri } => {
            let uri = if uri.starts_with("ms-settings:") {
                uri.clone()
            } else {
                format!("ms-settings:{uri}")
            };
            done(shell_open(&uri, "", None, false))
        }
        Action::EmptyRecycleBin => done(win::empty_recycle_bin()),
        Action::WiFi { on } => done(win::radio(true, *on)),
        Action::Bluetooth { on } => done(win::radio(false, *on)),
        Action::Dnd { on } => done(dnd(*on)),
        Action::GameMode { on } => done(game_mode(*on)),
        Action::Clipboard { text } => match text {
            Some(t) => done(win::set_clipboard(t)),
            None => win::get_clipboard().map(Some),
        },
        Action::Info { what } if what.starts_with("rate:") => rate(&what[5..]).map(Some),
        Action::Info { what } if what == "ip" => local_ip().map(Some),
        Action::Info { what } if what == "internet" => Ok(Some(internet())),
        Action::Info { what } => win::info(what).map(Some),
        _ => return None,
    })
}

/// Local clock hour (startup greeting by time of day); `None` off Windows.
pub fn local_hour() -> Option<u32> {
    win::local_hour()
}

#[cfg(windows)]
fn shot() -> Out {
    crate::screenshot::take().map(|_| None)
}

#[cfg(not(windows))]
fn shot() -> Out {
    Err("скриншот только в Windows".into())
}

/// Hryvnia rate from the NBU via the built-in curl.exe (Windows 10 1803+), §8.
fn rate(code: &str) -> Result<String, String> {
    let json = run_hidden(
        "curl",
        &[
            "-sf",
            "-m",
            "4",
            "https://bank.gov.ua/NBUStatService/v1/statdirectory/exchange?json",
        ],
    )
    .map_err(|_| jarvis_core::NO_INTERNET.to_owned())?;
    jarvis_core::info::rate_phrase(&json, code).ok_or_else(|| format!("нет курса {code}"))
}

fn power_plan(plan: PowerPlan) -> Result<(), String> {
    let alias = match plan {
        PowerPlan::Balanced => "SCHEME_BALANCED",
        PowerPlan::High => "SCHEME_MIN", // «minimum power saving» = high performance
        PowerPlan::Saver => "SCHEME_MAX",
    };
    run_hidden("powercfg", &["/setactive", alias]).map(|_| ())
}

/// Laptop panel brightness through WMI (external monitors ignore it).
fn brightness(level: f64) -> Result<(), String> {
    let l = level.clamp(0.0, 100.0) as u32;
    let script = format!(
        "(Get-CimInstance -Namespace root/WMI -ClassName WmiMonitorBrightnessMethods | \
         Invoke-CimMethod -MethodName WmiSetBrightness -Arguments @{{Timeout=1;Brightness={l}}}) | Out-Null"
    );
    run_hidden(
        "powershell",
        &["-NoProfile", "-NonInteractive", "-Command", &script],
    )
    .map(|_| ())
    .map_err(|_| "яркость меняется только на встроенном экране".into())
}

/// Relative laptop brightness (first panel only; external monitors ignore WMI).
fn brightness_step(delta: f64) -> Result<(), String> {
    let d = delta.clamp(-100.0, 100.0) as i32;
    let script = format!(
        "$c=(Get-CimInstance -Namespace root/WMI -ClassName WmiMonitorBrightness | Select-Object -First 1).CurrentBrightness; \
         $b=[math]::Max(0,[math]::Min(100,$c+({d}))); \
         (Get-CimInstance -Namespace root/WMI -ClassName WmiMonitorBrightnessMethods | Select-Object -First 1 | \
         Invoke-CimMethod -MethodName WmiSetBrightness -Arguments @{{Timeout=1;Brightness=[byte]$b}}) | Out-Null"
    );
    run_hidden(
        "powershell",
        &["-NoProfile", "-NonInteractive", "-Command", &script],
    )
    .map(|_| ())
    .map_err(|_| "яркость меняется только на встроенном экране".into())
}

/// Local LAN address: a UDP «connect» only picks the route, no packet leaves the PC.
fn local_ip() -> Result<String, String> {
    let s = std::net::UdpSocket::bind("0.0.0.0:0").map_err(|e| e.to_string())?;
    s.connect("8.8.8.8:80")
        .map_err(|_| jarvis_core::NO_INTERNET.to_owned())?;
    let ip = s.local_addr().map_err(|e| e.to_string())?.ip();
    Ok(format!("Ваш локальный IP-адрес {ip}"))
}

/// Can we reach the internet (TCP to Cloudflare DNS, 3 s)?
fn internet() -> String {
    let addr = std::net::SocketAddr::from(([1, 1, 1, 1], 443));
    match std::net::TcpStream::connect_timeout(&addr, std::time::Duration::from_secs(3)) {
        Ok(_) => "Интернет работает, сэр".into(),
        Err(_) => "Сэр, интернета нет".into(),
    }
}

/// Toast notifications off/on (closest public switch to «Не беспокоить»).
fn dnd(on: bool) -> Result<(), String> {
    let v = if on { "0" } else { "1" };
    run_hidden(
        "reg",
        &[
            "add",
            r"HKCU\Software\Microsoft\Windows\CurrentVersion\Notifications\Settings",
            "/v",
            "NOC_GLOBAL_SETTING_TOASTS_ENABLED",
            "/t",
            "REG_DWORD",
            "/d",
            v,
            "/f",
        ],
    )
    .map(|_| ())
}

/// §4.7 «игровой режим»: high performance + DND (background app list comes with PC modes, T029).
fn game_mode(on: bool) -> Result<(), String> {
    power_plan(if on {
        PowerPlan::High
    } else {
        PowerPlan::Balanced
    })?;
    dnd(on)
}

#[cfg(windows)]
mod win {
    use jarvis_core::info;
    use windows::Devices::Radios::{Radio, RadioKind, RadioState};
    use windows::Win32::Foundation::{FILETIME, HANDLE, HGLOBAL};
    use windows::Win32::System::DataExchange::{
        CloseClipboard, EmptyClipboard, GetClipboardData, OpenClipboard, SetClipboardData,
    };
    use windows::Win32::System::Memory::{GlobalAlloc, GlobalLock, GlobalUnlock, GMEM_MOVEABLE};
    use windows::Win32::System::Ole::CF_UNICODETEXT;
    use windows::Win32::System::Power::{
        GetSystemPowerStatus, SetSuspendState, SYSTEM_POWER_STATUS,
    };
    use windows::Win32::System::Shutdown::LockWorkStation;
    use windows::Win32::System::SystemInformation::{
        GetLocalTime, GetTickCount64, GlobalMemoryStatusEx, MEMORYSTATUSEX,
    };
    use windows::Win32::System::Threading::GetSystemTimes;
    use windows::Win32::UI::Shell::{
        SHEmptyRecycleBinW, SHERB_NOCONFIRMATION, SHERB_NOPROGRESSUI, SHERB_NOSOUND,
    };

    fn e(err: windows::core::Error) -> String {
        err.message()
    }

    pub fn sleep() -> Result<(), String> {
        // SAFETY: plain Win32 call.
        if unsafe { SetSuspendState(false, false, false) } {
            Ok(())
        } else {
            Err("не удалось перейти в сон".into())
        }
    }

    pub fn lock() -> Result<(), String> {
        // SAFETY: plain Win32 call.
        unsafe { LockWorkStation() }.map_err(e)
    }

    pub fn monitor_off() -> Result<(), String> {
        use windows::Win32::Foundation::{LPARAM, WPARAM};
        use windows::Win32::UI::WindowsAndMessaging::{
            PostMessageW, HWND_BROADCAST, SC_MONITORPOWER, WM_SYSCOMMAND,
        };
        // let the spoken reply start before the screen goes dark
        std::thread::sleep(std::time::Duration::from_millis(600));
        // SAFETY: broadcast post, 2 = power off; no pointers involved.
        unsafe {
            PostMessageW(
                Some(HWND_BROADCAST),
                WM_SYSCOMMAND,
                WPARAM(SC_MONITORPOWER as usize),
                LPARAM(2),
            )
        }
        .map_err(e)
    }

    /// Apps + taskbar theme (HKCU Personalize), then tell running apps to repaint.
    pub fn dark_theme(on: bool) -> Result<(), String> {
        use windows::core::w;
        use windows::Win32::Foundation::{LPARAM, WPARAM};
        use windows::Win32::UI::WindowsAndMessaging::{
            SendMessageTimeoutW, HWND_BROADCAST, SMTO_ABORTIFHUNG, WM_SETTINGCHANGE,
        };
        use winreg::enums::{HKEY_CURRENT_USER, KEY_SET_VALUE};
        let key = winreg::RegKey::predef(HKEY_CURRENT_USER)
            .open_subkey_with_flags(
                r"Software\Microsoft\Windows\CurrentVersion\Themes\Personalize",
                KEY_SET_VALUE,
            )
            .map_err(|err| err.to_string())?;
        let light = u32::from(!on);
        for v in ["AppsUseLightTheme", "SystemUsesLightTheme"] {
            key.set_value(v, &light).map_err(|err| err.to_string())?;
        }
        let area = w!("ImmersiveColorSet");
        // SAFETY: `area` is a static NUL-terminated string; timeout bounds hung windows.
        unsafe {
            SendMessageTimeoutW(
                HWND_BROADCAST,
                WM_SETTINGCHANGE,
                WPARAM(0),
                LPARAM(area.as_ptr() as isize),
                SMTO_ABORTIFHUNG,
                1000,
                None,
            )
        };
        Ok(())
    }

    /// Fixed drives with size, via GetLogicalDrives + GetDiskFreeSpaceExW.
    fn drives() -> Vec<(char, u64, u64)> {
        use windows::core::HSTRING;
        use windows::Win32::Storage::FileSystem::{
            GetDiskFreeSpaceExW, GetDriveTypeW, GetLogicalDrives,
        };
        const DRIVE_FIXED: u32 = 3;
        const GB: u64 = 1 << 30;
        // SAFETY: no arguments.
        let mask = unsafe { GetLogicalDrives() };
        (0..26u8)
            .filter(|i| mask & (1 << i) != 0)
            .filter_map(|i| {
                let letter = char::from(b'A' + i);
                let root = HSTRING::from(format!("{letter}:\\"));
                // SAFETY: NUL-terminated root path alive for the calls; out-pointers to locals.
                unsafe {
                    if GetDriveTypeW(&root) != DRIVE_FIXED {
                        return None;
                    }
                    let (mut free, mut total) = (0u64, 0u64);
                    GetDiskFreeSpaceExW(&root, Some(&mut free), Some(&mut total), None).ok()?;
                    Some((letter, free / GB, total / GB))
                }
            })
            .collect()
    }

    pub fn empty_recycle_bin() -> Result<(), String> {
        let flags = SHERB_NOCONFIRMATION | SHERB_NOPROGRESSUI | SHERB_NOSOUND;
        // SAFETY: null root = all drives.
        unsafe { SHEmptyRecycleBinW(None, None, flags) }.or_else(|err| {
            // E_UNEXPECTED when the bin is already empty
            if err.code().0 as u32 == 0x8000_FFFF {
                Ok(())
            } else {
                Err(e(err))
            }
        })
    }

    /// Wi-Fi (`wifi = true`) or Bluetooth radio on/off via WinRT (no admin needed).
    pub fn radio(wifi: bool, on: bool) -> Result<(), String> {
        crate::keep_mta();
        let kind = if wifi {
            RadioKind::WiFi
        } else {
            RadioKind::Bluetooth
        };
        let state = if on { RadioState::On } else { RadioState::Off };
        let _ = Radio::RequestAccessAsync().and_then(|a| a.join());
        let radios = Radio::GetRadiosAsync().and_then(|a| a.join()).map_err(e)?;
        let mut found = false;
        for i in 0..radios.Size().map_err(e)? {
            let r = radios.GetAt(i).map_err(e)?;
            if r.Kind().map_err(e)? == kind {
                found = true;
                r.SetStateAsync(state).and_then(|a| a.join()).map_err(e)?;
            }
        }
        if found {
            Ok(())
        } else {
            Err(if wifi {
                "Wi-Fi адаптер не найден"
            } else {
                "Bluetooth не найден"
            }
            .into())
        }
    }

    pub fn set_clipboard(text: &str) -> Result<(), String> {
        let wide: Vec<u16> = text.encode_utf16().chain([0]).collect();
        let bytes = wide.len() * 2;
        // SAFETY: standard clipboard protocol; memory handed to the system on success.
        unsafe {
            OpenClipboard(None).map_err(e)?;
            let r = (|| {
                EmptyClipboard().map_err(e)?;
                let mem: HGLOBAL = GlobalAlloc(GMEM_MOVEABLE, bytes).map_err(e)?;
                let dst = GlobalLock(mem) as *mut u16;
                if dst.is_null() {
                    return Err("GlobalLock failed".to_string());
                }
                std::ptr::copy_nonoverlapping(wide.as_ptr(), dst, wide.len());
                let _ = GlobalUnlock(mem);
                SetClipboardData(u32::from(CF_UNICODETEXT.0), Some(HANDLE(mem.0))).map_err(e)?;
                Ok(())
            })();
            let _ = CloseClipboard();
            r
        }
    }

    pub fn get_clipboard() -> Result<String, String> {
        // SAFETY: standard clipboard protocol; data is read while locked.
        unsafe {
            OpenClipboard(None).map_err(e)?;
            let r = (|| {
                let h = GetClipboardData(u32::from(CF_UNICODETEXT.0))
                    .map_err(|_| "в буфере обмена нет текста".to_string())?;
                let mem = HGLOBAL(h.0);
                let p = GlobalLock(mem) as *const u16;
                if p.is_null() {
                    return Err("в буфере обмена нет текста".to_string());
                }
                let mut len = 0;
                while *p.add(len) != 0 {
                    len += 1;
                }
                let s = String::from_utf16_lossy(std::slice::from_raw_parts(p, len));
                let _ = GlobalUnlock(mem);
                Ok(s)
            })();
            let _ = CloseClipboard();
            r
        }
    }

    fn ft(t: FILETIME) -> u64 {
        (u64::from(t.dwHighDateTime) << 32) | u64::from(t.dwLowDateTime)
    }

    fn cpu_percent() -> u8 {
        let sample = || {
            let (mut i, mut k, mut u) = (
                FILETIME::default(),
                FILETIME::default(),
                FILETIME::default(),
            );
            // SAFETY: out-pointers to locals.
            let _ = unsafe { GetSystemTimes(Some(&mut i), Some(&mut k), Some(&mut u)) };
            (ft(i), ft(k) + ft(u))
        };
        let (i0, t0) = sample();
        std::thread::sleep(std::time::Duration::from_millis(300));
        let (i1, t1) = sample();
        let total = t1.saturating_sub(t0).max(1);
        (100 - (i1.saturating_sub(i0) * 100 / total).min(100)) as u8
    }

    pub fn local_hour() -> Option<u32> {
        // SAFETY: GetLocalTime has no failure mode.
        Some(u32::from(unsafe { GetLocalTime() }.wHour))
    }

    pub fn info(what: &str) -> Result<String, String> {
        // SAFETY: GetLocalTime has no failure mode.
        let t = unsafe { GetLocalTime() };
        Ok(match what {
            "time" => info::time_phrase(u32::from(t.wHour), u32::from(t.wMinute)),
            "greeting" => info::greeting_phrase(u32::from(t.wHour)).to_owned(),
            "date" => info::date_phrase(
                u32::from(t.wDay),
                u32::from(t.wMonth),
                u32::from(t.wDayOfWeek),
            ),
            "battery" => {
                let mut s = SYSTEM_POWER_STATUS::default();
                // SAFETY: out-pointer to a local.
                unsafe { GetSystemPowerStatus(&mut s) }.map_err(e)?;
                let no_battery = s.BatteryFlag == 128 || s.BatteryLifePercent == 255;
                info::battery_phrase(
                    (!no_battery).then_some(s.BatteryLifePercent),
                    s.ACLineStatus == 1,
                )
            }
            // SAFETY: no arguments.
            "uptime" => info::uptime_phrase(unsafe { GetTickCount64() } / 1000),
            "disk" => info::disk_phrase(&drives()),
            "load" | "cpu" | "ram" => {
                let mut m = MEMORYSTATUSEX {
                    dwLength: std::mem::size_of::<MEMORYSTATUSEX>() as u32,
                    ..Default::default()
                };
                // SAFETY: struct sized correctly.
                unsafe { GlobalMemoryStatusEx(&mut m) }.map_err(e)?;
                info::load_phrase(cpu_percent(), m.dwMemoryLoad as u8)
            }
            other => return Err(format!("неизвестный запрос информации «{other}»")),
        })
    }
}

#[cfg(not(windows))]
mod win {
    const NO: &str = "Windows only";
    pub fn sleep() -> Result<(), String> {
        Err(NO.into())
    }
    pub fn lock() -> Result<(), String> {
        Err(NO.into())
    }
    pub fn monitor_off() -> Result<(), String> {
        Err(NO.into())
    }
    pub fn dark_theme(_: bool) -> Result<(), String> {
        Err(NO.into())
    }
    pub fn empty_recycle_bin() -> Result<(), String> {
        Err(NO.into())
    }
    pub fn radio(_: bool, _: bool) -> Result<(), String> {
        Err(NO.into())
    }
    pub fn set_clipboard(_: &str) -> Result<(), String> {
        Err(NO.into())
    }
    pub fn get_clipboard() -> Result<String, String> {
        Err(NO.into())
    }
    pub fn info(_: &str) -> Result<String, String> {
        Err(NO.into())
    }
    pub fn local_hour() -> Option<u32> {
        None
    }
}

#[cfg(all(test, windows))]
mod tests {
    use super::*;

    #[test]
    fn info_phrases_are_russian() {
        let t = perform(&Action::Info {
            what: "time".into(),
        })
        .expect("system action");
        assert!(t.expect("ok").expect("text").starts_with("Сейчас"));
        let d = perform(&Action::Info {
            what: "date".into(),
        })
        .expect("system action");
        assert!(d.expect("ok").expect("text").starts_with("Сегодня"));
        assert!(perform(&Action::WindowClose).is_none());
    }
}
