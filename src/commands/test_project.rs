use crate::java_writer::{DirtyFlags, regenerate_all};
use crate::state::{
    self, Advancement, Biome, Block, BlockModelKind, CreativeTab, Dimension, Entity, Feature, Item,
    ItemKind, LootTable, Mob, PotionEffect, Recipe, SoundEvent, Structure, Tag,
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
        ItemKind::MusicDisc => "music_disc",
        ItemKind::FireCharge => "fire_charge",
        ItemKind::FlintAndSteel => "flint_and_steel",
        ItemKind::Compass => "compass",
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
        let mcmeta_src = src.with_extension("png.mcmeta");
        if mcmeta_src.exists() {
            let mcmeta_dest = dest.with_extension("png.mcmeta");
            std::fs::copy(&mcmeta_src, &mcmeta_dest)?;
            println!(
                "Copied texture metadata: {} -> {}",
                mcmeta_src.display(),
                mcmeta_dest.display()
            );
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

    // Blue Polished Basalt
    {
        let mut block = Block::base("blue_polished_basalt", tab);
        block.model_kind = BlockModelKind::CubeBottomTop;
        block.properties_from = Some("polished_basalt".into());
        if state.try_add(Entity::Block(block))? {
            println!("Added block: blue_polished_basalt");
        }
    }

    // Blue Blackstone
    {
        let mut block = Block::base("blue_blackstone", tab);
        block.model_kind = BlockModelKind::CubeBottomTop;
        block.properties_from = Some("blackstone".into());
        if state.try_add(Entity::Block(block))? {
            println!("Added block: blue_blackstone");
        }
    }

    // Blue Magma Block
    {
        let mut block = Block::base("blue_magma_block", tab);
        block.properties_from = Some("magma_block".into());
        if state.try_add(Entity::Block(block))? {
            println!("Added block: blue_magma_block");
        }
    }

    // Blue Glowstone
    {
        let mut block = Block::base("blue_glowstone", tab);
        block.properties_from = Some("glowstone".into());
        if state.try_add(Entity::Block(block))? {
            println!("Added block: blue_glowstone");
        }
    }

    // Blue Shroomlight
    {
        let mut block = Block::base("blue_shroomlight", tab);
        block.properties_from = Some("shroomlight".into());
        if state.try_add(Entity::Block(block))? {
            println!("Added block: blue_shroomlight");
        }
    }

    // Blue Nether Quartz Ore
    {
        let mut block = Block::base("blue_nether_quartz_ore", tab);
        block.properties_from = Some("nether_quartz_ore".into());
        if state.try_add(Entity::Block(block))? {
            println!("Added block: blue_nether_quartz_ore");
        }
    }

    // Blue Nether Gold Ore
    {
        let mut block = Block::base("blue_nether_gold_ore", tab);
        block.properties_from = Some("nether_gold_ore".into());
        if state.try_add(Entity::Block(block))? {
            println!("Added block: blue_nether_gold_ore");
        }
    }

    // Blue Ancient Debris
    {
        let mut block = Block::base("blue_ancient_debris", tab);
        block.model_kind = BlockModelKind::CubeBottomTop;
        block.properties_from = Some("ancient_debris".into());
        if state.try_add(Entity::Block(block))? {
            println!("Added block: blue_ancient_debris");
        }
    }

    // Blue Gravel
    {
        let mut block = Block::base("blue_gravel", tab);
        block.properties_from = Some("gravel".into());
        if state.try_add(Entity::Block(block))? {
            println!("Added block: blue_gravel");
        }
    }

    // Blue Bedrock
    {
        let mut block = Block::base("blue_bedrock", tab);
        block.properties_from = Some("bedrock".into());
        if state.try_add(Entity::Block(block))? {
            println!("Added block: blue_bedrock");
        }
    }

    // Blue Nether Bricks
    {
        let mut block = Block::base("blue_nether_bricks", tab);
        block.properties_from = Some("nether_bricks".into());
        if state.try_add(Entity::Block(block))? {
            println!("Added block: blue_nether_bricks");
        }
    }

    // Blue Crying Obsidian
    {
        let mut block = Block::base("blue_crying_obsidian", tab);
        block.properties_from = Some("crying_obsidian".into());
        if state.try_add(Entity::Block(block))? {
            println!("Added block: blue_crying_obsidian");
        }
    }

    // Blue Gold Block
    {
        let mut block = Block::base("blue_gold_block", tab);
        block.properties_from = Some("gold_block".into());
        if state.try_add(Entity::Block(block))? {
            println!("Added block: blue_gold_block");
        }
    }

    // Blue Bone Block
    {
        let mut block = Block::base("blue_bone_block", tab);
        block.model_kind = BlockModelKind::CubeColumn;
        block.properties_from = Some("bone_block".into());
        if state.try_add(Entity::Block(block))? {
            println!("Added block: blue_bone_block");
        }
    }

    // Blue Quartz Block
    {
        let mut block = Block::base("blue_quartz_block", tab);
        block.model_kind = BlockModelKind::CubeBottomTop;
        block.properties_from = Some("quartz_block".into());
        if state.try_add(Entity::Block(block))? {
            println!("Added block: blue_quartz_block");
        }
    }

    // Blue Iron Chain
    {
        let mut block = Block::base("blue_iron_chain", tab);
        block.model_kind = BlockModelKind::Chain;
        block.properties_from = Some("iron_chain".into());
        block.block_class = "ChainBlock".into();
        if state.try_add(Entity::Block(block))? {
            println!("Added block: blue_iron_chain");
        }
    }

    // Blue Lantern
    {
        let mut block = Block::base("blue_lantern", tab);
        block.model_kind = BlockModelKind::Lantern;
        block.properties_from = Some("lantern".into());
        block.block_class = "LanternBlock".into();
        if state.try_add(Entity::Block(block))? {
            println!("Added block: blue_lantern");
        }
    }

    // Blue Netherite Ingot
    {
        let item = Item::base("blue_netherite_ingot", tab);
        if state.try_add(Entity::Item(item))? {
            println!("Added item: blue_netherite_ingot");
        }
    }

    // Blue Blaze Rod
    {
        let item = Item::base("blue_blaze_rod", tab);
        if state.try_add(Entity::Item(item))? {
            println!("Added item: blue_blaze_rod");
        }
    }

    // Blue Ghast Tear
    {
        let item = Item::base("blue_ghast_tear", tab);
        if state.try_add(Entity::Item(item))? {
            println!("Added item: blue_ghast_tear");
        }
    }

    // Blue Magma Cream
    {
        let item = Item::base("blue_magma_cream", tab);
        if state.try_add(Entity::Item(item))? {
            println!("Added item: blue_magma_cream");
        }
    }

    // Blue Nether Wart
    {
        let item = Item::base("blue_nether_wart", tab);
        if state.try_add(Entity::Item(item))? {
            println!("Added item: blue_nether_wart");
        }
    }

    // Blue Fire Charge
    {
        let mut item = Item::new("blue_fire_charge")?;
        item.kind = ItemKind::FireCharge;
        if state.try_add(Entity::Item(item))? {
            println!("Added item: blue_fire_charge");
        }
    }

    // Blue Flint and Steel
    {
        let mut item = Item::new("blue_flint_and_steel")?;
        item.kind = ItemKind::FlintAndSteel;
        if state.try_add(Entity::Item(item))? {
            println!("Added item: blue_flint_and_steel");
        }
    }

    // Blue Netherite Sword
    {
        let mut item = Item::new("blue_netherite_sword")?;
        item.kind = ItemKind::Tool;
        item.material = Some("netherite".into());
        item.attack_damage = Some(8.0);
        item.attack_speed = Some(-2.4);
        if state.try_add(Entity::Item(item))? {
            println!("Added item: blue_netherite_sword");
        }
    }

    // Blue Netherite Pickaxe
    {
        let mut item = Item::new("blue_netherite_pickaxe")?;
        item.kind = ItemKind::Tool;
        item.material = Some("netherite".into());
        item.attack_damage = Some(2.0);
        item.attack_speed = Some(-2.8);
        if state.try_add(Entity::Item(item))? {
            println!("Added item: blue_netherite_pickaxe");
        }
    }

    // Blue Nether Bricks recipe
    {
        let mut recipe = Recipe::new("blue_nether_bricks")?;
        recipe.kind = "crafting_shaped".into();
        recipe.pattern = vec!["NNN".into(), "NNN".into(), "NNN".into()];
        recipe
            .ingredients
            .insert("N".into(), "blue_netherrack".into());
        recipe.result = "blue_nether_bricks".into();
        recipe.count = 4;
        if state.try_add(Entity::Recipe(recipe))? {
            println!("Added recipe: blue_nether_bricks");
        }
    }

    // Blue Netherite Ingot recipe
    {
        let mut recipe = Recipe::new("blue_netherite_ingot")?;
        recipe.kind = "crafting_shaped".into();
        recipe.pattern = vec!["NNN".into(), "NGN".into(), "NNN".into()];
        recipe
            .ingredients
            .insert("N".into(), "minecraft:netherite_scrap".into());
        recipe
            .ingredients
            .insert("G".into(), "minecraft:gold_ingot".into());
        recipe.result = "blue_netherite_ingot".into();
        recipe.count = 1;
        if state.try_add(Entity::Recipe(recipe))? {
            println!("Added recipe: blue_netherite_ingot");
        }
    }

    // Blue Nether Bricks smelting recipe
    {
        let mut recipe = Recipe::new("blue_nether_bricks_smelting")?;
        recipe.kind = "smelting".into();
        recipe
            .ingredients
            .insert("I".into(), "blue_netherrack".into());
        recipe.result = "blue_nether_bricks".into();
        recipe.count = 1;
        recipe.cooking_time = Some(200);
        recipe.experience = Some(0.5);
        recipe.category = Some("building_blocks".into());
        if state.try_add(Entity::Recipe(recipe))? {
            println!("Added recipe: blue_nether_bricks_smelting");
        }
    }

    // Blue Ghast mob
    {
        let mob = Mob {
            id: "blue_ghast".into(),
            entity_type: "minecraft:ghast".into(),
            spawn_category: Some("monster".into()),
            spawn_egg_id: Some("blue_ghast_spawn_egg".into()),
            baby_spawn_egg_id: None,
            attributes: vec![],
            drops: vec![],
        };
        if state.try_add(Entity::Mob(mob))? {
            println!("Added mob: blue_ghast");
        }
    }

    // Blue Strider mob
    {
        let mob = Mob {
            id: "blue_strider".into(),
            entity_type: "minecraft:strider".into(),
            spawn_category: Some("creature".into()),
            spawn_egg_id: Some("blue_strider_spawn_egg".into()),
            baby_spawn_egg_id: None,
            attributes: vec![],
            drops: vec![],
        };
        if state.try_add(Entity::Mob(mob))? {
            println!("Added mob: blue_strider");
        }
    }

    // Blue Zombie Piglin mob
    {
        let mob = Mob {
            id: "blue_zombie_piglin".into(),
            entity_type: "minecraft:zombified_piglin".into(),
            spawn_category: Some("monster".into()),
            spawn_egg_id: Some("blue_zombie_piglin_spawn_egg".into()),
            baby_spawn_egg_id: None,
            attributes: vec![],
            drops: vec![],
        };
        if state.try_add(Entity::Mob(mob))? {
            println!("Added mob: blue_zombie_piglin");
        }
    }

    // Blue Hoglin mob
    {
        let mob = Mob {
            id: "blue_hoglin".into(),
            entity_type: "minecraft:hoglin".into(),
            spawn_category: Some("creature".into()),
            spawn_egg_id: Some("blue_hoglin_spawn_egg".into()),
            baby_spawn_egg_id: None,
            attributes: vec![],
            drops: vec![],
        };
        if state.try_add(Entity::Mob(mob))? {
            println!("Added mob: blue_hoglin");
        }
    }

    // Blue Piglin mob
    {
        let mob = Mob {
            id: "blue_piglin".into(),
            entity_type: "minecraft:piglin".into(),
            spawn_category: Some("creature".into()),
            spawn_egg_id: Some("blue_piglin_spawn_egg".into()),
            baby_spawn_egg_id: None,
            attributes: vec![],
            drops: vec![],
        };
        if state.try_add(Entity::Mob(mob))? {
            println!("Added mob: blue_piglin");
        }
    }

    // Blue Wither Skeleton mob
    {
        let mob = Mob {
            id: "blue_wither_skeleton".into(),
            entity_type: "minecraft:wither_skeleton".into(),
            spawn_category: Some("monster".into()),
            spawn_egg_id: Some("blue_wither_skeleton_spawn_egg".into()),
            baby_spawn_egg_id: None,
            attributes: vec![],
            drops: vec![],
        };
        if state.try_add(Entity::Mob(mob))? {
            println!("Added mob: blue_wither_skeleton");
        }
    }

    // Blue Nether biome
    {
        let biome = Biome {
            id: "blue_nether".into(),
            temperature: Some(2.0),
            downfall: Some(0.0),
            sky_color: None,
            water_color: None,
            water_fog_color: None,
            fog_color: None,
            has_precipitation: Some(false),
            features: vec![],
            structures: vec![],
            mob_spawns: vec![],
            player_spawn_friendly: None,
        };
        if state.try_add(Entity::Biome(biome))? {
            println!("Added biome: blue_nether");
        }
    }

    // Blue Nether dimension
    {
        let dimension = Dimension {
            id: "blue_nether".into(),
            dimension_type: "minecraft:nether".into(),
            portal_frame: None,
            portal_igniter: None,
        };
        if state.try_add(Entity::Dimension(dimension))? {
            println!("Added dimension: blue_nether");
        }
    }

    // Blue Nether structure
    {
        let structure = Structure {
            id: "blue_nether_fortress".into(),
            structure_type: "minecraft:fortress".into(),
            spawn: None,
            spacing: None,
            separation: None,
            salt: None,
        };
        if state.try_add(Entity::Structure(structure))? {
            println!("Added structure: blue_nether_fortress");
        }
    }

    // Blue Nether feature
    {
        let feature = Feature {
            id: "blue_nether_wart_patch".into(),
            feature_type: "minecraft:nether_wart_patch".into(),
            block: None,
            state: None,
            radius: None,
        };
        if state.try_add(Entity::Feature(feature))? {
            println!("Added feature: blue_nether_wart_patch");
        }
    }

    // Blue Nether loot table
    {
        let loot_table = LootTable {
            id: "blue_nether_chest".into(),
            loot_type: "chest".into(),
            rolls: None,
            entries: vec![],
        };
        if state.try_add(Entity::LootTable(loot_table))? {
            println!("Added loot_table: blue_nether_chest");
        }
    }

    // Blue Nether advancement
    {
        let advancement = Advancement {
            id: "enter_blue_nether".into(),
            parent: None,
            title: "Enter the Blue Nether".into(),
            description: "Enter the blue Nether dimension".into(),
            icon: "minecraft:netherrack".into(),
            background: None,
            frame: None,
            show_toast: None,
            announce_to_chat: None,
            hidden: None,
            criteria: vec![],
        };
        if state.try_add(Entity::Advancement(advancement))? {
            println!("Added advancement: enter_blue_nether");
        }
    }

    // Blue Nether sound event
    {
        let sound_event = SoundEvent {
            id: "blue_nether_ambient".into(),
            sound_path: "ambient/nether/blue_nether_ambient".into(),
        };
        if state.try_add(Entity::SoundEvent(sound_event))? {
            println!("Added sound_event: blue_nether_ambient");
        }
    }

    // Blue Nether tag
    {
        let tag = Tag {
            id: "blue_nether_blocks".into(),
            tag_type: "block".into(),
            values: vec!["blue_netherrack".into(), "blue_soul_sand".into()],
            replace: None,
        };
        if state.try_add(Entity::Tag(tag))? {
            println!("Added tag: blue_nether_blocks");
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
        ("blue_polished_basalt_top", "polished_basalt_top.png"),
        ("blue_polished_basalt_bottom", "polished_basalt_top.png"),
        ("blue_polished_basalt_side", "polished_basalt_side.png"),
        ("blue_blackstone_top", "blackstone_top.png"),
        ("blue_blackstone_bottom", "blackstone_top.png"),
        ("blue_blackstone_side", "blackstone.png"),
        ("blue_magma_block", "magma.png"),
        ("blue_glowstone", "glowstone.png"),
        ("blue_shroomlight", "shroomlight.png"),
        ("blue_nether_quartz_ore", "nether_quartz_ore.png"),
        ("blue_nether_gold_ore", "nether_gold_ore.png"),
        ("blue_ancient_debris_top", "ancient_debris_top.png"),
        ("blue_ancient_debris_bottom", "ancient_debris_top.png"),
        ("blue_ancient_debris_side", "ancient_debris_side.png"),
        ("blue_gravel", "gravel.png"),
        ("blue_bedrock", "bedrock.png"),
        ("blue_nether_bricks", "nether_bricks.png"),
        ("blue_crying_obsidian", "crying_obsidian.png"),
        ("blue_gold_block", "gold_block.png"),
        ("blue_bone_block_top", "bone_block_top.png"),
        ("blue_bone_block_bottom", "bone_block_bottom.png"),
        ("blue_bone_block_side", "bone_block_side.png"),
        ("blue_quartz_block_top", "quartz_block_top.png"),
        ("blue_quartz_block_bottom", "quartz_block_bottom.png"),
        ("blue_quartz_block_side", "quartz_block_side.png"),
        ("blue_iron_chain", "iron_chain.png"),
        ("blue_lantern", "lantern.png"),
        ("blue_crimson_nylium", "crimson_nylium.png"),
        ("blue_warped_nylium", "warped_nylium.png"),
        ("blue_crimson_stem", "crimson_stem.png"),
        ("blue_crimson_stem_top", "crimson_stem_top.png"),
        ("blue_warped_stem", "warped_stem.png"),
        ("blue_warped_stem_top", "warped_stem_top.png"),
        ("blue_crimson_fungus", "crimson_fungus.png"),
        ("blue_warped_fungus", "warped_fungus.png"),
        ("blue_crimson_roots", "crimson_roots.png"),
        ("blue_warped_roots", "warped_roots.png"),
        ("blue_nether_sprouts", "nether_sprouts.png"),
        ("blue_weeping_vines", "weeping_vines.png"),
        ("blue_weeping_vines_plant", "weeping_vines_plant.png"),
        ("blue_twisting_vines", "twisting_vines.png"),
        ("blue_twisting_vines_plant", "twisting_vines_plant.png"),
        ("blue_brown_mushroom", "brown_mushroom.png"),
        ("blue_red_mushroom", "red_mushroom.png"),
        ("blue_nether_wart", "nether_wart_stage0.png"),
        ("blue_nether_brick_fence", "nether_brick_fence.png"),
        ("blue_nether_brick_stairs", "nether_brick_stairs.png"),
        ("blue_chiseled_nether_bricks", "chiseled_nether_bricks.png"),
        ("blue_cracked_nether_bricks", "cracked_nether_bricks.png"),
        ("blue_polished_blackstone", "polished_blackstone.png"),
        (
            "blue_polished_blackstone_bricks",
            "polished_blackstone_bricks.png",
        ),
        (
            "blue_cracked_polished_blackstone_bricks",
            "cracked_polished_blackstone_bricks.png",
        ),
        (
            "blue_chiseled_polished_blackstone",
            "chiseled_polished_blackstone.png",
        ),
        ("blue_gilded_blackstone", "gilded_blackstone.png"),
        ("blue_blackstone_stairs", "blackstone_stairs.png"),
        ("blue_blackstone_slab", "blackstone_slab.png"),
        ("blue_blackstone_wall", "blackstone_wall.png"),
        ("blue_smooth_quartz_block", "smooth_quartz_block.png"),
        ("blue_smooth_quartz_slab", "smooth_quartz_slab.png"),
    ];
    copy_block_textures(
        state,
        &[
            "blue_netherrack",
            "blue_soul_sand",
            "blue_soul_soil",
            "blue_basalt",
            "blue_polished_basalt",
            "blue_blackstone",
            "blue_magma_block",
            "blue_glowstone",
            "blue_shroomlight",
            "blue_nether_quartz_ore",
            "blue_nether_gold_ore",
            "blue_ancient_debris",
            "blue_gravel",
            "blue_bedrock",
            "blue_nether_bricks",
            "blue_crying_obsidian",
            "blue_gold_block",
            "blue_bone_block",
            "blue_quartz_block",
            "blue_iron_chain",
            "blue_lantern",
            "blue_crimson_nylium",
            "blue_warped_nylium",
            "blue_crimson_stem",
            "blue_warped_stem",
            "blue_crimson_fungus",
            "blue_warped_fungus",
            "blue_crimson_roots",
            "blue_warped_roots",
            "blue_nether_sprouts",
            "blue_weeping_vines",
            "blue_twisting_vines",
            "blue_brown_mushroom",
            "blue_red_mushroom",
            "blue_nether_wart",
            "blue_nether_brick_fence",
            "blue_nether_brick_stairs",
            "blue_chiseled_nether_bricks",
            "blue_cracked_nether_bricks",
            "blue_polished_blackstone",
            "blue_polished_blackstone_bricks",
            "blue_cracked_polished_blackstone_bricks",
            "blue_chiseled_polished_blackstone",
            "blue_polished_blackstone_stairs",
            "blue_polished_blackstone_slab",
            "blue_polished_blackstone_brick_stairs",
            "blue_polished_blackstone_brick_slab",
            "blue_polished_blackstone_brick_wall",
            "blue_blackstone_stairs",
            "blue_blackstone_slab",
            "blue_blackstone_wall",
            "blue_gilded_blackstone",
            "blue_smooth_quartz_block",
            "blue_smooth_quartz_slab",
        ],
        &texture_map,
    )?;

    // Nylium blocks
    {
        let mut block = Block::base("blue_crimson_nylium", tab);
        block.properties_from = Some("crimson_nylium".into());
        if state.try_add(Entity::Block(block))? {
            println!("Added block: blue_crimson_nylium");
        }
    }
    {
        let mut block = Block::base("blue_warped_nylium", tab);
        block.properties_from = Some("warped_nylium".into());
        if state.try_add(Entity::Block(block))? {
            println!("Added block: blue_warped_nylium");
        }
    }

    // Fungi and plant blocks
    let plant_blocks = [
        ("blue_crimson_stem", "crimson_stem"),
        ("blue_warped_stem", "warped_stem"),
        ("blue_crimson_fungus", "crimson_fungus"),
        ("blue_warped_fungus", "warped_fungus"),
        ("blue_crimson_roots", "crimson_roots"),
        ("blue_warped_roots", "warped_roots"),
        ("blue_nether_sprouts", "nether_sprouts"),
        ("blue_weeping_vines", "weeping_vines"),
        ("blue_twisting_vines", "twisting_vines"),
        ("blue_brown_mushroom", "brown_mushroom"),
        ("blue_red_mushroom", "red_mushroom"),
    ];
    for (blue_id, vanilla_id) in plant_blocks {
        let mut block = Block::base(blue_id, tab);
        block.properties_from = Some(vanilla_id.into());
        if state.try_add(Entity::Block(block))? {
            println!("Added block: {}", blue_id);
        }
    }

    // Fire and portal blocks
    {
        let mut block = Block::base("blue_fire", tab);
        block.properties_from = Some("fire".into());
        if state.try_add(Entity::Block(block))? {
            println!("Added block: blue_fire");
        }
    }
    {
        let mut block = Block::base("blue_soul_fire", tab);
        block.properties_from = Some("soul_fire".into());
        if state.try_add(Entity::Block(block))? {
            println!("Added block: blue_soul_fire");
        }
    }
    {
        let mut block = Block::base("blue_nether_portal", tab);
        block.properties_from = Some("nether_portal".into());
        if state.try_add(Entity::Block(block))? {
            println!("Added block: blue_nether_portal");
        }
    }

    // Blue Nether Wart crop
    {
        let mut block = Block::base("blue_nether_wart", tab);
        block.properties_from = Some("nether_wart".into());
        if state.try_add(Entity::Block(block))? {
            println!("Added block: blue_nether_wart");
        }
    }

    // Structure blocks
    let structure_blocks = [
        ("blue_nether_brick_fence", "nether_brick_fence"),
        ("blue_nether_brick_stairs", "nether_brick_stairs"),
        ("blue_chiseled_nether_bricks", "chiseled_nether_bricks"),
        ("blue_cracked_nether_bricks", "cracked_nether_bricks"),
        ("blue_polished_blackstone", "polished_blackstone"),
        (
            "blue_polished_blackstone_bricks",
            "polished_blackstone_bricks",
        ),
        (
            "blue_cracked_polished_blackstone_bricks",
            "cracked_polished_blackstone_bricks",
        ),
        (
            "blue_chiseled_polished_blackstone",
            "chiseled_polished_blackstone",
        ),
        (
            "blue_polished_blackstone_stairs",
            "polished_blackstone_stairs",
        ),
        ("blue_polished_blackstone_slab", "polished_blackstone_slab"),
        (
            "blue_polished_blackstone_brick_stairs",
            "polished_blackstone_brick_stairs",
        ),
        (
            "blue_polished_blackstone_brick_slab",
            "polished_blackstone_brick_slab",
        ),
        (
            "blue_polished_blackstone_brick_wall",
            "polished_blackstone_brick_wall",
        ),
        ("blue_blackstone_stairs", "blackstone_stairs"),
        ("blue_blackstone_slab", "blackstone_slab"),
        ("blue_blackstone_wall", "blackstone_wall"),
        ("blue_gilded_blackstone", "gilded_blackstone"),
        ("blue_smooth_quartz_block", "smooth_quartz_block"),
        ("blue_smooth_quartz_slab", "smooth_quartz_slab"),
    ];
    for (blue_id, vanilla_id) in structure_blocks {
        let mut block = Block::base(blue_id, tab);
        block.properties_from = Some(vanilla_id.into());
        if state.try_add(Entity::Block(block))? {
            println!("Added block: {}", blue_id);
        }
    }

    // Items for new blocks
    let item_blocks = [
        "blue_crimson_nylium",
        "blue_warped_nylium",
        "blue_crimson_stem",
        "blue_warped_stem",
        "blue_crimson_fungus",
        "blue_warped_fungus",
        "blue_crimson_roots",
        "blue_warped_roots",
        "blue_nether_sprouts",
        "blue_weeping_vines",
        "blue_twisting_vines",
        "blue_brown_mushroom",
        "blue_red_mushroom",
        "blue_nether_wart",
        "blue_nether_brick_fence",
        "blue_nether_brick_stairs",
        "blue_chiseled_nether_bricks",
        "blue_cracked_nether_bricks",
        "blue_polished_blackstone",
        "blue_polished_blackstone_bricks",
        "blue_cracked_polished_blackstone_bricks",
        "blue_chiseled_polished_blackstone",
        "blue_polished_blackstone_stairs",
        "blue_polished_blackstone_slab",
        "blue_polished_blackstone_brick_stairs",
        "blue_polished_blackstone_brick_slab",
        "blue_polished_blackstone_brick_wall",
        "blue_blackstone_stairs",
        "blue_blackstone_slab",
        "blue_blackstone_wall",
        "blue_gilded_blackstone",
        "blue_smooth_quartz_block",
        "blue_smooth_quartz_slab",
    ];
    for block_id in item_blocks {
        let item = Item::base(block_id, tab);
        if state.try_add(Entity::Item(item))? {
            println!("Added item: {}", block_id);
        }
    }

    // Additional items
    let extra_items = [
        "blue_netherite_scrap",
        "blue_nether_star",
        "blue_blaze_powder",
        "blue_crimson_fungus",
        "blue_warped_fungus",
        "blue_leather",
        "blue_obsidian",
        "blue_music_disc",
    ];
    for item_id in extra_items {
        let item = Item::base(item_id, tab);
        if state.try_add(Entity::Item(item))? {
            println!("Added item: {}", item_id);
        }
    }

    // Recipes for new blocks
    {
        let mut recipe = Recipe::new("blue_nether_bricks_from_blue_netherrack")?;
        recipe.kind = "crafting_shaped".into();
        recipe.pattern = vec!["NN".into(), "NN".into()];
        recipe
            .ingredients
            .insert("N".into(), "blue_netherrack".into());
        recipe.result = "blue_nether_bricks".into();
        recipe.count = 4;
        if state.try_add(Entity::Recipe(recipe))? {
            println!("Added recipe: blue_nether_bricks_from_blue_netherrack");
        }
    }

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
        state.mobs.clear();
        state.biomes.clear();
        state.dimensions.clear();
        state.structures.clear();
        state.features.clear();
        state.loot_tables.clear();
        state.advancements.clear();
        state.sound_events.clear();
        state.tags.clear();
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
