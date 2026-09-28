use anyhow::{Result, bail};
use serde::{Deserialize, Serialize};
use serde_yaml::{from_str, to_string};
use std::collections::HashMap;
use std::fs::{read_to_string, write};
use std::path::PathBuf;

const STATE_FILE: &str = ".fw/fabric-writer.yml";

impl ModState {
    /// Returns `true` if the given entity's id already exists in the state.
    pub fn has_id(&self, entity: &Entity) -> bool {
        let id = entity.id();
        match entity {
            Entity::Item(_) => self.items.iter().any(|i| i.id == *id),
            Entity::Block(_) => self.blocks.iter().any(|b| b.id == *id),
            Entity::Recipe(_) => self.recipes.iter().any(|r| r.id == *id),
            Entity::CreativeTab(_) => self.creative_tabs.iter().any(|t| t.id == *id),
            Entity::Mob(_) => self.mobs.iter().any(|m| m.id == *id),
            Entity::Biome(_) => self.biomes.iter().any(|b| b.id == *id),
            Entity::Dimension(_) => self.dimensions.iter().any(|d| d.id == *id),
            Entity::DimensionType(_) => self.dimension_types.iter().any(|d| d.id == *id),
            Entity::Structure(_) => self.structures.iter().any(|s| s.id == *id),
            Entity::Feature(_) => self.features.iter().any(|f| f.id == *id),
            Entity::LootTable(_) => self.loot_tables.iter().any(|l| l.id == *id),
            Entity::Advancement(_) => self.advancements.iter().any(|a| a.id == *id),
            Entity::SoundEvent(_) => self.sound_events.iter().any(|s| s.id == *id),
            Entity::Tag(_) => self.tags.iter().any(|t| t.id == *id),
        }
    }

    /// Returns `true` if the given id is already used by a different entity type.
    pub fn has_cross_type_id(&self, id: &str) -> bool {
        self.items.iter().any(|i| i.id == id)
            || self.blocks.iter().any(|b| b.id == id)
            || self.recipes.iter().any(|r| r.id == id)
            || self.creative_tabs.iter().any(|t| t.id == id)
            || self.mobs.iter().any(|m| m.id == id)
            || self.biomes.iter().any(|b| b.id == id)
            || self.dimensions.iter().any(|d| d.id == id)
            || self.dimension_types.iter().any(|d| d.id == id)
            || self.structures.iter().any(|s| s.id == id)
            || self.features.iter().any(|f| f.id == id)
            || self.loot_tables.iter().any(|l| l.id == id)
            || self.advancements.iter().any(|a| a.id == id)
            || self.sound_events.iter().any(|s| s.id == id)
            || self.tags.iter().any(|t| t.id == id)
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
            Entity::Mob(mob) => self.mobs.push(mob),
            Entity::Biome(biome) => self.biomes.push(biome),
            Entity::Dimension(dim) => self.dimensions.push(dim),
            Entity::DimensionType(dim_type) => self.dimension_types.push(dim_type),
            Entity::Structure(structure) => self.structures.push(structure),
            Entity::Feature(feature) => self.features.push(feature),
            Entity::LootTable(loot) => self.loot_tables.push(loot),
            Entity::Advancement(adv) => self.advancements.push(adv),
            Entity::SoundEvent(se) => self.sound_events.push(se),
            Entity::Tag(tag) => self.tags.push(tag),
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
            Entity::Mob(mob) => {
                let before = self.mobs.len();
                self.mobs.retain(|m| m.id != mob.id);
                (mob.id, "mobs", before, self.mobs.len())
            }
            Entity::Biome(biome) => {
                let before = self.biomes.len();
                self.biomes.retain(|b| b.id != biome.id);
                (biome.id, "biomes", before, self.biomes.len())
            }
            Entity::Dimension(dim) => {
                let before = self.dimensions.len();
                self.dimensions.retain(|d| d.id != dim.id);
                (dim.id, "dimensions", before, self.dimensions.len())
            }
            Entity::DimensionType(dim_type) => {
                let before = self.dimension_types.len();
                self.dimension_types.retain(|d| d.id != dim_type.id);
                (dim_type.id, "dimension_types", before, self.dimension_types.len())
            }
            Entity::Structure(structure) => {
                let before = self.structures.len();
                self.structures.retain(|s| s.id != structure.id);
                (structure.id, "structures", before, self.structures.len())
            }
            Entity::Feature(feature) => {
                let before = self.features.len();
                self.features.retain(|f| f.id != feature.id);
                (feature.id, "features", before, self.features.len())
            }
            Entity::LootTable(loot) => {
                let before = self.loot_tables.len();
                self.loot_tables.retain(|l| l.id != loot.id);
                (loot.id, "loot_tables", before, self.loot_tables.len())
            }
            Entity::Advancement(adv) => {
                let before = self.advancements.len();
                self.advancements.retain(|a| a.id != adv.id);
                (adv.id, "advancements", before, self.advancements.len())
            }
            Entity::SoundEvent(se) => {
                let before = self.sound_events.len();
                self.sound_events.retain(|s| s.id != se.id);
                (se.id, "sound_events", before, self.sound_events.len())
            }
            Entity::Tag(tag) => {
                let before = self.tags.len();
                self.tags.retain(|t| t.id != tag.id);
                (tag.id, "tags", before, self.tags.len())
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
/// Returns an error if the file cannot be read or cannot be deserialized.
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
    /// A mob entity.
    Mob(Mob),
    /// A biome entity.
    Biome(Biome),
    /// A dimension entity.
    Dimension(Dimension),
    /// A dimension type entity.
    DimensionType(DimensionType),
    /// A structure entity.
    Structure(Structure),
    /// A feature entity.
    Feature(Feature),
    /// A loot table entity.
    LootTable(LootTable),
    /// An advancement entity.
    Advancement(Advancement),
    /// A sound event entity.
    SoundEvent(SoundEvent),
    /// A tag entity.
    Tag(Tag),
}

impl Entity {
    fn id(&self) -> &str {
        match self {
            Entity::Item(i) => &i.id,
            Entity::Block(b) => &b.id,
            Entity::Recipe(r) => &r.id,
            Entity::CreativeTab(t) => &t.id,
            Entity::Mob(m) => &m.id,
            Entity::Biome(b) => &b.id,
            Entity::Dimension(d) => &d.id,
            Entity::DimensionType(dt) => &dt.id,
            Entity::Structure(s) => &s.id,
            Entity::Feature(f) => &f.id,
            Entity::LootTable(l) => &l.id,
            Entity::Advancement(a) => &a.id,
            Entity::SoundEvent(s) => &s.id,
            Entity::Tag(t) => &t.id,
        }
    }

    fn kind(&self) -> &'static str {
        match self {
            Entity::Item(_) => "Item",
            Entity::Block(_) => "Block",
            Entity::Recipe(_) => "Recipe",
            Entity::CreativeTab(_) => "CreativeTab",
            Entity::Mob(_) => "Mob",
            Entity::Biome(_) => "Biome",
            Entity::Dimension(_) => "Dimension",
            Entity::DimensionType(_) => "DimensionType",
            Entity::Structure(_) => "Structure",
            Entity::Feature(_) => "Feature",
            Entity::LootTable(_) => "LootTable",
            Entity::Advancement(_) => "Advancement",
            Entity::SoundEvent(_) => "SoundEvent",
            Entity::Tag(_) => "Tag",
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
            model_kind: BlockModelKind::CubeAll,
            properties_from: None,
            block_class: "Block".into(),
        })
    }

    /// Creates a [`Block`] with a default creative tab assignment.
    pub fn base(id: &str, tab: &str) -> Self {
        Self {
            id: id.into(),
            creative_tab: Some(tab.into()),
            model_kind: BlockModelKind::CubeAll,
            properties_from: None,
            block_class: "Block".into(),
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
            cooking_time: None,
            experience: None,
            category: None,
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

    /// Mobs tracked in state.
    #[serde(default)]
    pub mobs: Vec<Mob>,

    /// Biomes tracked in state.
    #[serde(default)]
    pub biomes: Vec<Biome>,

    /// Dimensions tracked in state.
    #[serde(default)]
    pub dimensions: Vec<Dimension>,

    /// Structures tracked in state.
    #[serde(default)]
    pub structures: Vec<Structure>,

    /// Features tracked in state.
    #[serde(default)]
    pub features: Vec<Feature>,

    /// Loot tables tracked in state.
    #[serde(default)]
    pub loot_tables: Vec<LootTable>,

    /// Advancements tracked in state.
    #[serde(default)]
    pub advancements: Vec<Advancement>,

    /// Sound events tracked in state.
    #[serde(default)]
    pub sound_events: Vec<SoundEvent>,

    /// Dimension types tracked in state.
    #[serde(default)]
    pub dimension_types: Vec<DimensionType>,

    /// Tags tracked in state.
    #[serde(default)]
    pub tags: Vec<Tag>,
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

    /// A music disc item.
    MusicDisc,

    /// A fire charge item.
    FireCharge,

    /// A flint and steel item.
    FlintAndSteel,

    /// A compass item.
    Compass,
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

/// Block model kind for datagen.
#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq)]
pub enum BlockModelKind {
    /// Full cube block using one texture on all faces.
    #[default]
    CubeAll,

    /// Cube with distinct bottom and top textures.
    CubeBottomTop,

    /// Cube with no top/bottom distinction.
    Cube,

    /// Column-like block.
    CubeColumn,

    /// Orientable block (e.g. furnace, dispenser).
    Orientable,

    /// Slab block.
    Slab,

    /// Stairs block.
    Stairs,

    /// Wall block.
    Wall,

    /// Fence block.
    Fence,

    /// Fence gate block.
    FenceGate,

    /// Door block.
    Door,

    /// Trapdoor block.
    Trapdoor,

    /// Button block.
    Button,

    /// Pressure plate block.
    PressurePlate,

    /// Cross-shaped block (e.g. flowers).
    Cross,

    /// Crop block.
    Crop,

    /// Tinted cross-shaped block.
    TintedCross,

    /// Single-face block (e.g. torches).
    SingleFace,

    /// Ore block.
    Ore,

    /// Lantern block.
    Lantern,

    /// Chain block.
    Chain,

    /// Carpet block.
    Carpet,

    /// Ladder block.
    Ladder,

    /// Lever block.
    Lever,
}

/// A block tracked in [`ModState::blocks`].
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Block {
    /// Lowercase block identifier.
    pub id: String,

    /// Creative tab assignment (`None` means default to first custom tab or `ingredients`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub creative_tab: Option<String>,

    /// Model kind for datagen.
    #[serde(default)]
    pub model_kind: BlockModelKind,

    /// Vanilla block to copy properties from (`None` uses `minecraft:dirt`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub properties_from: Option<String>,

    /// Java block class name for registration (e.g. `"Block"`, `"ChainBlock"`, `"LanternBlock"`).
    #[serde(default = "default_block_class")]
    pub block_class: String,
}

fn default_block_class() -> String {
    "Block".into()
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

    /// Cooking time in ticks (for cooking recipes).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cooking_time: Option<i32>,

    /// Experience points (for cooking recipes).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub experience: Option<f32>,

    /// Recipe category (e.g. `"building_blocks"`, `"combat"`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,
}

/// A mob/entity definition tracked in [`ModState::mobs`].
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Mob {
    /// Lowercase mob identifier.
    pub id: String,

    /// Entity type constant (e.g. `"minecraft:phantom"` or `"mymod:blue_ghost"`).
    pub entity_type: String,

    /// Spawn category (e.g. `"monster"`, `"creature"`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub spawn_category: Option<String>,

    /// Spawn egg item id (`None` means no egg is generated).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub spawn_egg_id: Option<String>,

    /// Baby spawn egg item id (`None` defaults to same egg).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub baby_spawn_egg_id: Option<String>,

    /// Entity attribute modifiers.
    #[serde(default)]
    pub attributes: Vec<Attribute>,

    /// Mob drops.
    #[serde(default)]
    pub drops: Vec<Drop>,
}

/// An entity attribute modifier.
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Attribute {
    /// Attribute name (e.g. `"minecraft:generic.max_health"`).
    pub attribute: String,

    /// Base value.
    pub base: f64,

    /// Optional multiplier.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub multiplier: Option<f64>,
}

/// A mob drop entry.
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Drop {
    /// Item id to drop.
    pub item: String,

    /// Minimum count.
    pub min_count: u32,

    /// Maximum count.
    pub max_count: u32,

    /// Loot table source (e.g. `"entity"`, `"block"`, `"chest"`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub loot_type: Option<String>,
}

/// A biome definition tracked in [`ModState::biomes`].
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Biome {
    /// Lowercase biome identifier.
    pub id: String,

    /// Biome temperature.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,

    /// Biome downfall.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub downfall: Option<f32>,

    /// Sky color as integer RGB.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sky_color: Option<i32>,

    /// Water color as integer RGB.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub water_color: Option<i32>,

    /// Water fog color as integer RGB.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub water_fog_color: Option<i32>,

    /// Fog color as integer RGB.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fog_color: Option<i32>,

    /// Whether the biome has precipitation.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub has_precipitation: Option<bool>,

    /// Feature references (configured features / placed features).
    #[serde(default)]
    pub features: Vec<String>,

    /// Structure references.
    #[serde(default)]
    pub structures: Vec<String>,

    /// Mob spawn rules.
    #[serde(default)]
    pub mob_spawns: Vec<MobSpawn>,

    /// Whether players can spawn here naturally.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub player_spawn_friendly: Option<bool>,
}

/// A mob spawn rule inside a biome.
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct MobSpawn {
    /// Entity type id.
    pub entity_type: String,

    /// Spawn category (e.g. `"monster"`, `"creature"`, `"ambient"`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,

    /// Spawn weight.
    pub weight: i32,

    /// Minimum group size.
    pub min_count: i32,

    /// Maximum group size.
    pub max_count: i32,
}

/// A dimension definition tracked in [`ModState::dimensions`].
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Dimension {
    /// Lowercase dimension identifier.
    pub id: String,

    /// Dimension type key (e.g. `"minecraft:overworld"` or custom type id).
    pub dimension_type: String,

    /// Portal frame block id.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub portal_frame: Option<String>,

    /// Portal igniter item id.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub portal_igniter: Option<String>,
}

/// A dimension type definition tracked in [`ModState::dimension_types`].
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct DimensionType {
    /// Lowercase dimension type identifier.
    pub id: String,

    /// Whether the dimension is ultrawarm like the Nether.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ultrawarm: Option<bool>,

    /// Whether the dimension is natural.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub natural: Option<bool>,

    /// Whether piglins are safe in this dimension.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub piglin_safe: Option<bool>,

    /// Whether respawn anchors work.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub respawn_anchor_works: Option<bool>,

    /// Whether beds work.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bed_works: Option<bool>,

    /// Whether raids can happen.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub has_raids: Option<bool>,

    /// Whether the dimension has skylight.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub has_skylight: Option<bool>,

    /// Whether the dimension has a ceiling.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub has_ceiling: Option<bool>,

    /// Coordinate scale.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub coordinate_scale: Option<f64>,

    /// Logical height.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub logical_height: Option<i32>,

    /// Minimum Y coordinate.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub min_y: Option<i32>,

    /// Height of the dimension.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub height: Option<i32>,

    /// Monster spawn light level.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub monster_spawn_light_level: Option<i32>,

    /// Monster spawn block light limit.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub monster_spawn_block_light_limit: Option<i32>,

    /// Ambient light level.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ambient_light: Option<f32>,
}

/// A structure definition tracked in [`ModState::structures`].
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Structure {
    /// Lowercase structure identifier.
    pub id: String,

    /// Structure set / type key.
    pub structure_type: String,

    /// Biome id for spawning.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub spawn: Option<String>,

    /// Spacing (chunks).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub spacing: Option<i32>,

    /// Separation (chunks).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub separation: Option<i32>,

    /// Structure salt.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub salt: Option<i32>,
}

/// A terrain feature definition tracked in [`ModState::features`].
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Feature {
    /// Lowercase feature identifier.
    pub id: String,

    /// Configured feature key.
    pub feature_type: String,

    /// Block id for feature.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub block: Option<String>,

    /// Optional state override.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub state: Option<String>,

    /// Optional radius.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub radius: Option<i32>,
}

/// A loot table definition tracked in [`ModState::loot_tables`].
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct LootTable {
    /// Lowercase loot table identifier.
    pub id: String,

    /// Loot table type (e.g. `"entity"`, `"block"`, `"chest"`).
    pub loot_type: String,

    /// Roll count.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rolls: Option<u32>,

    /// Loot entries.
    #[serde(default)]
    pub entries: Vec<LootEntry>,
}

/// A loot table entry.
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct LootEntry {
    /// Item id.
    pub item: String,

    /// Minimum count.
    pub min_count: u32,

    /// Maximum count.
    pub max_count: u32,

    /// Entry weight.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub weight: Option<f32>,

    /// Optional condition key/value.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub condition: Option<String>,
}

/// An advancement definition tracked in [`ModState::advancements`].
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Advancement {
    /// Lowercase advancement identifier.
    pub id: String,

    /// Parent advancement id.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent: Option<String>,

    /// Display title.
    pub title: String,

    /// Display description.
    pub description: String,

    /// Icon item id.
    pub icon: String,

    /// Background texture path.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub background: Option<String>,

    /// Frame type (`"task"`, `"goal"`, `"challenge"`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub frame: Option<String>,

    /// Show toast.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub show_toast: Option<bool>,

    /// Announce to chat.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub announce_to_chat: Option<bool>,

    /// Hidden.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hidden: Option<bool>,

    /// Trigger criteria.
    #[serde(default)]
    pub criteria: Vec<AdvancementCriterion>,
}

/// An advancement criterion.
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct AdvancementCriterion {
    /// Criterion id.
    pub id: String,

    /// Trigger type.
    pub trigger: String,

    /// Condition key/value pairs.
    #[serde(default)]
    pub conditions: Vec<(String, String)>,
}

/// A sound event definition tracked in [`ModState::sound_events`].
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct SoundEvent {
    /// Lowercase sound event identifier.
    pub id: String,

    /// Sound path relative to sounds directory.
    pub sound_path: String,
}

/// A tag definition tracked in [`ModState::tags`].
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Tag {
    /// Lowercase tag identifier.
    pub id: String,

    /// Tag type (`"block"`, `"item"`, `"entity"`, `"fluid"`).
    pub tag_type: String,

    /// Tag values (ids).
    #[serde(default)]
    pub values: Vec<String>,

    /// Replace existing tag.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub replace: Option<bool>,
}
