use crate::java_writer::{DirtyFlags, regenerate_all};
use crate::state::{
    self, Block, BlockModelKind, CreativeTab, Entity, Item, ItemKind, PotionEffect, Recipe,
};
use anyhow::{Context, Result};
use clap::{Parser, ValueEnum};
use std::fs::create_dir_all;
use std::path::PathBuf;

const DEFAULT_TAB: &str = "testmod";

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

fn add_default_test_project(state: &mut state::ModState) -> Result<()> {
    let dirty = DirtyFlags::all();

    // Creative tab
    {
        let tab = CreativeTab {
            id: DEFAULT_TAB.into(),
        };
        let entity = Entity::CreativeTab(tab);
        if state.try_add(entity)? {
            println!("Added creative tab: {}", DEFAULT_TAB);
        }
    }

    // Basic item
    try_add_item(state, Item::base("testmod_ingot", DEFAULT_TAB))?;

    // Tool item
    {
        let mut item = Item::base("testmod_sword", DEFAULT_TAB);
        item.kind = ItemKind::Tool;
        item.material = Some("iron".into());
        item.attack_damage = Some(6.0);
        item.attack_speed = Some(1.6);
        try_add_item(state, item)?;
    }

    // Axe item
    {
        let mut item = Item::base("testmod_axe", DEFAULT_TAB);
        item.kind = ItemKind::Axe;
        item.material = Some("iron".into());
        item.attack_damage = Some(7.0);
        item.attack_speed = Some(1.0);
        try_add_item(state, item)?;
    }

    // Shovel item
    {
        let mut item = Item::base("testmod_shovel", DEFAULT_TAB);
        item.kind = ItemKind::Shovel;
        item.material = Some("iron".into());
        item.attack_damage = Some(4.5);
        item.attack_speed = Some(1.0);
        try_add_item(state, item)?;
    }

    // Hoe item
    {
        let mut item = Item::base("testmod_hoe", DEFAULT_TAB);
        item.kind = ItemKind::Hoe;
        item.material = Some("iron".into());
        item.attack_damage = Some(1.0);
        item.attack_speed = Some(4.0);
        try_add_item(state, item)?;
    }

    // Food item with tooltips
    {
        let mut item = Item::base("glowing_fruit", DEFAULT_TAB);
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
        try_add_item(state, item)?;
    }

    // Spawn egg
    {
        let mut item = Item::base("phantom_spawn_egg", DEFAULT_TAB);
        item.kind = ItemKind::SpawnEgg;
        item.entity_type = Some("minecraft:phantom".into());
        try_add_item(state, item)?;
    }

    // Fuel item
    {
        let mut item = Item::base("testmod_fuel", DEFAULT_TAB);
        item.kind = ItemKind::Fuel;
        item.burn_time = Some(3200);
        try_add_item(state, item)?;
    }

    // Compostable item
    {
        let mut item = Item::base("plant_fiber", DEFAULT_TAB);
        item.kind = ItemKind::Compostable;
        item.compost_chance = Some(0.6);
        try_add_item(state, item)?;
    }

    // Armor item
    {
        let mut item = Item::base("testmod_helmet", DEFAULT_TAB);
        item.kind = ItemKind::Armor;
        item.armor_material = Some("iron".into());
        item.armor_slot = Some("helmet".into());
        try_add_item(state, item)?;
    }

    // Shield item
    {
        let mut item = Item::base("testmod_shield", DEFAULT_TAB);
        item.kind = ItemKind::Shield;
        try_add_item(state, item)?;
    }

    // Potion item
    {
        let mut item = Item::base("speed_potion", DEFAULT_TAB);
        item.kind = ItemKind::Potion;
        item.effects = vec![PotionEffect {
            effect_type: "minecraft:speed".into(),
            duration: 3600,
            amplifier: 0,
        }];
        try_add_item(state, item)?;
    }

    // Block
    {
        let block = Block::base("testmod_ore", DEFAULT_TAB);
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
    regenerate_all(state, dirty, false).context("Failed to regenerate Java sources")?;

    println!("Test project added successfully!");
    println!("Run 'fw status -v' to see all added entities.");

    Ok(())
}

const VANILLA_TEXTURE_SOURCE: &str =
    "E:\\Coding_Projects\\MCSourceCode\\vanilla-minecraft\\26.2\\assets\\minecraft\\textures";

fn copy_block_textures(
    state: &state::ModState,
    _block_ids: &[&str],
    source_textures: &[(&str, &str)],
) -> Result<()> {
    let textures_root = PathBuf::from("src/main/resources/assets")
        .join(&state.mod_id)
        .join("textures")
        .join("block");

    create_dir_all(&textures_root)?;

    for (dest_name, source_file) in source_textures {
        let dest = textures_root.join(format!("{}.png", dest_name));
        let src = PathBuf::from(VANILLA_TEXTURE_SOURCE)
            .join("block")
            .join(source_file);
        if src.exists() {
            std::fs::copy(&src, &dest)?;
            println!("Copied texture: {} -> {}", src.display(), dest.display());
        }
    }

    Ok(())
}

fn add_blue_nether_base(state: &mut state::ModState) -> Result<()> {
    let tab = "blue_nether";
    let dirty = DirtyFlags::all();

    // Creative tab
    {
        let creative_tab = CreativeTab { id: tab.into() };
        let entity = Entity::CreativeTab(creative_tab);
        if state.try_add(entity)? {
            println!("Added creative tab: {}", tab);
        }
    }

    // Blue Netherrack
    {
        let mut block = Block::base("blue_netherrack", tab);
        block.properties_from = Some("netherrack".into());
        if state.try_add(Entity::Block(block))? {
            println!("Added block: blue_netherrack");
        }
    }

    // Blue Soul Sand
    {
        let mut block = Block::base("blue_soul_sand", tab);
        block.properties_from = Some("soul_sand".into());
        if state.try_add(Entity::Block(block))? {
            println!("Added block: blue_soul_sand");
        }
    }

    // Blue Soul Soil
    {
        let mut block = Block::base("blue_soul_soil", tab);
        block.properties_from = Some("soul_soil".into());
        if state.try_add(Entity::Block(block))? {
            println!("Added block: blue_soul_soil");
        }
    }

    // Blue Basalt
    {
        let mut block = Block::base("blue_basalt", tab);
        block.model_kind = BlockModelKind::CubeBottomTop;
        block.properties_from = Some("basalt".into());
        if state.try_add(Entity::Block(block))? {
            println!("Added block: blue_basalt");
        }
    }

    state.save().context("Failed to save blue nether state")?;
    regenerate_all(state, dirty, false).context("Failed to regenerate Java sources")?;

    // Copy vanilla textures for blue nether blocks
    let texture_map = vec![
        ("blue_netherrack", "netherrack.png"),
        ("blue_soul_sand", "soul_sand.png"),
        ("blue_soul_soil", "soul_soil.png"),
        ("blue_basalt_top", "basalt_top.png"),
        ("blue_basalt_bottom", "basalt_top.png"),
        ("blue_basalt_side", "basalt_side.png"),
    ];
    copy_block_textures(
        state,
        &[
            "blue_netherrack",
            "blue_soul_sand",
            "blue_soul_soil",
            "blue_basalt",
        ],
        &texture_map,
    )?;

    println!("Blue Nether base terrain added!");
    Ok(())
}

/// Preset test-project content definitions.
#[derive(Clone, ValueEnum)]
pub enum Preset {
    /// Default test project with mixed item kinds and recipes
    Default,
    /// Blue Nether base terrain blocks
    BlueNetherBase,
}

/// Adds a complete set of example entities to the current project for testing.
pub fn run(args: TestProjectArgs) -> Result<()> {
    let TestProjectArgs { reset, preset } = args;

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

    match preset {
        Some(Preset::Default) => add_default_test_project(&mut state),
        Some(Preset::BlueNetherBase) => add_blue_nether_base(&mut state),
        None => add_default_test_project(&mut state),
    }
}

/// CLI arguments for `fw test-project`.
#[derive(Parser)]
pub struct TestProjectArgs {
    /// Reset existing project state before adding test entities
    #[arg(long)]
    pub reset: bool,

    /// Preset content to generate
    #[arg(long, value_enum)]
    pub preset: Option<Preset>,
}
