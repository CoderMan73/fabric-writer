use crate::java_writer::{DirtyFlags, regenerate_all};
use crate::state::{self, Block, CreativeTab, Entity, Item, ItemKind, PotionEffect, Recipe};
use anyhow::{Context, Result};
use clap::Parser;

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
        let tab = CreativeTab {
            id: "testmod".into(),
        };
        let entity = Entity::CreativeTab(tab);
        if state.try_add(entity)? {
            println!("Added creative tab: testmod");
        }
    }

    // Basic item
    {
        let item = Item {
            id: "testmod_ingot".into(),
            kind: ItemKind::Basic,
            material: None,
            attack_damage: None,
            attack_speed: None,
            durability: None,
            tooltip: vec![],
            nutrition: None,
            saturation: None,
            always_edible: false,
            entity_type: None,
            burn_time: None,
            compost_chance: None,
            creative_tab: Some("testmod".into()),
            armor_material: None,
            armor_slot: None,
            effects: vec![],
        };
        if state.try_add(Entity::Item(item))? {
            println!("Added item: testmod_ingot (basic)");
        }
    }

    // Tool item
    {
        let item = Item {
            id: "testmod_sword".into(),
            kind: ItemKind::Tool,
            material: Some("iron".into()),
            attack_damage: Some(6.0),
            attack_speed: Some(1.6),
            durability: None,
            tooltip: vec![],
            nutrition: None,
            saturation: None,
            always_edible: false,
            entity_type: None,
            burn_time: None,
            compost_chance: None,
            creative_tab: Some("testmod".into()),
            armor_material: None,
            armor_slot: None,
            effects: vec![],
        };
        if state.try_add(Entity::Item(item))? {
            println!("Added item: testmod_sword (tool)");
        }
    }

    // Axe item
    {
        let item = Item {
            id: "testmod_axe".into(),
            kind: ItemKind::Axe,
            material: Some("iron".into()),
            attack_damage: Some(7.0),
            attack_speed: Some(1.0),
            durability: None,
            tooltip: vec![],
            nutrition: None,
            saturation: None,
            always_edible: false,
            entity_type: None,
            burn_time: None,
            compost_chance: None,
            creative_tab: Some("testmod".into()),
            armor_material: None,
            armor_slot: None,
            effects: vec![],
        };
        if state.try_add(Entity::Item(item))? {
            println!("Added item: testmod_axe (axe)");
        }
    }

    // Shovel item
    {
        let item = Item {
            id: "testmod_shovel".into(),
            kind: ItemKind::Shovel,
            material: Some("iron".into()),
            attack_damage: Some(4.5),
            attack_speed: Some(1.0),
            durability: None,
            tooltip: vec![],
            nutrition: None,
            saturation: None,
            always_edible: false,
            entity_type: None,
            burn_time: None,
            compost_chance: None,
            creative_tab: Some("testmod".into()),
            armor_material: None,
            armor_slot: None,
            effects: vec![],
        };
        if state.try_add(Entity::Item(item))? {
            println!("Added item: testmod_shovel (shovel)");
        }
    }

    // Hoe item
    {
        let item = Item {
            id: "testmod_hoe".into(),
            kind: ItemKind::Hoe,
            material: Some("iron".into()),
            attack_damage: Some(1.0),
            attack_speed: Some(4.0),
            durability: None,
            tooltip: vec![],
            nutrition: None,
            saturation: None,
            always_edible: false,
            entity_type: None,
            burn_time: None,
            compost_chance: None,
            creative_tab: Some("testmod".into()),
            armor_material: None,
            armor_slot: None,
            effects: vec![],
        };
        if state.try_add(Entity::Item(item))? {
            println!("Added item: testmod_hoe (hoe)");
        }
    }

    // Food item with tooltips
    {
        let item = Item {
            id: "glowing_fruit".into(),
            kind: ItemKind::Food,
            material: None,
            attack_damage: None,
            attack_speed: None,
            durability: None,
            tooltip: vec![
                "A mysterious glowing fruit.".into(),
                "Grants temporary night vision.".into(),
            ],
            nutrition: Some(6),
            saturation: Some(0.4),
            always_edible: true,
            entity_type: None,
            burn_time: None,
            compost_chance: None,
            creative_tab: Some("testmod".into()),
            armor_material: None,
            armor_slot: None,
            effects: vec![PotionEffect {
                effect_type: "minecraft:night_vision".into(),
                duration: 1200,
                amplifier: 0,
            }],
        };
        if state.try_add(Entity::Item(item))? {
            println!("Added item: glowing_fruit (food)");
        }
    }

    // Spawn egg
    {
        let item = Item {
            id: "phantom_spawn_egg".into(),
            kind: ItemKind::SpawnEgg,
            material: None,
            attack_damage: None,
            attack_speed: None,
            durability: None,
            tooltip: vec![],
            nutrition: None,
            saturation: None,
            always_edible: false,
            entity_type: Some("minecraft:phantom".into()),
            burn_time: None,
            compost_chance: None,
            creative_tab: Some("testmod".into()),
            armor_material: None,
            armor_slot: None,
            effects: vec![],
        };
        if state.try_add(Entity::Item(item))? {
            println!("Added item: phantom_spawn_egg (spawn_egg)");
        }
    }

    // Fuel item
    {
        let item = Item {
            id: "testmod_fuel".into(),
            kind: ItemKind::Fuel,
            material: None,
            attack_damage: None,
            attack_speed: None,
            durability: None,
            tooltip: vec![],
            nutrition: None,
            saturation: None,
            always_edible: false,
            entity_type: None,
            burn_time: Some(3200),
            compost_chance: None,
            creative_tab: Some("testmod".into()),
            armor_material: None,
            armor_slot: None,
            effects: vec![],
        };
        if state.try_add(Entity::Item(item))? {
            println!("Added item: testmod_fuel (fuel)");
        }
    }

    // Compostable item
    {
        let item = Item {
            id: "plant_fiber".into(),
            kind: ItemKind::Compostable,
            material: None,
            attack_damage: None,
            attack_speed: None,
            durability: None,
            tooltip: vec![],
            nutrition: None,
            saturation: None,
            always_edible: false,
            entity_type: None,
            burn_time: None,
            compost_chance: Some(0.6),
            creative_tab: Some("testmod".into()),
            armor_material: None,
            armor_slot: None,
            effects: vec![],
        };
        if state.try_add(Entity::Item(item))? {
            println!("Added item: plant_fiber (compostable)");
        }
    }

    // Armor item
    {
        let item = Item {
            id: "testmod_helmet".into(),
            kind: ItemKind::Armor,
            material: None,
            attack_damage: None,
            attack_speed: None,
            durability: None,
            tooltip: vec![],
            nutrition: None,
            saturation: None,
            always_edible: false,
            entity_type: None,
            burn_time: None,
            compost_chance: None,
            creative_tab: Some("testmod".into()),
            armor_material: Some("iron".into()),
            armor_slot: Some("helmet".into()),
            effects: vec![],
        };
        if state.try_add(Entity::Item(item))? {
            println!("Added item: testmod_helmet (armor)");
        }
    }

    // Shield item
    {
        let item = Item {
            id: "testmod_shield".into(),
            kind: ItemKind::Shield,
            material: None,
            attack_damage: None,
            attack_speed: None,
            durability: None,
            tooltip: vec![],
            nutrition: None,
            saturation: None,
            always_edible: false,
            entity_type: None,
            burn_time: None,
            compost_chance: None,
            creative_tab: Some("testmod".into()),
            armor_material: None,
            armor_slot: None,
            effects: vec![],
        };
        if state.try_add(Entity::Item(item))? {
            println!("Added item: testmod_shield (shield)");
        }
    }

    // Potion item
    {
        let item = Item {
            id: "speed_potion".into(),
            kind: ItemKind::Potion,
            material: None,
            attack_damage: None,
            attack_speed: None,
            durability: None,
            tooltip: vec![],
            nutrition: None,
            saturation: None,
            always_edible: false,
            entity_type: None,
            burn_time: None,
            compost_chance: None,
            creative_tab: Some("testmod".into()),
            armor_material: None,
            armor_slot: None,
            effects: vec![PotionEffect {
                effect_type: "minecraft:speed".into(),
                duration: 3600,
                amplifier: 0,
            }],
        };
        if state.try_add(Entity::Item(item))? {
            println!("Added item: speed_potion (potion)");
        }
    }

    // Block
    {
        let block = Block {
            id: "testmod_ore".into(),
            creative_tab: Some("testmod".into()),
        };
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
