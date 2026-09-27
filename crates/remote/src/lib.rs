//! LAN remote control (axum + PWA). Off by default, never exposed to the internet.

/// Default state of the remote server (SPEC §10.1: off by default).
pub const ENABLED_BY_DEFAULT: bool = false;

#[cfg(test)]
mod tests {
    #[test]
    fn remote_is_off_by_default() {
        const { assert!(!super::ENABLED_BY_DEFAULT) };
    }
}
