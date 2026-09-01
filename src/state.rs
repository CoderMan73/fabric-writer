use anyhow::{Result, bail};
use serde::{Deserialize, Serialize};
use serde_yaml::{from_str, to_string};
use std::collections::HashMap;
use std::fs::{read_to_string, write};
use std::path::PathBuf;

const STATE_FILE: &str = ".fw/fabric-writer.yml";

impl ModState {
    pub fn has_id(&self, entity: &Entity) -> bool {
        let id = entity.id();
        match entity {
            Entity::Item(_) => self.items.iter().any(|i| i.id == *id),
            Entity::Block(_) => self.blocks.iter().any(|b| b.id == *id),
            Entity::Recipe(_) => self.recipes.iter().any(|r| r.id == *id),
        }
    }

    pub fn add(&mut self, entity: Entity) -> Result<()> {
        if self.has_id(&entity) {
            bail!("{} '{}' already exists.", entity.kind(), entity.id());
        }
        match entity {
            Entity::Item(item) => self.items.push(item),
            Entity::Block(block) => self.blocks.push(block),
            Entity::Recipe(recipe) => self.recipes.push(recipe),
        }
        Ok(())
    }

    pub fn remove(&mut self, entity: Entity) -> Result<()> {
        let (id, label, before, after) = match entity {
            Entity::Item(item) => {
                let before = self.items.len();
                self.items.retain(|i| i.id != item.id);
                (item.id, "items", before, self.items.len())
            }
            Entity::Block(block) => {
                let before = self.blocks.len();
                self.blocks.retain(|b| b.id != block.id);
                (block.id, "blocks", before, self.blocks.len())
            }
            Entity::Recipe(recipe) => {
                let before = self.recipes.len();
                self.recipes.retain(|r| r.id != recipe.id);
                (recipe.id, "recipes", before, self.recipes.len())
            }
        };
        if before == after {
            bail!("'{}' not found in {}.", id, label);
        }
        Ok(())
    }

    pub fn save(&self) -> Result<()> {
        save_to(&PathBuf::from(STATE_FILE), self)
    }
}

pub fn save_to(path: &PathBuf, state: &ModState) -> Result<()> {
    let yaml = to_string(state)?;
    write(path, yaml)?;
    Ok(())
}

pub fn load() -> Result<ModState> {
    let path = PathBuf::from(STATE_FILE);
    if !path.exists() {
        bail!("No fabric-writer project found. Are you in the right directory?");
    }
    let text = read_to_string(path)?;
    let state: ModState = from_str(&text)?;
    Ok(state)
}

pub enum Entity {
    Item(Item),
    Block(Block),
    Recipe(Recipe),
}

impl Entity {
    fn id(&self) -> &str {
        match self {
            Entity::Item(i) => &i.id,
            Entity::Block(b) => &b.id,
            Entity::Recipe(r) => &r.id,
        }
    }

    fn kind(&self) -> &'static str {
        match self {
            Entity::Item(_) => "Item",
            Entity::Block(_) => "Block",
            Entity::Recipe(_) => "Recipe",
        }
    }
}

impl Item {
    pub fn new(raw: &str) -> Result<Self> {
        let id = raw.trim().to_lowercase();
        if id.is_empty() {
            bail!("Item id cannot be empty.");
        }
        Ok(Self {
            id,
            kind: ItemKind::Basic,
            material: None,
            attack_damage: None,
            attack_speed: None,
            durability: None,
        })
    }
}

impl Block {
    pub fn new(raw: &str) -> Result<Self> {
        let id = raw.trim().to_lowercase();
        if id.is_empty() {
            bail!("Block id cannot be empty.");
        }
        Ok(Self { id })
    }
}

impl Recipe {
    pub fn new(id: &str) -> Result<Self> {
        let id = id.trim().to_lowercase();
        if id.is_empty() {
            bail!("Recipe id cannot be empty.");
        }
        Ok(Self {
            id,
            kind: "crafting_shaped".into(),
            pattern: vec!["   ".into(), "   ".into(), "   ".into()],
            ingredients: HashMap::new(),
            result: String::new(),
            count: 1,
        })
    }
}

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
    #[serde(default)]
    pub blocks: Vec<Block>,
    #[serde(default)]
    pub recipes: Vec<Recipe>,
    pub java_path: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Item {
    pub id: String,
    #[serde(default)]
    pub kind: ItemKind,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub material: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub attack_damage: Option<f32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub attack_speed: Option<f32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub durability: Option<i32>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ItemKind {
    #[default]
    Basic,
    Tool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Block {
    pub id: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Recipe {
    pub id: String,
    #[serde(rename = "type")]
    pub kind: String,
    pub pattern: Vec<String>,
    pub ingredients: HashMap<String, String>,
    pub result: String,
    pub count: u32,
}
