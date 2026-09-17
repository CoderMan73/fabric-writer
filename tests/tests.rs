//! Integration tests for fabric-writer.
#![allow(missing_docs)]

mod common;

use anyhow::Result;
use common::TestEnv;
use fabric_writer::commands::block::BlockAddArgs;
use fabric_writer::commands::item::{ItemAddArgs, add as item_add};
use fabric_writer::commands::recipe::{RecipeAddArgs, RecipeRemoveArgs, add as recipe_add, remove};
use fabric_writer::commands::save_load;
use fabric_writer::commands::test_project;
use serial_test::serial;
use std::env;
use std::fs::{read, read_to_string};
use std::path::{Path, PathBuf};

/// Guard that restores the working directory on drop.
struct DirGuard {
    original: std::path::PathBuf,
}

impl DirGuard {
    fn enter(path: &Path) -> Self {
        let original = env::current_dir().unwrap_or_default();
        env::set_current_dir(path).unwrap();
        Self { original }
    }
}

impl Drop for DirGuard {
    fn drop(&mut self) {
        let _ = env::set_current_dir(&self.original);
    }
}

#[test]
#[ignore]
#[serial]
fn cache_copies_correctly() -> Result<()> {
    let env = TestEnv::new()?;

    // Verify key files from the cache were copied identically to the temp project.
    let cache = PathBuf::from(".testing-cache/26.2/TestMod");

    for relative in ["gradle.properties", ".fw/fabric-writer.yml"] {
        let cache_file = cache.join(relative);
        let temp_file = env.project_dir.join(relative);

        assert!(cache_file.exists(), "Cache file missing: {}", relative);
        assert!(temp_file.exists(), "Temp file missing: {}", relative);

        let cache_contents = read(&cache_file)?;
        let temp_contents = read(&temp_file)?;
        assert_eq!(
            cache_contents, temp_contents,
            "File contents differ: {}",
            relative
        );
    }

    Ok(())
}

#[test]
#[ignore]
#[serial]
fn add_shaped_recipe_generates_provider() -> Result<()> {
    let env = TestEnv::new()?;
    let _guard = DirGuard::enter(&env.project_dir);

    let args = RecipeAddArgs {
        id: "wooden_gear".into(),
        kind: Some("crafting_shaped".into()),
        result: Some("minecraft:diamond".into()),
        count: Some(1),
        pattern: vec!["W W".into(), " W ".into(), "W W".into()],
        ingredients: vec!["W=minecraft:wood".into()],
        verbose: false,
    };
    recipe_add(args)?;

    let provider_path = env
        .project_dir
        .join("src/client/java")
        .join("testmod")
        .join("client")
        .join("TestModRecipeProvider.java");
    assert!(
        provider_path.exists(),
        "RecipeProvider.java was not generated"
    );

    let content = read_to_string(&provider_path)?;
    assert!(content.contains("shaped"), "Missing shaped() call");
    assert!(content.contains("getHasName"), "Missing getHasName call");
    assert!(content.contains("has("), "Missing has() call");

    Ok(())
}

#[test]
#[ignore]
#[serial]
fn add_shapeless_recipe_generates_provider() -> Result<()> {
    let env = TestEnv::new()?;
    let _guard = DirGuard::enter(&env.project_dir);

    let args = RecipeAddArgs {
        id: "mixed_dirt".into(),
        kind: Some("crafting_shapeless".into()),
        result: Some("minecraft:dirt".into()),
        count: Some(4),
        pattern: vec![],
        ingredients: vec!["X=minecraft:coarse_dirt".into()],
        verbose: false,
    };
    recipe_add(args)?;

    let provider_path = env
        .project_dir
        .join("src/client/java")
        .join("testmod")
        .join("client")
        .join("TestModRecipeProvider.java");
    assert!(
        provider_path.exists(),
        "RecipeProvider.java was not generated"
    );

    let content = read_to_string(&provider_path)?;
    assert!(content.contains("shapeless"), "Missing shapeless() call");
    assert!(content.contains("requires"), "Missing requires() call");

    Ok(())
}

#[test]
#[ignore]
#[serial]
fn remove_recipe_prunes_provider() -> Result<()> {
    let env = TestEnv::new()?;
    let _guard = DirGuard::enter(&env.project_dir);

    let add_args = RecipeAddArgs {
        id: "temp_recipe".into(),
        kind: Some("crafting_shaped".into()),
        result: Some("minecraft:dirt".into()),
        count: Some(1),
        pattern: vec!["D".into()],
        ingredients: vec!["D=minecraft:dirt".into()],
        verbose: false,
    };
    recipe_add(add_args)?;

    let provider_path = env
        .project_dir
        .join("src/client/java")
        .join("testmod")
        .join("client")
        .join("TestModRecipeProvider.java");
    assert!(
        provider_path.exists(),
        "RecipeProvider.java should exist after add"
    );

    let remove_args = RecipeRemoveArgs {
        id: "temp_recipe".into(),
        verbose: false,
    };
    remove(remove_args)?;

    assert!(
        !provider_path.exists(),
        "RecipeProvider.java should be pruned when no recipes remain"
    );

    Ok(())
}

#[test]
#[ignore]
#[serial]
fn datagen_succeeds_with_mixed_vanilla_and_modded_content() -> Result<()> {
    let env = TestEnv::new()?;
    let _guard = DirGuard::enter(&env.project_dir);

    // Add a basic item (modded)
    item_add(ItemAddArgs {
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

    // Add a tool item (modded)
    item_add(ItemAddArgs {
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

    // Add a block (modded)
    fabric_writer::commands::block::add(BlockAddArgs {
        id: "copper_ore".into(),
        creative_tab: None,
        verbose: false,
    })?;

    // Recipe 1: vanilla-only ingredients, vanilla result
    recipe_add(RecipeAddArgs {
        id: "vanilla_recipe".into(),
        kind: Some("crafting_shaped".into()),
        result: Some("minecraft:diamond".into()),
        count: Some(1),
        pattern: vec!["W".into()],
        ingredients: vec!["W=minecraft:stick".into()],
        verbose: false,
    })?;

    // Recipe 2: modded-only ingredients and result
    recipe_add(RecipeAddArgs {
        id: "modded_recipe".into(),
        kind: Some("crafting_shaped".into()),
        result: Some("testmod:copper_ingot".into()),
        count: Some(4),
        pattern: vec!["C".into(), "C".into()],
        ingredients: vec!["C=testmod:copper_ore".into()],
        verbose: false,
    })?;

    // Recipe 3: mixed vanilla + modded
    recipe_add(RecipeAddArgs {
        id: "mixed_recipe".into(),
        kind: Some("crafting_shapeless".into()),
        result: Some("minecraft:dirt".into()),
        count: Some(4),
        pattern: vec![],
        ingredients: vec![
            "X=minecraft:coarse_dirt".into(),
            "Y=testmod:copper_ingot".into(),
        ],
        verbose: false,
    })?;

    // Run datagen — this will fail if the generated Java has errors
    fabric_writer::commands::run::datagen()?;

    // Verify that datagen produced recipe JSON files
    // Fabric Loom outputs datagen results to src/main/generated
    let datagen_output = env.project_dir.join("src/main/generated");
    assert!(
        datagen_output.exists(),
        "Datagen output directory was not created"
    );

    // Search for recipe JSON files in the datagen output
    let recipe_files: Vec<_> = find_json_files(&datagen_output)
        .into_iter()
        .filter(|path| path.to_string_lossy().contains("recipes"))
        .collect();

    assert!(
        !recipe_files.is_empty(),
        "No recipe JSON files found in datagen output"
    );

    assert!(
        recipe_files.len() >= 3,
        "Expected at least 3 recipe JSON files, found {}",
        recipe_files.len()
    );

    Ok(())
}

#[test]
#[ignore]
#[serial]
fn test_project_generates_valid_java_and_datagen() -> Result<()> {
    let env = TestEnv::new()?;
    let _guard = DirGuard::enter(&env.project_dir);

    // Add the full test project
    test_project::run()?;

    // Run datagen — this will fail if the generated Java has errors
    fabric_writer::commands::run::datagen()?;

    // Verify datagen output exists
    let datagen_output = env.project_dir.join("src/main/generated");
    assert!(
        datagen_output.exists(),
        "Datagen output directory was not created"
    );

    Ok(())
}

#[test]
#[ignore]
#[serial]
fn add_item_with_tooltips_generates_custom_class() -> Result<()> {
    let env = TestEnv::new()?;
    let _guard = DirGuard::enter(&env.project_dir);

    item_add(ItemAddArgs {
        id: "glow_berry".into(),
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
        tooltip: vec!["A glowing berry.".into(), "Consumes on use.".into()],
        creative_tab: None,
        armor_material: None,
        armor_slot: None,
        effect: vec![],
        verbose: false,
    })?;

    let java_root = env.project_dir.join("src/main/java").join("testmod");
    let item_class = java_root.join("GLOW_BERRYItem.java");
    let mod_items = java_root.join("ModItems.java");
    let lang_provider = env
        .project_dir
        .join("src/client/java")
        .join("testmod")
        .join("client")
        .join("LangProvider.java");

    assert!(item_class.exists(), "Expected GLOW_BERRYItem.java to exist");
    assert!(mod_items.exists(), "Expected ModItems.java to exist");
    assert!(
        lang_provider.exists(),
        "Expected LangProvider.java to exist"
    );

    let item_src = read(&item_class)?;
    let mod_items_src = read(&mod_items)?;
    let lang_src = read(&lang_provider)?;

    let item_src_str = String::from_utf8(item_src)?;
    let mod_items_str = String::from_utf8(mod_items_src)?;
    let lang_str = String::from_utf8(lang_src)?;

    assert!(
        item_src_str.contains("appendHoverText"),
        "Expected custom item class to override appendHoverText"
    );
    assert!(
        item_src_str.contains("GLOW_BERRYItem"),
        "Expected custom item class to be named GLOW_BERRYItem"
    );
    assert!(
        mod_items_str.contains("GLOW_BERRYItem::new"),
        "Expected ModItems to register GLOW_BERRY using GLOW_BERRYItem"
    );
    assert!(
        lang_str.contains("itemTooltip.testmod.glow_berry.0"),
        "Expected tooltip translation key 0 in LangProvider"
    );
    assert!(
        lang_str.contains("itemTooltip.testmod.glow_berry.1"),
        "Expected tooltip translation key 1 in LangProvider"
    );
    assert!(
        lang_str.contains("A glowing berry."),
        "Expected tooltip translation text 0 in LangProvider"
    );
    assert!(
        lang_str.contains("Consumes on use."),
        "Expected tooltip translation text 1 in LangProvider"
    );

    Ok(())
}

#[test]
#[ignore]
#[serial]
fn add_armor_item_generates_class_and_provider() -> Result<()> {
    let env = TestEnv::new()?;
    let _guard = DirGuard::enter(&env.project_dir);

    item_add(ItemAddArgs {
        id: "diamond_helmet".into(),
        kind: Some("armor".into()),
        material: Some("diamond".into()),
        armor_material: Some("diamond".into()),
        armor_slot: Some("helmet".into()),
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
        effect: vec![],
        verbose: false,
    })?;

    let java_root = env.project_dir.join("src/main/java").join("testmod");
    let item_class = java_root.join("DIAMOND_HELMETItem.java");
    let mod_items = java_root.join("ModItems.java");
    let armor_provider = env
        .project_dir
        .join("src/client/java")
        .join("testmod")
        .join("client")
        .join("TestModArmorProvider.java");

    assert!(
        item_class.exists(),
        "Expected DIAMOND_HELMETItem.java to exist"
    );
    assert!(mod_items.exists(), "Expected ModItems.java to exist");
    assert!(
        armor_provider.exists(),
        "Expected TestModArmorProvider.java to exist"
    );

    let item_src = read_to_string(&item_class)?;
    let mod_items_src = read_to_string(&mod_items)?;
    let provider_src = read_to_string(&armor_provider)?;

    assert!(
        item_src.contains("extends ArmorItem"),
        "Expected custom armor class to extend ArmorItem"
    );
    assert!(
        mod_items_src.contains("DIAMOND_HELMETItem::new"),
        "Expected ModItems to register DIAMOND_HELMET using DIAMOND_HELMETItem"
    );
    assert!(
        provider_src.contains("generateArmor"),
        "Expected ArmorProvider to call generateArmor"
    );

    Ok(())
}

#[test]
#[ignore]
#[serial]
fn add_shield_item_generates_class_and_provider() -> Result<()> {
    let env = TestEnv::new()?;
    let _guard = DirGuard::enter(&env.project_dir);

    item_add(ItemAddArgs {
        id: "custom_shield".into(),
        kind: Some("shield".into()),
        material: None,
        armor_material: None,
        armor_slot: None,
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
        effect: vec![],
        verbose: false,
    })?;

    let java_root = env.project_dir.join("src/main/java").join("testmod");
    let item_class = java_root.join("CUSTOM_SHIELDItem.java");
    let mod_items = java_root.join("ModItems.java");
    let shield_provider = env
        .project_dir
        .join("src/client/java")
        .join("testmod")
        .join("client")
        .join("TestModShieldProvider.java");

    assert!(
        item_class.exists(),
        "Expected CUSTOM_SHIELDItem.java to exist"
    );
    assert!(mod_items.exists(), "Expected ModItems.java to exist");
    assert!(
        shield_provider.exists(),
        "Expected TestModShieldProvider.java to exist"
    );

    let item_src = read_to_string(&item_class)?;
    let mod_items_src = read_to_string(&mod_items)?;
    let provider_src = read_to_string(&shield_provider)?;

    assert!(
        item_src.contains("extends ShieldItem"),
        "Expected custom shield class to extend ShieldItem"
    );
    assert!(
        mod_items_src.contains("CUSTOM_SHIELDItem::new"),
        "Expected ModItems to register CUSTOM_SHIELD using CUSTOM_SHIELDItem"
    );
    assert!(
        provider_src.contains("generateShield"),
        "Expected ShieldProvider to call generateShield"
    );

    Ok(())
}

#[test]
#[ignore]
#[serial]
fn add_potion_item_generates_data_component() -> Result<()> {
    let env = TestEnv::new()?;
    let _guard = DirGuard::enter(&env.project_dir);

    item_add(ItemAddArgs {
        id: "speed_elixir".into(),
        kind: Some("potion".into()),
        material: None,
        armor_material: None,
        armor_slot: None,
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
        effect: vec!["minecraft:speed=3600:1".into()],
        verbose: false,
    })?;

    let java_root = env.project_dir.join("src/main/java").join("testmod");
    let mod_items = java_root.join("ModItems.java");

    assert!(mod_items.exists(), "Expected ModItems.java to exist");

    let mod_items_src = read_to_string(&mod_items)?;

    assert!(
        mod_items_src.contains("POTION_CONTENTS"),
        "Expected ModItems to set POTION_CONTENTS data component"
    );
    assert!(
        mod_items_src.contains("SPEED"),
        "Expected ModItems to reference SPEED effect"
    );

    Ok(())
}

/// Recursively finds all `.json` files under `dir`.
fn find_json_files(dir: &Path) -> Vec<std::path::PathBuf> {
    let mut results = Vec::new();
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                results.extend(find_json_files(&path));
            } else if path.extension().is_some_and(|ext| ext == "json") {
                results.push(path);
            }
        }
    }
    results
}

#[test]
#[ignore]
#[serial]
fn save_produces_yaml_file() -> Result<()> {
    let env = TestEnv::new()?;
    let _guard = DirGuard::enter(&env.project_dir);

    item_add(ItemAddArgs {
        id: "test_item".into(),
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

    let save_path = env.project_dir.join("exported_state.yml");
    let args = save_load::SaveLoadArgs {
        command: save_load::SaveLoadCommand::Save(save_load::SaveArgs {
            path: save_path.display().to_string(),
        }),
    };
    save_load::run(args)?;

    assert!(save_path.exists(), "Exported state file should exist");
    let content = read_to_string(&save_path)?;
    assert!(
        content.contains("test_item"),
        "Exported YAML should contain item id"
    );
    assert!(
        content.contains("mod_name"),
        "Exported YAML should contain mod_name"
    );

    Ok(())
}

#[test]
#[ignore]
#[serial]
fn load_recreates_state_and_regenerates() -> Result<()> {
    let env = TestEnv::new()?;
    let _guard = DirGuard::enter(&env.project_dir);

    item_add(ItemAddArgs {
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

    fabric_writer::commands::block::add(BlockAddArgs {
        id: "copper_ore".into(),
        creative_tab: None,
        verbose: false,
    })?;

    let save_path = env.project_dir.join("exported_state.yml");
    let save_args = save_load::SaveLoadArgs {
        command: save_load::SaveLoadCommand::Save(save_load::SaveArgs {
            path: save_path.display().to_string(),
        }),
    };
    save_load::run(save_args)?;

    // Remove all items and blocks
    fabric_writer::commands::item::remove(fabric_writer::commands::item::ItemRemoveArgs {
        id: "copper_ingot".into(),
        verbose: false,
    })?;
    fabric_writer::commands::block::remove(fabric_writer::commands::block::BlockRemoveArgs {
        id: "copper_ore".into(),
        verbose: false,
    })?;

    let java_root = env.project_dir.join("src/main/java").join("testmod");
    let item_class = java_root.join("COPPER_INGOTItem.java");
    let block_class = java_root.join("COPPER_OREBlock.java");
    assert!(
        !item_class.exists(),
        "Item class should be pruned after remove"
    );
    assert!(
        !block_class.exists(),
        "Block class should be pruned after remove"
    );

    // Load state back
    let load_args = save_load::SaveLoadArgs {
        command: save_load::SaveLoadCommand::Load(save_load::LoadArgs {
            path: save_path.display().to_string(),
            verbose: false,
        }),
    };
    save_load::run(load_args)?;

    assert!(
        item_class.exists(),
        "Item class should be regenerated after load"
    );
    assert!(
        block_class.exists(),
        "Block class should be regenerated after load"
    );

    Ok(())
}

#[test]
#[ignore]
#[serial]
fn load_validates_duplicate_ids() -> Result<()> {
    let env = TestEnv::new()?;
    let _guard = DirGuard::enter(&env.project_dir);

    let bad_yaml = r#"mod_name: TestMod
mod_id: testmod
namespace: testmod
package_name: testmod
minecraft_version: "26.2"
java_path: /tmp/java
items:
  - id: duplicate_item
    kind: basic
  - id: duplicate_item
    kind: basic
"#;

    let bad_path = env.project_dir.join("bad_state.yml");
    std::fs::write(&bad_path, bad_yaml)?;

    let args = save_load::SaveLoadArgs {
        command: save_load::SaveLoadCommand::Load(save_load::LoadArgs {
            path: bad_path.display().to_string(),
            verbose: false,
        }),
    };
    let result = save_load::run(args);
    assert!(
        result.is_err(),
        "Loading state with duplicate ids should fail"
    );
    let err_msg = result.unwrap_err().to_string();
    assert!(
        err_msg.contains("Duplicate"),
        "Error should mention duplicate id"
    );

    Ok(())
}

#[test]
#[ignore]
#[serial]
fn load_validates_invalid_recipe_references() -> Result<()> {
    let env = TestEnv::new()?;
    let _guard = DirGuard::enter(&env.project_dir);

    let bad_yaml = r#"mod_name: TestMod
mod_id: testmod
namespace: testmod
package_name: testmod
minecraft_version: "26.2"
java_path: /tmp/java
items: []
blocks: []
recipes:
  - id: bad_recipe
    kind: crafting_shaped
    pattern:
      - "X"
    ingredients:
      X: nonexistent_item
    result: minecraft:dirt
    count: 1
"#;

    let bad_path = env.project_dir.join("bad_state.yml");
    std::fs::write(&bad_path, bad_yaml)?;

    let args = save_load::SaveLoadArgs {
        command: save_load::SaveLoadCommand::Load(save_load::LoadArgs {
            path: bad_path.display().to_string(),
            verbose: false,
        }),
    };
    let result = save_load::run(args);
    assert!(
        result.is_err(),
        "Loading state with invalid references should fail"
    );
    let err_msg = result.unwrap_err().to_string();
    assert!(
        err_msg.contains("references unknown"),
        "Error should mention unknown reference"
    );

    Ok(())
}
