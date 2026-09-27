//! WinAPI wrappers (windows, volume, power, COM, UIA, input).
//! Every side effect is exposed through a trait so tests run against a dry-run mock.

pub mod apps;

/// True when compiled for Windows; other targets only get mock executors.
pub const fn is_windows() -> bool {
    cfg!(windows)
}
