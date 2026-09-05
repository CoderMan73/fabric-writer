use anyhow::{Result, bail};
use serde::{Deserialize, Serialize};
use serde_yaml::{from_str, to_string};
use std::collections::HashMap;
use std::fs::{read_to_string, write};
use std::path::PathBuf;

const STATE_FILE: &str = ".fw/fabric-writer.yml";

impl ModState {
    // TODO: Should this allow repeat ID's if they are of different Entity Type? Possible bug.
    /// Returns `true` if the given entity's id already exists in the state.
    pub fn has_id(&self, entity: &Entity) -> bool {
        let id = entity.id();
        match entity {
            Entity::Item(_) => self.items.iter().any(|i| i.id == *id),
            Entity::Block(_) => self.blocks.iter().any(|b| b.id == *id),
            Entity::Recipe(_) => self.recipes.iter().any(|r| r.id == *id),
        }
    }

    /// Appends an entity to the appropriate collection, erroring if the id
    /// already exists.
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

    /// Removes an entity by id, erroring if it's not present.
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

    /// Saves the state to the default `.fw/fabric-writer.yml` path.
    pub fn save(&self) -> Result<()> {
        save_to(&PathBuf::from(STATE_FILE), self)
    }
}

/// Serializes `state` as YAML and writes it to `path`, creating parent dirs.
pub fn save_to(path: &PathBuf, state: &ModState) -> Result<()> {
    let yaml = to_string(state)?;
    write(path, yaml)?;
    Ok(())
}

/// Loads the [`ModState`] from `.fw/fabric-writer.yml` in the current directory.
///
/// # Errors
///
/// Returns an error if the state file does not exist or cannot be deserialized.
pub fn load() -> Result<ModState> {
    let path = PathBuf::from(STATE_FILE);
    if !path.exists() {
        bail!("No fabric-writer project found. Are you in the right directory?");
    }
    let text = read_to_string(path)?;
    let state: ModState = from_str(&text)?;
    Ok(state)
}

/// A stateful entity that can be added to or removed from a [`ModState`].
pub enum Entity {
    /// An item entity.
    Item(Item),
    /// A block entity.
    Block(Block),
    /// A recipe entity.
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
    /// Creates an [`Item`] from a raw id string.
    pub fn new(raw: &str) -> Result<Self> {
        let Some(id) = normalize_id(raw) else {
            bail!("Item id cannot be empty.");
        };
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
    /// Creates a [`Block`] from a raw id string.
    pub fn new(raw: &str) -> Result<Self> {
        let Some(id) = normalize_id(raw) else {
            bail!("Block id cannot be empty.");
        };
        Ok(Self { id })
    }
}

impl Recipe {
    /// Creates a [`Recipe`] with default `crafting_shaped` kind and a 3×3 empty pattern.
    pub fn new(id: &str) -> Result<Self> {
        let Some(id) = normalize_id(id) else {
            bail!("Recipe id cannot be empty.");
        };
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

fn normalize_id(raw: &str) -> Option<String> {
    let id = raw.trim().to_lowercase();
    if id.is_empty() { None } else { Some(id) }
}

/// Serializable mod configuration stored in `.fw/fabric-writer.yml`.
///
/// This is the single source of truth for a fabric-writer project.
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct ModState {
    /// Human-readable name of the mod.
    pub mod_name: String,

    /// Lowercase identifier with only alphanumeric, `_`, and `-`.
    pub mod_id: String,

    /// Namespace used in resource locations (defaults to `mod_id`).
    pub namespace: String,

    /// Java package name (lowercase, no dashes).
    pub package_name: String,

    /// Minecraft version string (e.g. `"26.2"`).
    pub minecraft_version: String,

    /// Advanced options passed to `fabric init`.
    pub advanced_options: Vec<String>,

    /// Path to the JDK used by Gradle.
    pub java_path: String,

    /// Items tracked in state.
    #[serde(default)]
    pub items: Vec<Item>,

    /// Blocks tracked in state.
    #[serde(default)]
    pub blocks: Vec<Block>,

    /// Recipes tracked in state.
    #[serde(default)]
    pub recipes: Vec<Recipe>,
}

/// An item tracked in [`ModState::items`].
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Item {
    /// Lowercase item identifier.
    pub id: String,

    /// `Basic` or `Tool`; tool items get sword/material properties.
    #[serde(default)]
    pub kind: ItemKind,

    /// Tool material (e.g. `"diamond"`); None for basic items.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub material: Option<String>,

    /// Attack damage bonus for tool items.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub attack_damage: Option<f32>,

    /// Attack speed modifier for tool items.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub attack_speed: Option<f32>,

    /// Custom durability override for tool items.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub durability: Option<i32>,
}

/// Whether an [`Item`] is a basic item or a tool.
#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ItemKind {
    /// A basic (non-tool) item.
    #[default]
    Basic,

    /// A tool item with material/damage/speed/durability properties.
    Tool,
}

/// A block tracked in [`ModState::blocks`].
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Block {
    /// Lowercase block identifier.
    pub id: String,
}

/// A crafting recipe tracked in [`ModState::recipes`].
/// Currently unimplemented in codegen.
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Recipe {
    /// Lowercase recipe identifier.
    pub id: String,

    /// Recipe type (e.g. `"crafting_shaped"`), serialized as `"type"`.
    #[serde(rename = "type")]
    pub kind: String,

    /// Crafting pattern grid (Vec of strings).
    pub pattern: Vec<String>,

    /// Ingredient key→item mapping.
    pub ingredients: HashMap<String, String>,

    /// Result item id.
    pub result: String,

    /// Number of items produced.
    pub count: u32,
}
