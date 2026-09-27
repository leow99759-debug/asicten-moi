//! Data locations: `%APPDATA%/Jarvis` on Windows.

use std::path::{Path, PathBuf};

use crate::{Error, Result};

#[derive(Debug, Clone)]
pub struct Paths {
    pub root: PathBuf,
}

impl Paths {
    /// `%APPDATA%/Jarvis` (roaming config dir on other OSes). Creates it.
    pub fn from_env() -> Result<Self> {
        let base = dirs::config_dir().ok_or(Error::NoConfigDir)?;
        Self::at(base.join(crate::APP_NAME))
    }

    /// Use an explicit root (tests, portable mode). Creates it.
    pub fn at(root: impl AsRef<Path>) -> Result<Self> {
        let root = root.as_ref().to_path_buf();
        std::fs::create_dir_all(root.join("logs"))?;
        Ok(Self { root })
    }

    pub fn config(&self) -> PathBuf {
        self.root.join("config.json")
    }

    pub fn db(&self) -> PathBuf {
        self.root.join("jarvis.db")
    }

    pub fn logs(&self) -> PathBuf {
        self.root.join("logs")
    }
}

/// Where models/voices live: `JARVIS_ASSETS`, then `assets/` next to the exe or in the
/// installer resources, then the repo's `assets/` (dev builds).
pub fn find_assets(resource_dir: Option<&Path>) -> Option<PathBuf> {
    let exe_dir = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(Path::to_path_buf));
    let dev = cfg!(debug_assertions).then(|| Path::new(env!("CARGO_MANIFEST_DIR")).join("../.."));
    std::env::var_os("JARVIS_ASSETS")
        .map(PathBuf::from)
        .into_iter()
        .chain(
            [exe_dir, resource_dir.map(Path::to_path_buf), dev]
                .into_iter()
                .flatten()
                .map(|d| d.join("assets")),
        )
        .find(|p| p.is_dir())
}
