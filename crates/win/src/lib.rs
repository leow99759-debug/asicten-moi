//! WinAPI wrappers (windows, volume, power, COM, UIA, input).
//! Every side effect is exposed through a trait so tests run against a dry-run mock.

pub mod apps;
#[cfg(windows)]
pub mod audio;
#[cfg(windows)]
pub mod autostart;
pub mod backend;
pub mod keys;
pub mod recorder;
#[cfg(windows)]
pub mod speech;
pub mod system;
#[cfg(windows)]
pub mod window;

/// True when compiled for Windows; other targets only get mock executors.
pub const fn is_windows() -> bool {
    cfg!(windows)
}
