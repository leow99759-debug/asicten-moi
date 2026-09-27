//! Tauri shell: wires the core to the UI windows.

use anyhow::Context;

/// Start the Tauri application and block until exit.
pub fn run() -> anyhow::Result<()> {
    tauri::Builder::default()
        .run(tauri::generate_context!())
        .context("tauri runtime failed")
}
