use crate::commands::{
    block::{self, BlockAddArgs},
    creative_tab::{self, CreativeTabAddArgs},
    item::{self, ItemAddArgs},
    recipe::{self, RecipeAddArgs},
};
use anyhow::Result;

/// Adds a complete set of example entities to the current project for testing.
pub fn run() -> Result<()> {
    println!("Adding test project items...");

    // Creative tab
    creative_tab::add(CreativeTabAddArgs {
        id: "testmod".into(),
        verbose: false,
    })?;

    // Basic item
    item::add(ItemAddArgs {
        id: "testmod_ingot".into(),
        kind: Some("basic".into()),
        material: None,
        attack_damage: None,
        attack_speed: None,
        durability: None,
        nutrition: None,
        saturation: None,
        always_edible: false,
        entity_type: None,
        burn_time: None,
        compost_chance: None,
        tooltip: vec![],
        creative_tab: Some("testmod".into()),
        armor_material: None,
        armor_slot: None,
        effect: vec![],
        verbose: false,
    })?;

    // Tool item
    item::add(ItemAddArgs {
        id: "testmod_sword".into(),
        kind: Some("tool".into()),
        material: Some("iron".into()),
        attack_damage: Some(6.0),
        attack_speed: Some(1.6),
        durability: None,
        nutrition: None,
        saturation: None,
        always_edible: false,
        entity_type: None,
        burn_time: None,
        compost_chance: None,
        tooltip: vec![],
        creative_tab: Some("testmod".into()),
        armor_material: None,
        armor_slot: None,
        effect: vec![],
        verbose: false,
    })?;

    // Axe item
    item::add(ItemAddArgs {
        id: "testmod_axe".into(),
        kind: Some("axe".into()),
        material: Some("iron".into()),
        attack_damage: Some(7.0),
        attack_speed: Some(1.0),
        durability: None,
        nutrition: None,
        saturation: None,
        always_edible: false,
        entity_type: None,
        burn_time: None,
        compost_chance: None,
        tooltip: vec![],
        creative_tab: Some("testmod".into()),
        armor_material: None,
        armor_slot: None,
        effect: vec![],
        verbose: false,
    })?;

    // Shovel item
    item::add(ItemAddArgs {
        id: "testmod_shovel".into(),
        kind: Some("shovel".into()),
        material: Some("iron".into()),
        attack_damage: Some(4.5),
        attack_speed: Some(1.0),
        durability: None,
        nutrition: None,
        saturation: None,
        always_edible: false,
        entity_type: None,
        burn_time: None,
        compost_chance: None,
        tooltip: vec![],
        creative_tab: Some("testmod".into()),
        armor_material: None,
        armor_slot: None,
        effect: vec![],
        verbose: false,
    })?;

    // Hoe item
    item::add(ItemAddArgs {
        id: "testmod_hoe".into(),
        kind: Some("hoe".into()),
        material: Some("iron".into()),
        attack_damage: Some(1.0),
        attack_speed: Some(4.0),
        durability: None,
        nutrition: None,
        saturation: None,
        always_edible: false,
        entity_type: None,
        burn_time: None,
        compost_chance: None,
        tooltip: vec![],
        creative_tab: Some("testmod".into()),
        armor_material: None,
        armor_slot: None,
        effect: vec![],
        verbose: false,
    })?;

    // Food item with tooltips
    item::add(ItemAddArgs {
        id: "glowing_fruit".into(),
        kind: Some("food".into()),
        material: None,
        attack_damage: None,
        attack_speed: None,
        durability: None,
        nutrition: Some(6),
        saturation: Some(0.4),
        always_edible: true,
        entity_type: None,
        burn_time: None,
        compost_chance: None,
        tooltip: vec![
            "A mysterious glowing fruit.".into(),
            "Grants temporary night vision.".into(),
        ],
        creative_tab: Some("testmod".into()),
        armor_material: None,
        armor_slot: None,
        effect: vec![],
        verbose: false,
    })?;

    // Spawn egg
    item::add(ItemAddArgs {
        id: "phantom_spawn_egg".into(),
        kind: Some("spawn_egg".into()),
        material: None,
        attack_damage: None,
        attack_speed: None,
        durability: None,
        nutrition: None,
        saturation: None,
        always_edible: false,
        entity_type: Some("minecraft:phantom".into()),
        burn_time: None,
        compost_chance: None,
        tooltip: vec![],
        creative_tab: Some("testmod".into()),
        armor_material: None,
        armor_slot: None,
        effect: vec![],
        verbose: false,
    })?;

    // Fuel item
    item::add(ItemAddArgs {
        id: "testmod_fuel".into(),
        kind: Some("fuel".into()),
        material: None,
        attack_damage: None,
        attack_speed: None,
        durability: None,
        nutrition: None,
        saturation: None,
        always_edible: false,
        entity_type: None,
        burn_time: Some(3200),
        compost_chance: None,
        tooltip: vec![],
        creative_tab: Some("testmod".into()),
        armor_material: None,
        armor_slot: None,
        effect: vec![],
        verbose: false,
    })?;

    // Compostable item
    item::add(ItemAddArgs {
        id: "plant_fiber".into(),
        kind: Some("compostable".into()),
        material: None,
        attack_damage: None,
        attack_speed: None,
        durability: None,
        nutrition: None,
        saturation: None,
        always_edible: false,
        entity_type: None,
        burn_time: None,
        compost_chance: Some(0.6),
        tooltip: vec![],
        creative_tab: Some("testmod".into()),
        armor_material: None,
        armor_slot: None,
        effect: vec![],
        verbose: false,
    })?;

    // Armor item
    item::add(ItemAddArgs {
        id: "testmod_helmet".into(),
        kind: Some("armor".into()),
        material: None,
        attack_damage: None,
        attack_speed: None,
        durability: None,
        nutrition: None,
        saturation: None,
        always_edible: false,
        entity_type: None,
        burn_time: None,
        compost_chance: None,
        tooltip: vec![],
        creative_tab: Some("testmod".into()),
        armor_material: Some("iron".into()),
        armor_slot: Some("helmet".into()),
        effect: vec![],
        verbose: false,
    })?;

    // Shield item
    item::add(ItemAddArgs {
        id: "testmod_shield".into(),
        kind: Some("shield".into()),
        material: None,
        attack_damage: None,
        attack_speed: None,
        durability: None,
        nutrition: None,
        saturation: None,
        always_edible: false,
        entity_type: None,
        burn_time: None,
        compost_chance: None,
        tooltip: vec![],
        creative_tab: Some("testmod".into()),
        armor_material: None,
        armor_slot: None,
        effect: vec![],
        verbose: false,
    })?;

    // Potion item
    item::add(ItemAddArgs {
        id: "testmod_potion".into(),
        kind: Some("potion".into()),
        material: None,
        attack_damage: None,
        attack_speed: None,
        durability: None,
        nutrition: None,
        saturation: None,
        always_edible: false,
        entity_type: None,
        burn_time: None,
        compost_chance: None,
        tooltip: vec![],
        creative_tab: Some("testmod".into()),
        armor_material: None,
        armor_slot: None,
        effect: vec![
            "minecraft:speed=1200:1".into(),
            "minecraft:strength=600:0".into(),
        ],
        verbose: false,
    })?;

    // Block
    block::add(BlockAddArgs {
        id: "testmod_ore".into(),
        creative_tab: Some("testmod".into()),
        verbose: false,
    })?;

    // Shaped recipe
    recipe::add(RecipeAddArgs {
        id: "testmod_ingot_from_ore".into(),
        kind: Some("crafting_shaped".into()),
        result: Some("testmod:testmod_ingot".into()),
        count: Some(4),
        pattern: vec!["CCC".into(), "S S".into(), "S S".into()],
        ingredients: vec!["C=testmod:testmod_ore".into(), "S=minecraft:stick".into()],
        verbose: false,
    })?;

    // Shapeless recipe
    recipe::add(RecipeAddArgs {
        id: "testmod_pie".into(),
        kind: Some("crafting_shapeless".into()),
        result: Some("minecraft:apple".into()),
        count: Some(1),
        pattern: vec![],
        ingredients: vec!["F=testmod:glowing_fruit".into(), "W=minecraft:wheat".into()],
        verbose: false,
    })?;

    println!("Test project added successfully!");
    println!("Run 'fw status -v' to see all added entities.");

    Ok(())
}
