//! «Автозапуск программы» (SPEC §10.4): HKCU Run key, no admin rights needed.

const RUN: &str = r"Software\Microsoft\Windows\CurrentVersion\Run";
const NAME: &str = "Jarvis";

/// Start `exe` (with `--minimized`) at logon, or stop doing so.
pub fn set(on: bool, exe: &std::path::Path) -> Result<(), String> {
    use winreg::enums::{HKEY_CURRENT_USER, KEY_SET_VALUE};
    use winreg::RegKey;
    let key = RegKey::predef(HKEY_CURRENT_USER)
        .open_subkey_with_flags(RUN, KEY_SET_VALUE)
        .map_err(|e| e.to_string())?;
    if on {
        let cmd = format!("\"{}\" --minimized", exe.display());
        key.set_value(NAME, &cmd).map_err(|e| e.to_string())
    } else {
        match key.delete_value(NAME) {
            Err(e) if e.kind() != std::io::ErrorKind::NotFound => Err(e.to_string()),
            _ => Ok(()),
        }
    }
}
