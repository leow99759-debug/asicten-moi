//! `%VAR%` expansion in action paths (SPEC §4.1): environment variables plus app
//! variables (`%CHROME%`, `%STEAM%`…) resolved by an [`AppLocator`] (registry / Start Menu).

/// App variable → executable name looked up by the locator.
pub const APP_VARS: &[(&str, &str)] = &[
    ("CHROME", "chrome.exe"),
    ("EDGE", "msedge.exe"),
    ("FIREFOX", "firefox.exe"),
    ("OPERA", "opera.exe"),
    ("YANDEX", "browser.exe"),
    ("STEAM", "steam.exe"),
    ("EPIC", "EpicGamesLauncher.exe"),
    ("SPOTIFY", "Spotify.exe"),
    ("TELEGRAM", "Telegram.exe"),
    ("DISCORD", "Discord.exe"),
    ("WHATSAPP", "WhatsApp.exe"),
    ("ZOOM", "Zoom.exe"),
    ("OBS", "obs64.exe"),
    ("VSCODE", "Code.exe"),
    ("WORD", "WINWORD.EXE"),
    ("EXCEL", "EXCEL.EXE"),
    ("POWERPOINT", "POWERPNT.EXE"),
];

/// Finds installed apps by exe name (implemented in `jarvis-win`, mocked in tests).
pub trait AppLocator {
    fn locate(&self, exe: &str) -> Option<String>;
}

/// Expand `%NAME%` tokens; unknown tokens are left as-is (the launch then reports the error).
pub fn expand(s: &str, apps: &dyn AppLocator) -> String {
    let mut out = String::with_capacity(s.len());
    let mut rest = s;
    while let Some(start) = rest.find('%') {
        let Some(len) = rest[start + 1..].find('%') else {
            break;
        };
        let name = &rest[start + 1..start + 1 + len];
        out.push_str(&rest[..start]);
        match resolve(name, apps) {
            Some(v) => out.push_str(&v),
            None => out.push_str(&rest[start..start + len + 2]),
        }
        rest = &rest[start + len + 2..];
    }
    out.push_str(rest);
    out
}

fn resolve(name: &str, apps: &dyn AppLocator) -> Option<String> {
    let upper = name.to_uppercase();
    if let Some((_, exe)) = APP_VARS.iter().find(|(v, _)| *v == upper) {
        return apps.locate(exe);
    }
    std::env::var(name).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Fake;
    impl AppLocator for Fake {
        fn locate(&self, exe: &str) -> Option<String> {
            (exe == "chrome.exe").then(|| r"C:\Chrome\chrome.exe".to_owned())
        }
    }

    #[test]
    fn expands_app_and_env_vars() {
        std::env::set_var("JARVIS_TEST_DIR", r"C:\Users\me");
        assert_eq!(expand("%CHROME%", &Fake), r"C:\Chrome\chrome.exe");
        assert_eq!(
            expand("%chrome% --new", &Fake),
            r"C:\Chrome\chrome.exe --new"
        );
        assert_eq!(
            expand(r"%JARVIS_TEST_DIR%\Music", &Fake),
            r"C:\Users\me\Music"
        );
        assert_eq!(expand("%STEAM%", &Fake), "%STEAM%", "not installed stays");
        assert_eq!(expand("100% sure", &Fake), "100% sure");
        assert_eq!(expand("plain", &Fake), "plain");
    }
}
