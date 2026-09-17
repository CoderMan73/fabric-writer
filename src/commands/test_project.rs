use crate::commands::{
    block::{self, BlockAddArgs},
    item::{self, ItemAddArgs},
    recipe::{self, RecipeAddArgs},
};
use anyhow::Result;

/// Adds a complete set of example entities to the current project for testing.
pub fn run() -> Result<()> {
    println!("Adding test project items...");

    item::add(ItemAddArgs {
        id: "copper_ingot".into(),
        kind: None,
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
        creative_tab: None,
        armor_material: None,
        armor_slot: None,
        effect: vec![],
        verbose: false,
    })?;

    item::add(ItemAddArgs {
        id: "copper_sword".into(),
        kind: Some("tool".into()),
        material: Some("copper".into()),
        attack_damage: Some(5.0),
        attack_speed: Some(1.6),
        durability: None,
        nutrition: None,
        saturation: None,
        always_edible: false,
        entity_type: None,
        burn_time: None,
        compost_chance: None,
        tooltip: vec![],
        creative_tab: None,
        armor_material: None,
        armor_slot: None,
        effect: vec![],
        verbose: false,
    })?;

    item::add(ItemAddArgs {
        id: "glow_berry".into(),
        kind: Some("food".into()),
        material: None,
        attack_damage: None,
        attack_speed: None,
        durability: None,
        nutrition: Some(4),
        saturation: Some(0.3),
        always_edible: true,
        entity_type: None,
        burn_time: None,
        compost_chance: None,
        tooltip: vec!["A glowing berry.".into(), "Consumes on use.".into()],
        creative_tab: None,
        armor_material: None,
        armor_slot: None,
        effect: vec![],
        verbose: false,
    })?;

    item::add(ItemAddArgs {
        id: "cow_spawn_egg".into(),
        kind: Some("spawn_egg".into()),
        material: None,
        attack_damage: None,
        attack_speed: None,
        durability: None,
        nutrition: None,
        saturation: None,
        always_edible: false,
        entity_type: Some("minecraft:cow".into()),
        burn_time: None,
        compost_chance: None,
        tooltip: vec![],
        creative_tab: None,
        armor_material: None,
        armor_slot: None,
        effect: vec![],
        verbose: false,
    })?;

    item::add(ItemAddArgs {
        id: "dried_kelp".into(),
        kind: Some("food".into()),
        material: None,
        attack_damage: None,
        attack_speed: None,
        durability: None,
        nutrition: Some(4),
        saturation: Some(0.3),
        always_edible: false,
        entity_type: None,
        burn_time: Some(1600),
        compost_chance: None,
        tooltip: vec![],
        creative_tab: None,
        armor_material: None,
        armor_slot: None,
        effect: vec![],
        verbose: false,
    })?;

    item::add(ItemAddArgs {
        id: "sugarcane".into(),
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
        compost_chance: Some(0.5),
        tooltip: vec![],
        creative_tab: None,
        armor_material: None,
        armor_slot: None,
        effect: vec![],
        verbose: false,
    })?;

    block::add(BlockAddArgs {
        id: "copper_ore".into(),
        creative_tab: None,
        verbose: false,
    })?;

    recipe::add(RecipeAddArgs {
        id: "copper_ingot_from_ore".into(),
        kind: Some("crafting_shaped".into()),
        result: Some("testmod:copper_ingot".into()),
        count: Some(4),
        pattern: vec!["CCC".into(), "S S".into(), "S S".into()],
        ingredients: vec!["C=testmod:copper_ore".into(), "S=minecraft:stick".into()],
        verbose: false,
    })?;

    recipe::add(RecipeAddArgs {
        id: "berry_pie".into(),
        kind: Some("crafting_shapeless".into()),
        result: Some("minecraft:apple".into()),
        count: Some(1),
        pattern: vec![],
        ingredients: vec!["B=testmod:glow_berry".into(), "W=minecraft:wheat".into()],
        verbose: false,
    })?;

    println!("Test project added successfully!");
    println!("Run 'fw status -v' to see all added entities.");

    Ok(())
}
