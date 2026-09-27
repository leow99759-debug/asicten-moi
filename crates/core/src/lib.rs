//! Jarvis core: audio, wake word, STT, NLU, executor, TTS, config, DB, IPC.
//! Platform-independent; Windows side effects live in `jarvis-win` behind traits.

/// Product name shown in UI, tray and logs.
pub const APP_NAME: &str = "Jarvis";

/// Crate version, shared by every workspace crate.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_is_semver() {
        assert_eq!(VERSION.split('.').count(), 3);
        assert_eq!(APP_NAME, "Jarvis");
    }
}
