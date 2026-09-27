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

/// Pin the process MTA for good. WinRT calls from threads that never joined an apartment
/// run in the implicit MTA, which dies with the last explicit MTA thread (e.g. a finished
/// audio worker); the next call then hits freed objects: STATUS_ACCESS_VIOLATION.
#[cfg(windows)]
pub(crate) fn keep_mta() {
    static ONCE: std::sync::Once = std::sync::Once::new();
    ONCE.call_once(|| {
        // SAFETY: plain COM call; the cookie is never released on purpose.
        let _ = unsafe { windows::Win32::System::Com::CoIncrementMTAUsage() };
    });
}

/// True when compiled for Windows; other targets only get mock executors.
pub const fn is_windows() -> bool {
    cfg!(windows)
}
