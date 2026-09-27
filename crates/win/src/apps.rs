//! Locate installed apps for `%CHROME%`-style path variables (SPEC §4.1):
//! registry «App Paths» (HKCU, HKLM), then Start Menu shortcuts named like the exe.
//! Full Start Menu/UWP index with fuzzy search comes with `Launch.Find` (T080).

use std::path::{Path, PathBuf};

use jarvis_core::commands::AppLocator;

pub struct SystemApps;

impl AppLocator for SystemApps {
    fn locate(&self, exe: &str) -> Option<String> {
        app_paths(exe)
            .or_else(|| find_shortcut(&start_menu_dirs(), exe))
            .map(|p| p.to_string_lossy().into_owned())
    }
}

#[cfg(windows)]
fn app_paths(exe: &str) -> Option<PathBuf> {
    use winreg::enums::{HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE};
    use winreg::RegKey;
    let sub = format!(r"SOFTWARE\Microsoft\Windows\CurrentVersion\App Paths\{exe}");
    [HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE]
        .into_iter()
        .find_map(|hive| {
            let v: String = RegKey::predef(hive)
                .open_subkey(&sub)
                .ok()?
                .get_value("")
                .ok()?;
            let p = PathBuf::from(v.trim_matches('"'));
            p.exists().then_some(p)
        })
}

#[cfg(not(windows))]
fn app_paths(_exe: &str) -> Option<PathBuf> {
    None
}

fn start_menu_dirs() -> Vec<PathBuf> {
    ["APPDATA", "PROGRAMDATA"]
        .iter()
        .filter_map(std::env::var_os)
        .map(|base| PathBuf::from(base).join(r"Microsoft\Windows\Start Menu\Programs"))
        .collect()
}

/// First `*.lnk` (recursive) whose name equals the exe stem, case-insensitive.
/// Launching the .lnk itself works, so no need to resolve its target.
pub fn find_shortcut(dirs: &[PathBuf], exe: &str) -> Option<PathBuf> {
    let stem = Path::new(exe).file_stem()?.to_string_lossy().to_lowercase();
    dirs.iter().find_map(|d| walk(d, &stem, 4))
}

fn walk(dir: &Path, stem: &str, depth: u8) -> Option<PathBuf> {
    let entries = std::fs::read_dir(dir).ok()?;
    let mut subdirs = Vec::new();
    for e in entries.flatten() {
        let p = e.path();
        if p.is_dir() {
            subdirs.push(p);
        } else if p.extension().is_some_and(|x| x.eq_ignore_ascii_case("lnk"))
            && p.file_stem()
                .is_some_and(|s| s.to_string_lossy().to_lowercase() == stem)
        {
            return Some(p);
        }
    }
    if depth == 0 {
        return None;
    }
    subdirs.into_iter().find_map(|d| walk(&d, stem, depth - 1))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_shortcut_recursively_case_insensitive() {
        let root = tempfile::tempdir().expect("tmp");
        let sub = root.path().join("Telegram Desktop");
        std::fs::create_dir_all(&sub).expect("mkdir");
        std::fs::write(sub.join("Telegram.lnk"), b"").expect("w");
        std::fs::write(root.path().join("Other.lnk"), b"").expect("w");
        let dirs = vec![root.path().to_path_buf()];
        assert_eq!(
            find_shortcut(&dirs, "telegram.EXE"),
            Some(sub.join("Telegram.lnk"))
        );
        assert_eq!(find_shortcut(&dirs, "spotify.exe"), None);
    }

    #[cfg(windows)]
    #[test]
    fn edge_is_registered_in_app_paths() {
        // windows-latest runners ship Edge
        assert!(SystemApps.locate("msedge.exe").is_some());
    }
}
