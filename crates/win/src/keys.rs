//! Keyboard/mouse via SendInput (SPEC §4.4 Keys.*, Mouse.*). Combo parsing is pure and tested;
//! sending is Windows-only and never runs in tests (guardrail: tests don't type/click).

/// Virtual-key code for a key name (`ctrl`, `shift`, `s`, `f5`, `enter`, `media_next`…).
pub fn vk(name: &str) -> Option<u16> {
    let n = name.trim().to_lowercase();
    let code = match n.as_str() {
        "ctrl" | "control" | "ctl" => 0x11,
        "shift" => 0x10,
        "alt" => 0x12,
        "win" | "super" | "meta" => 0x5B,
        "enter" | "return" => 0x0D,
        "esc" | "escape" => 0x1B,
        "tab" => 0x09,
        "space" => 0x20,
        "backspace" => 0x08,
        "delete" | "del" => 0x2E,
        "insert" | "ins" => 0x2D,
        "home" => 0x24,
        "end" => 0x23,
        "pageup" | "pgup" => 0x21,
        "pagedown" | "pgdn" => 0x22,
        "left" => 0x25,
        "up" => 0x26,
        "right" => 0x27,
        "down" => 0x28,
        "printscreen" | "prtsc" => 0x2C,
        "capslock" => 0x14,
        "volume_mute" => 0xAD,
        "volume_down" => 0xAE,
        "volume_up" => 0xAF,
        "media_next" => 0xB0,
        "media_prev" => 0xB1,
        "media_stop" => 0xB2,
        "media_play_pause" => 0xB3,
        _ => {
            let mut chars = n.chars();
            match (chars.next(), chars.as_str()) {
                (Some(c @ 'a'..='z'), "") => c.to_ascii_uppercase() as u16,
                (Some(c @ '0'..='9'), "") => c as u16,
                (Some('f'), num) => match num.parse::<u16>() {
                    Ok(k @ 1..=24) => 0x6F + k,
                    _ => return None,
                },
                _ => return None,
            }
        }
    };
    Some(code)
}

/// Key name for a VK code, as packs write it (`Ctrl`, `S`, `F5`, `Enter`); inverse of [`vk`].
pub fn name(code: u16) -> Option<String> {
    const NAMED: [&str; 28] = [
        "ctrl",
        "shift",
        "alt",
        "win",
        "enter",
        "esc",
        "tab",
        "space",
        "backspace",
        "delete",
        "insert",
        "home",
        "end",
        "pageup",
        "pagedown",
        "left",
        "up",
        "right",
        "down",
        "printscreen",
        "capslock",
        "volume_mute",
        "volume_down",
        "volume_up",
        "media_next",
        "media_prev",
        "media_stop",
        "media_play_pause",
    ];
    let code = match code {
        0xA0 | 0xA1 => 0x10,
        0xA2 | 0xA3 => 0x11,
        0xA4 | 0xA5 => 0x12,
        0x5C => 0x5B,
        c => c,
    };
    let n = match code {
        0x41..=0x5A | 0x30..=0x39 => (code as u8 as char).to_string(),
        0x70..=0x87 => format!("F{}", code - 0x6F),
        _ => {
            let n = NAMED.iter().find(|n| vk(n) == Some(code))?;
            let mut c = n.chars();
            c.next()
                .map(|f| f.to_ascii_uppercase().to_string() + c.as_str())?
        }
    };
    Some(n)
}

/// `"ctrl+shift+s"` → VK codes in press order.
pub fn parse_combo(combo: &str) -> Result<Vec<u16>, String> {
    combo
        .split('+')
        .map(|k| vk(k).ok_or_else(|| format!("неизвестная клавиша «{k}»")))
        .collect()
}

#[cfg_attr(not(windows), allow(dead_code))]
fn is_extended(vk: u16) -> bool {
    matches!(vk, 0x21..=0x28 | 0x2D | 0x2E | 0x5B | 0xAD..=0xB3)
}

#[cfg(windows)]
mod send {
    use windows::Win32::UI::Input::KeyboardAndMouse::{
        SendInput, INPUT, INPUT_0, INPUT_KEYBOARD, INPUT_MOUSE, KEYBDINPUT, KEYBD_EVENT_FLAGS,
        KEYEVENTF_EXTENDEDKEY, KEYEVENTF_KEYUP, KEYEVENTF_UNICODE, MOUSEEVENTF_LEFTDOWN,
        MOUSEEVENTF_LEFTUP, MOUSEEVENTF_RIGHTDOWN, MOUSEEVENTF_RIGHTUP, MOUSEEVENTF_WHEEL,
        MOUSEINPUT, MOUSE_EVENT_FLAGS, VIRTUAL_KEY,
    };
    use windows::Win32::UI::WindowsAndMessaging::SetCursorPos;

    fn key(vk: u16, scan: u16, flags: KEYBD_EVENT_FLAGS) -> INPUT {
        INPUT {
            r#type: INPUT_KEYBOARD,
            Anonymous: INPUT_0 {
                ki: KEYBDINPUT {
                    wVk: VIRTUAL_KEY(vk),
                    wScan: scan,
                    dwFlags: flags,
                    time: 0,
                    dwExtraInfo: 0,
                },
            },
        }
    }

    fn mouse(flags: MOUSE_EVENT_FLAGS, data: i32) -> INPUT {
        INPUT {
            r#type: INPUT_MOUSE,
            Anonymous: INPUT_0 {
                mi: MOUSEINPUT {
                    dx: 0,
                    dy: 0,
                    mouseData: data as u32,
                    dwFlags: flags,
                    time: 0,
                    dwExtraInfo: 0,
                },
            },
        }
    }

    fn send(inputs: &[INPUT]) -> Result<(), String> {
        // SAFETY: slice of fully initialised INPUT structs.
        let n = unsafe { SendInput(inputs, std::mem::size_of::<INPUT>() as i32) };
        if n as usize == inputs.len() {
            Ok(())
        } else {
            Err("SendInput заблокирован".into())
        }
    }

    fn flags(vk: u16, up: bool) -> KEYBD_EVENT_FLAGS {
        let mut f = KEYBD_EVENT_FLAGS(0);
        if super::is_extended(vk) {
            f |= KEYEVENTF_EXTENDEDKEY;
        }
        if up {
            f |= KEYEVENTF_KEYUP;
        }
        f
    }

    pub fn down(vks: &[u16]) -> Result<(), String> {
        send(
            &vks.iter()
                .map(|&v| key(v, 0, flags(v, false)))
                .collect::<Vec<_>>(),
        )
    }

    pub fn up(vks: &[u16]) -> Result<(), String> {
        send(
            &vks.iter()
                .rev()
                .map(|&v| key(v, 0, flags(v, true)))
                .collect::<Vec<_>>(),
        )
    }

    pub fn press(vks: &[u16]) -> Result<(), String> {
        down(vks)?;
        up(vks)
    }

    pub fn type_text(text: &str) -> Result<(), String> {
        let inputs: Vec<INPUT> = text
            .encode_utf16()
            .flat_map(|u| {
                [
                    key(0, u, KEYEVENTF_UNICODE),
                    key(0, u, KEYEVENTF_UNICODE | KEYEVENTF_KEYUP),
                ]
            })
            .collect();
        send(&inputs)
    }

    pub fn move_to(x: i32, y: i32) -> Result<(), String> {
        // SAFETY: plain Win32 call with value arguments.
        unsafe { SetCursorPos(x, y) }.map_err(|e| e.to_string())
    }

    pub fn click(right: bool, times: u32) -> Result<(), String> {
        let (d, u) = if right {
            (MOUSEEVENTF_RIGHTDOWN, MOUSEEVENTF_RIGHTUP)
        } else {
            (MOUSEEVENTF_LEFTDOWN, MOUSEEVENTF_LEFTUP)
        };
        for _ in 0..times.clamp(1, 50) {
            send(&[mouse(d, 0), mouse(u, 0)])?;
        }
        Ok(())
    }

    pub fn scroll(notches: i32) -> Result<(), String> {
        send(&[mouse(MOUSEEVENTF_WHEEL, notches * 120)])
    }
}

#[cfg(windows)]
pub use send::{click, down, move_to, press, scroll, type_text, up};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn repo_pack_hotkeys_parse() {
        use jarvis_core::commands::{addons, Action};
        let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../packs");
        for pack in addons(&dir, &[]).into_iter().map(|a| a.pack) {
            for c in &pack.commands {
                for a in &c.actions {
                    if let Action::KeysPress { keys } | Action::KeysHold { key: keys, .. } = a {
                        assert!(parse_combo(keys).is_ok(), "{}: {keys}", c.id);
                    }
                }
            }
        }
    }

    #[test]
    fn parses_combos() {
        assert_eq!(parse_combo("ctrl+shift+s"), Ok(vec![0x11, 0x10, 0x53]));
        assert_eq!(parse_combo("Win+D"), Ok(vec![0x5B, 0x44]));
        assert_eq!(parse_combo("alt+f4"), Ok(vec![0x12, 0x73]));
        assert_eq!(parse_combo("f11"), Ok(vec![0x7A]));
        assert_eq!(parse_combo("ctrl+1"), Ok(vec![0x11, 0x31]));
        assert_eq!(parse_combo("media_play_pause"), Ok(vec![0xB3]));
        assert!(parse_combo("ctrl+щ").is_err());
        assert!(parse_combo("f25").is_err());
        assert!(is_extended(0x25) && !is_extended(0x41));
        for (code, n) in [
            (0xA2, "Ctrl"),
            (0x53, "S"),
            (0x74, "F5"),
            (0x0D, "Enter"),
            (0x5C, "Win"),
        ] {
            assert_eq!(name(code).as_deref(), Some(n));
        }
        assert_eq!(name(0xBA), None);
    }
}
