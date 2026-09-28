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

/// Spoken Russian name → Start Menu name, or a system exe (no shortcut on Windows 11).
const ALIASES: &[(&str, &str)] = &[
    ("вс код", "visual studio code"),
    ("вскод", "visual studio code"),
    ("visual studio код", "visual studio code"),
    ("визуал студио код", "visual studio code"),
    ("код", "visual studio code"),
    ("стим", "steam"),
    ("дискорд", "discord"),
    ("телеграм", "telegram"),
    ("спотифай", "spotify"),
    ("ватсап", "whatsapp"),
    ("вотсап", "whatsapp"),
    ("зум", "zoom"),
    ("ворд", "word"),
    ("эксель", "excel"),
    ("пауэрпоинт", "powerpoint"),
    ("поверпоинт", "powerpoint"),
    ("фотошоп", "photoshop"),
    ("премьер", "premiere pro"),
    ("обс", "obs studio"),
    ("блендер", "blender"),
    ("фигма", "figma"),
    ("ноушен", "notion"),
    ("капкат", "capcut"),
    ("кап кат", "capcut"),
    ("хром", "google chrome"),
    ("гугл хром", "google chrome"),
    ("эдж", "microsoft edge"),
    ("опера", "opera"),
    ("яндекс", "yandex"),
    ("эпик геймс", "epic games launcher"),
    ("майнкрафт", "minecraft"),
    ("блокнот", "notepad.exe"),
    ("калькулятор", "calc.exe"),
    ("пейнт", "mspaint.exe"),
    ("диспетчер задач", "taskmgr.exe"),
    ("панель управления", "control.exe"),
    ("командную строку", "cmd.exe"),
    ("командная строка", "cmd.exe"),
    ("терминал", "wt.exe"),
    ("протон", "proton vpn"),
    ("протон впн", "proton vpn"),
    ("протон vpn", "proton vpn"),
    ("окто", "octo browser"),
    ("окто браузер", "octo browser"),
    ("гитхаб", "github"),
    ("чат гпт", "chatgpt"),
    ("чатгпт", "chatgpt"),
    ("джимейл", "gmail"),
    ("почту", "gmail"),
    ("вк", "vk"),
    ("вконтакте", "vk"),
];

/// Not installed (or a site anyway) → open it in the browser.
const WEB: &[(&str, &str)] = &[
    ("telegram", "https://web.telegram.org"),
    ("discord", "https://discord.com/app"),
    ("spotify", "https://open.spotify.com"),
    ("whatsapp", "https://web.whatsapp.com"),
    ("github", "https://github.com"),
    ("chatgpt", "https://chatgpt.com"),
    ("gmail", "https://mail.google.com"),
    ("vk", "https://vk.com"),
    ("figma", "https://www.figma.com"),
    ("notion", "https://www.notion.so"),
];

/// Cyrillic → Latin, so «дискорд» still meets «Discord.lnk» without an alias.
fn translit(s: &str) -> String {
    const MAP: [(char, &str); 33] = [
        ('а', "a"),
        ('б', "b"),
        ('в', "v"),
        ('г', "g"),
        ('д', "d"),
        ('е', "e"),
        ('ё', "e"),
        ('ж', "zh"),
        ('з', "z"),
        ('и', "i"),
        ('й', "y"),
        ('к', "k"),
        ('л', "l"),
        ('м', "m"),
        ('н', "n"),
        ('о', "o"),
        ('п', "p"),
        ('р', "r"),
        ('с', "s"),
        ('т', "t"),
        ('у', "u"),
        ('ф', "f"),
        ('х', "h"),
        ('ц', "ts"),
        ('ч', "ch"),
        ('ш', "sh"),
        ('щ', "sch"),
        ('ъ', ""),
        ('ы', "y"),
        ('ь', ""),
        ('э', "e"),
        ('ю', "yu"),
        ('я', "ya"),
    ];
    s.chars()
        .map(|c| {
            MAP.iter()
                .find(|(k, _)| *k == c)
                .map_or_else(|| c.to_string(), |(_, v)| (*v).to_owned())
        })
        .collect()
}

/// «открой {приложение}» (SPEC §4.1): alias, then Start Menu shortcut by name/translit.
/// Returns what to shell-open: a .lnk path or a system exe.
pub fn resolve(name: &str) -> Option<String> {
    let want = name.to_lowercase().replace('ё', "е");
    let want = ALIASES
        .iter()
        .find(|(k, _)| *k == want)
        .map_or(want, |(_, v)| (*v).to_owned());
    if want.ends_with(".exe") {
        return Some(want);
    }
    find_app(&want)
        .or_else(|| find_app(&translit(&want)))
        .map(|p| p.to_string_lossy().into_owned())
        .or_else(|| {
            WEB.iter()
                .find(|(k, _)| *k == want)
                .map(|(_, url)| (*url).to_owned())
        })
}

/// Start Menu shortcut by human name («Telegram», «спотифай» → best fuzzy match on the .lnk name).
pub fn find_app(name: &str) -> Option<PathBuf> {
    let want = name.to_lowercase();
    let mut all = Vec::new();
    for d in start_menu_dirs() {
        collect_lnk(&d, 4, &mut all);
    }
    best_by_name(&all, &want)
}

pub fn best_by_name(lnks: &[PathBuf], want: &str) -> Option<PathBuf> {
    lnks.iter()
        .filter_map(|p| {
            let stem = p.file_stem()?.to_string_lossy().to_lowercase();
            let score = if stem == want {
                2.0
            } else if stem.contains(want) {
                1.5
            } else {
                strsim::jaro_winkler(&stem, want)
            };
            (score >= 0.85).then(|| (score, p.clone()))
        })
        .max_by(|a, b| a.0.total_cmp(&b.0))
        .map(|(_, p)| p)
}

fn collect_lnk(dir: &Path, depth: u8, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for e in entries.flatten() {
        let p = e.path();
        if p.is_dir() {
            if depth > 0 {
                collect_lnk(&p, depth - 1, out);
            }
        } else if p.extension().is_some_and(|x| x.eq_ignore_ascii_case("lnk")) {
            out.push(p);
        }
    }
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
    fn aliases_and_translit() {
        assert_eq!(resolve("Блокнот").as_deref(), Some("notepad.exe"));
        assert_eq!(resolve("гитхаб").as_deref(), Some("https://github.com"));
        assert_eq!(translit("дискорд"), "diskord");
        let lnks = [PathBuf::from("Discord.lnk"), PathBuf::from("Steam.lnk")];
        assert_eq!(
            best_by_name(&lnks, &translit("дискорд")),
            Some(PathBuf::from("Discord.lnk"))
        );
    }

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

    #[test]
    fn fuzzy_name_prefers_exact_then_contains() {
        let l = |s: &str| PathBuf::from(format!("{s}.lnk"));
        let all = vec![
            l("Telegram"),
            l("Telegram Desktop Uninstall"),
            l("Spotify"),
            l("Steam"),
        ];
        assert_eq!(best_by_name(&all, "telegram"), Some(l("Telegram")));
        assert_eq!(best_by_name(&all, "spotif"), Some(l("Spotify")));
        assert_eq!(best_by_name(&all, "blender"), None);
    }

    #[cfg(windows)]
    #[test]
    fn edge_is_registered_in_app_paths() {
        // windows-latest runners ship Edge
        assert!(SystemApps.locate("msedge.exe").is_some());
    }
}
