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
    #[serde(default)]
    pub items: Vec<Item>,
    pub java_path: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Item {
    pub id: String,
    pub material: String,
    pub damage: i32,
    pub durability: i32,
}

/// Save state to an absolute path.
pub fn save_to(path: &PathBuf, state: &ModState) -> Result<()> {
    let yaml = serde_yaml::to_string(state)?;
    std::fs::write(path, yaml)?;
    
    Ok(())
}

/// Load state from the current project's `.fw/fabric-writer.yml`.
pub fn load() -> Result<ModState> {
    let path = PathBuf::from(".fw/fabric-writer.yml");
    if !path.exists() {
        anyhow::bail!("fabric-writer.yml not found, are you in a fw-init project?");
    }

    let text = std::fs::read_to_string(path)?;
    let state: ModState = serde_yaml::from_str(&text)?;

    Ok(state)
}
