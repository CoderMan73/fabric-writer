use anyhow::Result;
use crate::item::Item;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

const STATE_FILE: &str = ".fw/fabric-writer.yml";

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

impl ModState {
    pub fn has_item_id(&self, id: &str) -> bool {
        self.items.iter().any(|i| i.id == id)
    }

    pub fn add_item(&mut self, item: Item) -> Result<()> {
        if self.has_item_id(&item.id) {
            anyhow::bail!("Item '{}' already exists.", item.id);
        }
        self.items.push(item);
        Ok(())
    }

    pub fn save(&self) -> Result<()> {
        save_to(&PathBuf::from(STATE_FILE), self)
    }
}

/// Save state to an absolute path.
pub fn save_to(path: &PathBuf, state: &ModState) -> Result<()> {
    let yaml = serde_yaml::to_string(state)?;
    std::fs::write(path, yaml)?;

    Ok(())
}

/// Load state from the current project's `.fw/fabric-writer.yml`.
pub fn load() -> Result<ModState> {
    let path = PathBuf::from(STATE_FILE);
    if !path.exists() {
        anyhow::bail!("fabric-writer.yml not found, are you in a fw-init project?");
    }

    let text = std::fs::read_to_string(path)?;
    let state: ModState = serde_yaml::from_str(&text)?;

    Ok(state)
}
