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
            Entity::CreativeTab(_) => self.creative_tabs.iter().any(|t| t.id == *id),
        }
    }

    /// Returns `true` if the given id is already used by a different entity type.
    pub fn has_cross_type_id(&self, id: &str) -> bool {
        self.items.iter().any(|i| i.id == id)
            || self.blocks.iter().any(|b| b.id == id)
            || self.recipes.iter().any(|r| r.id == id)
            || self.creative_tabs.iter().any(|t| t.id == id)
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
            Entity::CreativeTab(tab) => self.creative_tabs.push(tab),
        }
        Ok(())
    }

    /// Tries to append an entity. Returns `Ok(false)` if it already exists.
    pub fn try_add(&mut self, entity: Entity) -> Result<bool> {
        if self.has_id(&entity) {
            return Ok(false);
        }
        self.add(entity).map(|()| true)
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
            Entity::CreativeTab(tab) => {
                let before = self.creative_tabs.len();
                self.creative_tabs.retain(|t| t.id != tab.id);
                for item in &mut self.items {
                    if item.creative_tab.as_deref() == Some(&tab.id) {
                        item.creative_tab = None;
                    }
                }
                for block in &mut self.blocks {
                    if block.creative_tab.as_deref() == Some(&tab.id) {
                        block.creative_tab = None;
                    }
                }
                (tab.id, "creative_tabs", before, self.creative_tabs.len())
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

/// Loads the [`ModState`] from an arbitrary YAML file.
///
/// # Errors
///
/// Returns an error if the file cannot be read or deserialized.
pub fn load_from(path: &PathBuf) -> Result<ModState> {
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
    /// A creative tab entity.
    CreativeTab(CreativeTab),
}

impl Entity {
    fn id(&self) -> &str {
        match self {
            Entity::Item(i) => &i.id,
            Entity::Block(b) => &b.id,
            Entity::Recipe(r) => &r.id,
            Entity::CreativeTab(t) => &t.id,
        }
    }

    fn kind(&self) -> &'static str {
        match self {
            Entity::Item(_) => "Item",
            Entity::Block(_) => "Block",
            Entity::Recipe(_) => "Recipe",
            Entity::CreativeTab(_) => "CreativeTab",
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
            tooltip: Vec::new(),
            nutrition: None,
            saturation: None,
            always_edible: false,
            entity_type: None,
            burn_time: None,
            compost_chance: None,
            creative_tab: None,
            armor_material: None,
            armor_slot: None,
            effects: Vec::new(),
        })
    }

    /// Creates an [`Item`] with a default creative tab assignment.
    pub fn base(id: &str, tab: &str) -> Self {
        Self {
            id: id.into(),
            kind: ItemKind::Basic,
            material: None,
            attack_damage: None,
            attack_speed: None,
            durability: None,
            tooltip: Vec::new(),
            nutrition: None,
            saturation: None,
            always_edible: false,
            entity_type: None,
            burn_time: None,
            compost_chance: None,
            creative_tab: Some(tab.into()),
            armor_material: None,
            armor_slot: None,
            effects: Vec::new(),
        }
    }
}

impl Block {
    /// Creates a [`Block`] from a raw id string.
    pub fn new(raw: &str) -> Result<Self> {
        let Some(id) = normalize_id(raw) else {
            bail!("Block id cannot be empty.");
        };
        Ok(Self {
            id,
            creative_tab: None,
        })
    }

    /// Creates a [`Block`] with a default creative tab assignment.
    pub fn base(id: &str, tab: &str) -> Self {
        Self {
            id: id.into(),
            creative_tab: Some(tab.into()),
        }
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

impl CreativeTab {
    /// Creates a [`CreativeTab`] from a raw id string.
    pub fn new(raw: &str) -> Result<Self> {
        let Some(id) = normalize_id(raw) else {
            bail!("Creative tab id cannot be empty.");
        };
        Ok(Self { id })
    }
}

pub(crate) fn normalize_id(raw: &str) -> Option<String> {
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
    #[serde(default)]
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

    /// Creative tabs tracked in state.
    #[serde(default)]
    pub creative_tabs: Vec<CreativeTab>,
}

/// A creative tab tracked in [`ModState::creative_tabs`].
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct CreativeTab {
    /// Lowercase creative tab identifier.
    pub id: String,
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

    /// Tooltip lines shown in item description.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tooltip: Vec<String>,

    /// Nutrition value for food items.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub nutrition: Option<i32>,

    /// Saturation modifier for food items.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub saturation: Option<f32>,

    /// Whether the food item can always be eaten.
    #[serde(default)]
    pub always_edible: bool,

    /// Entity type for spawn egg items.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub entity_type: Option<String>,

    /// Burn time in ticks for fuel items.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub burn_time: Option<i32>,

    /// Compost chance for compostable items (0.0 to 1.0).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub compost_chance: Option<f32>,

    /// Creative tab assignment (`None` means default to first custom tab or `ingredients`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub creative_tab: Option<String>,

    /// Armor material (e.g. "diamond", "iron", "netherite"); None for non-armor items.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub armor_material: Option<String>,

    /// Armor slot (helmet, chestplate, leggings, boots); None for non-armor items.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub armor_slot: Option<String>,

    /// Custom potion effects for potion items.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub effects: Vec<PotionEffect>,
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

    /// An axe item.
    Axe,

    /// A shovel item.
    Shovel,

    /// A hoe item.
    Hoe,

    /// A food item with nutrition/saturation properties.
    Food,

    /// A spawn egg for an entity type.
    SpawnEgg,

    /// A fuel item with a burn time.
    Fuel,

    /// A compostable item with a chance to increase composter level.
    Compostable,

    /// An armor item with material and slot properties.
    Armor,

    /// A shield item.
    Shield,

    /// A potion item with custom effects.
    Potion,
}

/// A potion effect with type, duration, and amplifier.
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct PotionEffect {
    /// Effect type (e.g. "minecraft:speed").
    #[serde(default)]
    pub effect_type: String,

    /// Duration in ticks.
    #[serde(default)]
    pub duration: i32,

    /// Amplifier (0 = level I).
    #[serde(default)]
    pub amplifier: i32,
}

/// A block tracked in [`ModState::blocks`].
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Block {
    /// Lowercase block identifier.
    pub id: String,

    /// Creative tab assignment (`None` means default to first custom tab or `ingredients`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub creative_tab: Option<String>,
}

/// A crafting recipe tracked in [`ModState::recipes`].
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
