pub mod bookmarks;
pub mod history;
pub mod tabs;

use anyhow::Result;
use std::path::PathBuf;

pub fn config_dir() -> Result<PathBuf> {
    let base = dirs::config_dir()
        .ok_or_else(|| anyhow::anyhow!("Cannot determine config directory"))?;
    let dir = base.join("cgrep");
    std::fs::create_dir_all(&dir)?;
    Ok(dir)
}
