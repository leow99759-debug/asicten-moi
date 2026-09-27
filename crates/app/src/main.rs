// Hide the console window on Windows release builds.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    if let Err(err) = jarvis_app::run() {
        eprintln!("jarvis: fatal: {err:#}");
        std::process::exit(1);
    }
}
