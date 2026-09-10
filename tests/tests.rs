//! Integration tests for fabric-writer.
#![allow(missing_docs)]

mod common;

use anyhow::Result;
use common::TestEnv;
use fabric_writer::commands::block::BlockAddArgs;
use fabric_writer::commands::item::ItemAddArgs;
use fabric_writer::commands::recipe::{RecipeAddArgs, RecipeRemoveArgs, add, remove};
use serial_test::serial;
use std::fs::read;
use std::path::PathBuf;

/// Guard that restores the working directory on drop.
struct DirGuard {
    original: std::path::PathBuf,
}

impl DirGuard {
    fn enter(path: &std::path::Path) -> Self {
        let original = std::env::current_dir().unwrap_or_default();
        std::env::set_current_dir(path).unwrap();
        Self { original }
    }
}

impl Drop for DirGuard {
    fn drop(&mut self) {
        let _ = std::env::set_current_dir(&self.original);
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
    add(args)?;

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

    let content = std::fs::read_to_string(&provider_path)?;
    assert!(content.contains("shaped"), "Missing shaped() call");
    assert!(content.contains("getHasName"), "Missing getHasName call");
    assert!(content.contains("has("), "Missing has() call");

    Ok(())
}

#[test]
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
    add(args)?;

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

    let content = std::fs::read_to_string(&provider_path)?;
    assert!(content.contains("shapeless"), "Missing shapeless() call");
    assert!(content.contains("requires"), "Missing requires() call");

    Ok(())
}

#[test]
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
    add(add_args)?;

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
    fabric_writer::commands::item::add(ItemAddArgs {
        id: "copper_ingot".into(),
        kind: None,
        material: None,
        attack_damage: None,
        attack_speed: None,
        durability: None,
        verbose: false,
    })?;

    // Add a tool item (modded)
    fabric_writer::commands::item::add(ItemAddArgs {
        id: "copper_sword".into(),
        kind: Some("tool".into()),
        material: Some("copper".into()),
        attack_damage: Some(5.0),
        attack_speed: Some(1.6),
        durability: None,
        verbose: false,
    })?;

    // Add a block (modded)
    fabric_writer::commands::block::add(BlockAddArgs {
        id: "copper_ore".into(),
        verbose: false,
    })?;

    // Recipe 1: vanilla-only ingredients, vanilla result
    add(RecipeAddArgs {
        id: "vanilla_recipe".into(),
        kind: Some("crafting_shaped".into()),
        result: Some("minecraft:diamond".into()),
        count: Some(1),
        pattern: vec!["W".into()],
        ingredients: vec!["W=minecraft:stick".into()],
        verbose: false,
    })?;

    // Recipe 2: modded-only ingredients and result
    add(RecipeAddArgs {
        id: "modded_recipe".into(),
        kind: Some("crafting_shaped".into()),
        result: Some("testmod:copper_ingot".into()),
        count: Some(4),
        pattern: vec!["C".into(), "C".into()],
        ingredients: vec!["C=testmod:copper_ore".into()],
        verbose: false,
    })?;

    // Recipe 3: mixed vanilla + modded
    add(RecipeAddArgs {
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

/// Recursively finds all `.json` files under `dir`.
fn find_json_files(dir: &std::path::Path) -> Vec<std::path::PathBuf> {
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
