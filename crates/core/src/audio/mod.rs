//! Audio input (SPEC §2.1): device capture → 16 kHz mono i16 chunks, ring buffer, level meter.

mod dsp;
pub use dsp::{level, Converter, Ring, SAMPLE_RATE};

#[cfg(windows)]
mod capture;
#[cfg(windows)]
pub use capture::{input_devices, Capture};
