use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Persistent project state for a Fabric mod scaffolded by `fw init`.
/// Stored in `.fw/fabric-writer.yml` inside the mod project root.
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct ModState {
    pub mod_name: String,
    pub mod_id: String,
    pub namespace: String,
    pub package_name: String,
    pub minecraft_version: String,
    pub advanced_options: Vec<String>,
}

/// Save state to an absolute path.
pub fn save_to(path: &PathBuf, state: &ModState) -> Result<()> {
    let yaml = serde_yaml::to_string(state)?;
    std::fs::write(path, yaml)?;
    Ok(())
}
