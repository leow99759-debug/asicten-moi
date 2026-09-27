//! Optional Jarvis AI module (M13): llama-server manager, tool-calling, scenario builder.
//! Disabled by default; the core never depends on it at runtime.

/// Default state of the AI module (SPEC §0 3a: off by default).
pub const ENABLED_BY_DEFAULT: bool = false;

#[cfg(test)]
mod tests {
    #[test]
    fn ai_is_off_by_default() {
        const { assert!(!super::ENABLED_BY_DEFAULT) };
    }
}
