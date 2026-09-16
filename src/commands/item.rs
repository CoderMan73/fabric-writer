use crate::java_writer::{DirtyFlags, regenerate_all};
use crate::state::{self, Entity, Item, ItemKind};
use anyhow::Result;
use clap::Parser;

/// Adds an item to the mod state and regenerates Java sources.
pub fn add(args: ItemAddArgs) -> Result<()> {
    let verbose = args.verbose;
    let mut state = state::load()?;
    let item = build_item(&args)?;
    let entity = Entity::Item(item.clone());
    let item_label = format!("{} ({})", item.id, kind_label(&item));
    let dirty = DirtyFlags::from_entity(&entity);
    state.add(entity)?;
    state.save()?;
    regenerate_all(&state, dirty, verbose)?;
    println!("Added item: {}", item_label);
    Ok(())
}

/// Removes an item from the mod state and regenerates Java sources.
pub fn remove(args: ItemRemoveArgs) -> Result<()> {
    let verbose = args.verbose;
    let mut state = state::load()?;
    let entity = Entity::Item(Item::new(&args.id)?);
    let dirty = DirtyFlags::from_entity(&entity);
    state.remove(entity)?;
    state.save()?;
    regenerate_all(&state, dirty, verbose)?;
    println!("Removed item: {}", args.id);
    Ok(())
}

fn build_item(args: &ItemAddArgs) -> Result<Item> {
    let mut item = Item::new(&args.id)?;
    if let Some(kind) = &args.kind {
        item.kind = match kind.to_lowercase().as_str() {
            "tool" => ItemKind::Tool,
            "axe" => ItemKind::Axe,
            "shovel" => ItemKind::Shovel,
            "hoe" => ItemKind::Hoe,
            "food" => ItemKind::Food,
            "spawn_egg" => ItemKind::SpawnEgg,
            "fuel" => ItemKind::Fuel,
            "compostable" => ItemKind::Compostable,
            "armor" => ItemKind::Armor,
            "shield" => ItemKind::Shield,
            "potion" => ItemKind::Potion,
            _ => ItemKind::Basic,
        };
    }
    item.material = args.material.clone();
    item.attack_damage = args.attack_damage;
    item.attack_speed = args.attack_speed;
    item.durability = args.durability;
    item.tooltip = args.tooltip.clone();
    item.nutrition = args.nutrition;
    item.saturation = args.saturation;
    item.always_edible = args.always_edible;
    item.entity_type = args.entity_type.clone();
    item.burn_time = args.burn_time;
    item.compost_chance = args.compost_chance;
    item.creative_tab = args.creative_tab.clone();
    item.armor_material = args.armor_material.clone();
    item.armor_slot = args.armor_slot.clone();
    for effect_str in &args.effect {
        let effect = parse_effect(effect_str)?;
        item.effects.push(effect);
    }
    Ok(item)
}

fn parse_effect(effect_str: &str) -> Result<crate::state::PotionEffect> {
    let Some((effect_type, rest)) = effect_str.split_once('=') else {
        anyhow::bail!(
            "Invalid effect format: '{}'. Expected <type>=<duration>:<amplifier>",
            effect_str
        );
    };
    let Some((duration_str, amplifier_str)) = rest.split_once(':') else {
        anyhow::bail!(
            "Invalid effect format: '{}'. Expected <type>=<duration>:<amplifier>",
            effect_str
        );
    };
    let duration = duration_str.parse::<i32>()?;
    let amplifier = amplifier_str.parse::<i32>()?;
    Ok(crate::state::PotionEffect {
        effect_type: effect_type.to_string(),
        duration,
        amplifier,
    })
}

fn kind_label(item: &Item) -> &'static str {
    match item.kind {
        ItemKind::Basic => "basic",
        ItemKind::Tool => "tool",
        ItemKind::Axe => "axe",
        ItemKind::Shovel => "shovel",
        ItemKind::Hoe => "hoe",
        ItemKind::Food => "food",
        ItemKind::SpawnEgg => "spawn_egg",
        ItemKind::Fuel => "fuel",
        ItemKind::Compostable => "compostable",
        ItemKind::Armor => "armor",
        ItemKind::Shield => "shield",
        ItemKind::Potion => "potion",
    }
}

/// CLI arguments for `fw add item`.
#[derive(Parser)]
pub struct ItemAddArgs {
    /// Item ID
    #[arg(short = 'i', long)]
    pub id: String,

    /// Item kind: `tool`, `axe`, `shovel`, `hoe`, `food`, `spawn_egg`, or `basic` (defaults to `basic`).
    #[arg(long)]
    pub kind: Option<String>,

    /// Tool material (e.g. `diamond`); required for tool/axe/shovel/hoe.
    #[arg(long)]
    pub material: Option<String>,

    /// Attack damage for tool items.
    #[arg(long)]
    pub attack_damage: Option<f32>,

    /// Attack speed modifier for tool items.
    #[arg(long)]
    pub attack_speed: Option<f32>,

    /// Durability for tool items (overrides material default).
    #[arg(long)]
    pub durability: Option<i32>,

    /// Nutrition value for food items.
    #[arg(long)]
    pub nutrition: Option<i32>,

    /// Saturation modifier for food items.
    #[arg(long)]
    pub saturation: Option<f32>,

    /// Allow eating regardless of hunger level.
    #[arg(long, default_value_t = false)]
    pub always_edible: bool,

    /// Entity type for spawn egg items (e.g. `minecraft:cow`).
    #[arg(long)]
    pub entity_type: Option<String>,

    /// Burn time in ticks for fuel items.
    #[arg(long)]
    pub burn_time: Option<i32>,

    /// Compost chance for compostable items (0.0 to 1.0).
    #[arg(long)]
    pub compost_chance: Option<f32>,

    /// Tooltip line for the item (repeatable).
    #[arg(long)]
    pub tooltip: Vec<String>,

    /// Creative tab for the item (defaults to default behavior: first custom tab or ingredients).
    #[arg(long)]
    pub creative_tab: Option<String>,

    /// Armor material (e.g. `diamond`, `iron`); required for armor items.
    #[arg(long)]
    pub armor_material: Option<String>,

    /// Armor slot (helmet, chestplate, leggings, boots); required for armor items.
    #[arg(long)]
    pub armor_slot: Option<String>,

    /// Potion effect in `type=duration:amplifier` format (repeatable).
    #[arg(long)]
    pub effect: Vec<String>,

    /// Show which files were regenerated, skipped, or pruned
    #[arg(short = 'v', long, default_value_t = false)]
    pub verbose: bool,
}

/// CLI arguments for `fw remove item`.
#[derive(Parser)]
pub struct ItemRemoveArgs {
    /// Item ID to remove.
    #[arg(short = 'i', long)]
    pub id: String,

    /// Show which files were regenerated, skipped, or pruned
    #[arg(short = 'v', long, default_value_t = false)]
    pub verbose: bool,
}
