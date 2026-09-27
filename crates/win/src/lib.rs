//! WinAPI wrappers (windows, volume, power, COM, UIA, input).
//! Every side effect is exposed through a trait so tests run against a dry-run mock.

/// True when compiled for Windows; other targets only get mock executors.
pub const fn is_windows() -> bool {
    cfg!(windows)
}

#[cfg(test)]
mod tests {
    #[test]
    fn target_flag_matches_cfg() {
        assert_eq!(super::is_windows(), cfg!(target_os = "windows"));
    }
}
