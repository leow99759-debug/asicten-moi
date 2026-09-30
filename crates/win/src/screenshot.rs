//! «Скриншот»: the whole virtual screen (all monitors) → PNG in Pictures\Screenshots and an
//! image on the clipboard, ready to paste into Telegram/Discord. Plain GDI, no flash, no
//! Snipping Tool. Jarvis overlays opt out of capture (`window::hide_from_capture`).

use std::path::PathBuf;

use windows::Win32::Foundation::{HANDLE, HGLOBAL};
use windows::Win32::Graphics::Gdi::{
    BitBlt, CreateCompatibleBitmap, CreateCompatibleDC, DeleteDC, DeleteObject, GetDC, GetDIBits,
    ReleaseDC, SelectObject, BITMAPINFO, BITMAPINFOHEADER, BI_RGB, CAPTUREBLT, DIB_RGB_COLORS,
    SRCCOPY,
};
use windows::Win32::System::Com::CoTaskMemFree;
use windows::Win32::System::DataExchange::{
    CloseClipboard, EmptyClipboard, OpenClipboard, SetClipboardData,
};
use windows::Win32::System::Memory::{GlobalAlloc, GlobalLock, GlobalUnlock, GMEM_MOVEABLE};
use windows::Win32::System::Ole::CF_DIB;
use windows::Win32::System::SystemInformation::GetLocalTime;
use windows::Win32::UI::Shell::{FOLDERID_Screenshots, SHGetKnownFolderPath, KF_FLAG_CREATE};
use windows::Win32::UI::WindowsAndMessaging::{
    GetSystemMetrics, SM_CXVIRTUALSCREEN, SM_CYVIRTUALSCREEN, SM_XVIRTUALSCREEN, SM_YVIRTUALSCREEN,
};

fn e(err: windows::core::Error) -> String {
    err.to_string()
}

/// Bottom-up 32-bit BGRA rows (the CF_DIB layout), alpha forced to 255.
struct Shot {
    w: i32,
    h: i32,
    bgra: Vec<u8>,
}

fn header(w: i32, h: i32) -> BITMAPINFOHEADER {
    BITMAPINFOHEADER {
        biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
        biWidth: w,
        biHeight: h, // positive = bottom-up
        biPlanes: 1,
        biBitCount: 32,
        biCompression: BI_RGB.0,
        ..Default::default()
    }
}

fn grab() -> Result<Shot, String> {
    // SAFETY: GDI objects are created, used and released in this block only.
    unsafe {
        let (x, y) = (
            GetSystemMetrics(SM_XVIRTUALSCREEN),
            GetSystemMetrics(SM_YVIRTUALSCREEN),
        );
        let (w, h) = (
            GetSystemMetrics(SM_CXVIRTUALSCREEN),
            GetSystemMetrics(SM_CYVIRTUALSCREEN),
        );
        if w <= 0 || h <= 0 {
            return Err("экран не найден".into());
        }
        let screen = GetDC(None);
        let mem = CreateCompatibleDC(Some(screen));
        let bmp = CreateCompatibleBitmap(screen, w, h);
        let old = SelectObject(mem, bmp.into());
        let copied = BitBlt(mem, 0, 0, w, h, Some(screen), x, y, SRCCOPY | CAPTUREBLT);
        let mut info = BITMAPINFO {
            bmiHeader: header(w, h),
            ..Default::default()
        };
        let mut bgra = vec![0u8; w as usize * h as usize * 4];
        let rows = GetDIBits(
            mem,
            bmp,
            0,
            h as u32,
            Some(bgra.as_mut_ptr().cast()),
            &mut info,
            DIB_RGB_COLORS,
        );
        SelectObject(mem, old);
        let _ = DeleteObject(bmp.into());
        let _ = DeleteDC(mem);
        ReleaseDC(None, screen);
        copied.map_err(e)?;
        if rows == 0 {
            return Err("не удалось прочитать экран".into());
        }
        for px in bgra.chunks_exact_mut(4) {
            px[3] = 255;
        }
        Ok(Shot { w, h, bgra })
    }
}

fn png(shot: &Shot) -> Result<Vec<u8>, String> {
    let (w, h) = (shot.w as usize, shot.h as usize);
    let mut rgb = Vec::with_capacity(w * h * 3);
    for row in shot.bgra.chunks_exact(w * 4).rev() {
        for px in row.chunks_exact(4) {
            rgb.extend_from_slice(&[px[2], px[1], px[0]]);
        }
    }
    let mut out = Vec::new();
    let mut enc = png::Encoder::new(&mut out, w as u32, h as u32);
    enc.set_color(png::ColorType::Rgb);
    enc.set_compression(png::Compression::Fast);
    let mut wr = enc.write_header().map_err(|e| e.to_string())?;
    wr.write_image_data(&rgb).map_err(|e| e.to_string())?;
    wr.finish().map_err(|e| e.to_string())?;
    Ok(out)
}

fn to_clipboard(shot: &Shot) -> Result<(), String> {
    let head = header(shot.w, shot.h);
    let hs = std::mem::size_of::<BITMAPINFOHEADER>();
    // SAFETY: standard clipboard protocol; the memory belongs to the system on success.
    unsafe {
        OpenClipboard(None).map_err(e)?;
        let r = (|| {
            EmptyClipboard().map_err(e)?;
            let mem: HGLOBAL = GlobalAlloc(GMEM_MOVEABLE, hs + shot.bgra.len()).map_err(e)?;
            let dst = GlobalLock(mem) as *mut u8;
            if dst.is_null() {
                return Err("GlobalLock failed".to_string());
            }
            std::ptr::copy_nonoverlapping((&head as *const BITMAPINFOHEADER).cast(), dst, hs);
            std::ptr::copy_nonoverlapping(shot.bgra.as_ptr(), dst.add(hs), shot.bgra.len());
            let _ = GlobalUnlock(mem);
            SetClipboardData(u32::from(CF_DIB.0), Some(HANDLE(mem.0))).map_err(e)?;
            Ok(())
        })();
        let _ = CloseClipboard();
        r
    }
}

/// Pictures\Screenshots (follows OneDrive/redirected folders), created if missing.
pub fn folder() -> Result<PathBuf, String> {
    // SAFETY: known-folder query; the returned string is freed with CoTaskMemFree.
    unsafe {
        let p = SHGetKnownFolderPath(&FOLDERID_Screenshots, KF_FLAG_CREATE, None).map_err(e)?;
        let s = p.to_string().map_err(|e| e.to_string());
        CoTaskMemFree(Some(p.0 as *const _));
        Ok(PathBuf::from(s?))
    }
}

/// Take the screenshot: saved file path (the clipboard copy is best effort).
pub fn take() -> Result<PathBuf, String> {
    let shot = grab()?;
    // SAFETY: plain query.
    let t = unsafe { GetLocalTime() };
    let name = format!(
        "Jarvis {:04}-{:02}-{:02} {:02}-{:02}-{:02}.png",
        t.wYear, t.wMonth, t.wDay, t.wHour, t.wMinute, t.wSecond
    );
    let path = folder()?.join(name);
    std::fs::write(&path, png(&shot)?).map_err(|e| format!("не удалось сохранить: {e}"))?;
    if let Err(err) = to_clipboard(&shot) {
        tracing::warn!(%err, "screenshot clipboard");
    }
    tracing::info!(path = %path.display(), w = shot.w, h = shot.h, "screenshot");
    Ok(path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn png_flips_rows_and_drops_alpha() {
        // 1×2 bottom-up: first stored row is the bottom one
        let shot = Shot {
            w: 1,
            h: 2,
            bgra: vec![0, 0, 255, 255, 255, 0, 0, 255],
        };
        let data = png(&shot).expect("png");
        let dec = png::Decoder::new(std::io::Cursor::new(data));
        let mut r = dec.read_info().expect("info");
        let mut buf = vec![0; r.output_buffer_size().expect("size")];
        r.next_frame(&mut buf).expect("frame");
        assert_eq!(&buf[..6], &[0, 0, 255, 255, 0, 0]); // top = blue, bottom = red
    }
}
