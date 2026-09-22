use crate::java_writer::{DirtyFlags, regenerate_all};
use crate::state::{self, Block, CreativeTab, Entity, Item, ItemKind, PotionEffect, Recipe};
use anyhow::{Context, Result};
use clap::Parser;

const TAB: &str = "testmod";

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

fn try_add_item(state: &mut state::ModState, item: Item) -> Result<bool> {
    let label = format!("{} ({})", item.id, kind_label(&item));
    let added = state.try_add(Entity::Item(item))?;
    if added {
        println!("Added item: {}", label);
    }
    Ok(added)
}

/// Adds a complete set of example entities to the current project for testing.
pub fn run(args: TestProjectArgs) -> Result<()> {
    let TestProjectArgs { reset } = args;

    let mut state = state::load().context("Failed to load project state")?;

    if reset {
        println!("Resetting test project state...");
        state.items.clear();
        state.blocks.clear();
        state.recipes.clear();
        state.creative_tabs.clear();
        state.save().context("Failed to save reset state")?;
        regenerate_all(&state, DirtyFlags::all(), false)
            .context("Failed to regenerate after reset")?;
    }

    println!("Adding test project items...");

    let dirty = DirtyFlags::all();

    // Creative tab
    {
        let tab = CreativeTab { id: TAB.into() };
        let entity = Entity::CreativeTab(tab);
        if state.try_add(entity)? {
            println!("Added creative tab: {}", TAB);
        }
    }

    // Basic item
    try_add_item(&mut state, Item::base("testmod_ingot", TAB))?;

    // Tool item
    {
        let mut item = Item::base("testmod_sword", TAB);
        item.kind = ItemKind::Tool;
        item.material = Some("iron".into());
        item.attack_damage = Some(6.0);
        item.attack_speed = Some(1.6);
        try_add_item(&mut state, item)?;
    }

    // Axe item
    {
        let mut item = Item::base("testmod_axe", TAB);
        item.kind = ItemKind::Axe;
        item.material = Some("iron".into());
        item.attack_damage = Some(7.0);
        item.attack_speed = Some(1.0);
        try_add_item(&mut state, item)?;
    }

    // Shovel item
    {
        let mut item = Item::base("testmod_shovel", TAB);
        item.kind = ItemKind::Shovel;
        item.material = Some("iron".into());
        item.attack_damage = Some(4.5);
        item.attack_speed = Some(1.0);
        try_add_item(&mut state, item)?;
    }

    // Hoe item
    {
        let mut item = Item::base("testmod_hoe", TAB);
        item.kind = ItemKind::Hoe;
        item.material = Some("iron".into());
        item.attack_damage = Some(1.0);
        item.attack_speed = Some(4.0);
        try_add_item(&mut state, item)?;
    }

    // Food item with tooltips
    {
        let mut item = Item::base("glowing_fruit", TAB);
        item.kind = ItemKind::Food;
        item.tooltip = vec![
            "A mysterious glowing fruit.".into(),
            "Grants temporary night vision.".into(),
        ];
        item.nutrition = Some(6);
        item.saturation = Some(0.4);
        item.always_edible = true;
        item.effects = vec![PotionEffect {
            effect_type: "minecraft:night_vision".into(),
            duration: 1200,
            amplifier: 0,
        }];
        try_add_item(&mut state, item)?;
    }

    // Spawn egg
    {
        let mut item = Item::base("phantom_spawn_egg", TAB);
        item.kind = ItemKind::SpawnEgg;
        item.entity_type = Some("minecraft:phantom".into());
        try_add_item(&mut state, item)?;
    }

    // Fuel item
    {
        let mut item = Item::base("testmod_fuel", TAB);
        item.kind = ItemKind::Fuel;
        item.burn_time = Some(3200);
        try_add_item(&mut state, item)?;
    }

    // Compostable item
    {
        let mut item = Item::base("plant_fiber", TAB);
        item.kind = ItemKind::Compostable;
        item.compost_chance = Some(0.6);
        try_add_item(&mut state, item)?;
    }

    // Armor item
    {
        let mut item = Item::base("testmod_helmet", TAB);
        item.kind = ItemKind::Armor;
        item.armor_material = Some("iron".into());
        item.armor_slot = Some("helmet".into());
        try_add_item(&mut state, item)?;
    }

    // Shield item
    {
        let mut item = Item::base("testmod_shield", TAB);
        item.kind = ItemKind::Shield;
        try_add_item(&mut state, item)?;
    }

    // Potion item
    {
        let mut item = Item::base("speed_potion", TAB);
        item.kind = ItemKind::Potion;
        item.effects = vec![PotionEffect {
            effect_type: "minecraft:speed".into(),
            duration: 3600,
            amplifier: 0,
        }];
        try_add_item(&mut state, item)?;
    }

    // Block
    {
        let block = Block::base("testmod_ore", TAB);
        if state.try_add(Entity::Block(block))? {
            println!("Added block: testmod_ore");
        }
    }

    // Shaped recipe
    {
        let mut recipe = Recipe::new("testmod_ingot_from_ore")?;
        recipe.kind = "crafting_shaped".into();
        recipe.pattern = vec!["CCC".into(), "S S".into(), "S S".into()];
        recipe
            .ingredients
            .insert('C'.into(), "testmod:testmod_ore".into());
        recipe
            .ingredients
            .insert('S'.into(), "minecraft:stick".into());
        recipe.result = "testmod:testmod_ingot".into();
        recipe.count = 4;
        if state.try_add(Entity::Recipe(recipe))? {
            println!("Added recipe: testmod_ingot_from_ore (crafting_shaped)");
        }
    }

    // Shapeless recipe
    {
        let mut recipe = Recipe::new("testmod_pie")?;
        recipe.kind = "crafting_shapeless".into();
        recipe
            .ingredients
            .insert('F'.into(), "testmod:glowing_fruit".into());
        recipe
            .ingredients
            .insert('W'.into(), "minecraft:wheat".into());
        recipe.result = "minecraft:apple".into();
        recipe.count = 1;
        if state.try_add(Entity::Recipe(recipe))? {
            println!("Added recipe: testmod_pie (crafting_shapeless)");
        }
    }

    state.save().context("Failed to save test project state")?;
    regenerate_all(&state, dirty, false).context("Failed to regenerate Java sources")?;

    println!("Test project added successfully!");
    println!("Run 'fw status -v' to see all added entities.");

    Ok(())
}

/// CLI arguments for `fw test-project`.
#[derive(Parser)]
pub struct TestProjectArgs {
    /// Reset existing project state before adding test entities
    #[arg(long)]
    pub reset: bool,
}
