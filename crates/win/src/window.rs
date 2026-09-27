//! Window.* actions (SPEC §4.4) on top-level windows.

use windows::core::BOOL;
use windows::Win32::Foundation::{CloseHandle, HWND, LPARAM, RECT, WPARAM};
use windows::Win32::Graphics::Gdi::{
    EnumDisplayMonitors, GetMonitorInfoW, HDC, HMONITOR, MONITORINFO,
};
use windows::Win32::System::Threading::{
    OpenProcess, QueryFullProcessImageNameW, PROCESS_NAME_WIN32, PROCESS_QUERY_LIMITED_INFORMATION,
};
use windows::Win32::UI::WindowsAndMessaging::{
    EnumWindows, GetForegroundWindow, GetWindowRect, GetWindowTextW, GetWindowThreadProcessId,
    IsIconic, IsWindowVisible, PostMessageW, SetForegroundWindow, SetWindowPos, ShowWindow,
    SHOW_WINDOW_CMD, SWP_NOZORDER, SW_RESTORE, WM_CLOSE,
};

use crate::keys;

fn foreground() -> Result<HWND, String> {
    // SAFETY: no arguments; returns null when there's no foreground window.
    let h = unsafe { GetForegroundWindow() };
    if h.0.is_null() {
        Err("нет активного окна".into())
    } else {
        Ok(h)
    }
}

pub fn close() -> Result<(), String> {
    let h = foreground()?;
    // SAFETY: valid window handle, WM_CLOSE takes no params.
    unsafe { PostMessageW(Some(h), WM_CLOSE, WPARAM(0), LPARAM(0)) }.map_err(|e| e.to_string())
}

pub fn show(cmd: SHOW_WINDOW_CMD) -> Result<(), String> {
    let h = foreground()?;
    // SAFETY: valid window handle.
    let _ = unsafe { ShowWindow(h, cmd) };
    Ok(())
}

fn title(h: HWND) -> String {
    let mut buf = [0u16; 512];
    // SAFETY: buffer is valid for its length.
    let n = unsafe { GetWindowTextW(h, &mut buf) };
    String::from_utf16_lossy(&buf[..n.max(0) as usize])
}

/// Exe file name of the window's process, lowercase (`chrome.exe`).
pub fn exe_of(h: HWND) -> Option<String> {
    let mut pid = 0u32;
    // SAFETY: out-pointer to a local.
    unsafe { GetWindowThreadProcessId(h, Some(&mut pid)) };
    // SAFETY: querying a pid with limited rights; handle closed below.
    let proc = unsafe { OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid) }.ok()?;
    let mut buf = [0u16; 1024];
    let mut len = buf.len() as u32;
    // SAFETY: buffer and length describe the same memory.
    let ok = unsafe {
        QueryFullProcessImageNameW(
            proc,
            PROCESS_NAME_WIN32,
            windows::core::PWSTR(buf.as_mut_ptr()),
            &mut len,
        )
    };
    // SAFETY: handle from OpenProcess.
    let _ = unsafe { CloseHandle(proc) };
    ok.ok()?;
    let path = String::from_utf16_lossy(&buf[..len as usize]);
    path.rsplit('\\').next().map(str::to_lowercase)
}

/// Exe of the foreground window (context rules, SPEC §4.5).
pub fn foreground_exe() -> Option<String> {
    exe_of(foreground().ok()?)
}

fn visible_windows() -> Vec<HWND> {
    unsafe extern "system" fn cb(h: HWND, lp: LPARAM) -> BOOL {
        // SAFETY: lp is the &mut Vec passed below, alive during EnumWindows.
        let v = unsafe { &mut *(lp.0 as *mut Vec<HWND>) };
        // SAFETY: valid handle from the enumeration.
        if unsafe { IsWindowVisible(h) }.as_bool() {
            v.push(h);
        }
        BOOL(1)
    }
    let mut v: Vec<HWND> = Vec::new();
    // SAFETY: callback only touches `v` through the pointer we pass.
    let _ = unsafe { EnumWindows(Some(cb), LPARAM(&mut v as *mut _ as isize)) };
    v
}

/// Bring a window to front by title substring or process exe name.
pub fn focus(target: &str) -> Result<(), String> {
    let t = target.to_lowercase();
    let exe = if t.ends_with(".exe") {
        t.clone()
    } else {
        format!("{t}.exe")
    };
    let h = visible_windows()
        .into_iter()
        .find(|&h| {
            let ti = title(h).to_lowercase();
            !ti.is_empty() && (ti.contains(&t) || exe_of(h).as_deref() == Some(exe.as_str()))
        })
        .ok_or_else(|| format!("окно «{target}» не найдено"))?;
    // SAFETY: valid handles; Alt tap lifts the foreground lock.
    unsafe {
        if IsIconic(h).as_bool() {
            let _ = ShowWindow(h, SW_RESTORE);
        }
        let _ = keys::press(&[0x12]);
        let _ = SetForegroundWindow(h);
    }
    Ok(())
}

fn monitors() -> Vec<RECT> {
    unsafe extern "system" fn cb(m: HMONITOR, _: HDC, _: *mut RECT, lp: LPARAM) -> BOOL {
        // SAFETY: lp is the &mut Vec passed below.
        let v = unsafe { &mut *(lp.0 as *mut Vec<RECT>) };
        let mut mi = MONITORINFO {
            cbSize: std::mem::size_of::<MONITORINFO>() as u32,
            ..Default::default()
        };
        // SAFETY: mi is sized correctly.
        if unsafe { GetMonitorInfoW(m, &mut mi) }.as_bool() {
            v.push(mi.rcWork);
        }
        BOOL(1)
    }
    let mut v: Vec<RECT> = Vec::new();
    // SAFETY: callback only touches `v`.
    let _ = unsafe { EnumDisplayMonitors(None, None, Some(cb), LPARAM(&mut v as *mut _ as isize)) };
    v.sort_by_key(|r| (r.left, r.top));
    v
}

/// Move the active window to monitor `n` (1 = leftmost), keeping its size.
pub fn move_to_monitor(n: usize) -> Result<(), String> {
    let h = foreground()?;
    let mons = monitors();
    let m = mons
        .get(n.saturating_sub(1))
        .ok_or_else(|| format!("монитор {n} не найден (всего {})", mons.len()))?;
    let mut r = RECT::default();
    // SAFETY: valid handle and out-pointer.
    unsafe { GetWindowRect(h, &mut r) }.map_err(|e| e.to_string())?;
    let (w, hgt) = (r.right - r.left, r.bottom - r.top);
    // SAFETY: valid handle.
    unsafe {
        let _ = ShowWindow(h, SW_RESTORE);
        SetWindowPos(h, None, m.left + 40, m.top + 40, w, hgt, SWP_NOZORDER)
    }
    .map_err(|e| e.to_string())
}

/// A fullscreen app (game, F11 video, presentation) owns the screen: overlays hide (§3.6).
/// Uses the shell's own notification state, the same signal Windows uses for Focus Assist.
pub fn fullscreen_app() -> bool {
    use windows::Win32::UI::Shell::{
        SHQueryUserNotificationState, QUNS_BUSY, QUNS_PRESENTATION_MODE,
        QUNS_RUNNING_D3D_FULL_SCREEN,
    };
    // SAFETY: plain query, no arguments besides the out value handled by the wrapper.
    match unsafe { SHQueryUserNotificationState() } {
        Ok(s) => s == QUNS_BUSY || s == QUNS_RUNNING_D3D_FULL_SCREEN || s == QUNS_PRESENTATION_MODE,
        Err(_) => false,
    }
}
