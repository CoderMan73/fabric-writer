use anyhow::{Context, Result, bail};
use clap::{Parser, Subcommand};
use std::path::PathBuf;

use crate::java_writer::{DirtyFlags, regenerate_all};
use crate::state::{ModState, load_from, save_to};

/// Export or import project state.
pub fn run(args: SaveLoadArgs) -> Result<()> {
    match args.command {
        SaveLoadCommand::Save(args) => save(args),
        SaveLoadCommand::Load(args) => load(args),
    }
}

fn save(args: SaveArgs) -> Result<()> {
    let state = crate::state::load()?;
    let path = PathBuf::from(args.path);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create directory {}", parent.display()))?;
    }
    save_to(&path, &state)?;
    println!("Saved state to {}", path.display());
    Ok(())
}

fn load(args: LoadArgs) -> Result<()> {
    let path = PathBuf::from(&args.path);
    if !path.exists() {
        bail!("State file not found: {}", path.display());
    }
    let state = load_from(&path)?;
    validate(&state)?;
    save_to(&PathBuf::from(".fw/fabric-writer.yml"), &state)?;
    regenerate_all(&state, DirtyFlags::all(), args.verbose)?;
    println!("Loaded state from {}", path.display());
    Ok(())
}

fn validate(state: &ModState) -> Result<()> {
    if state.mod_name.trim().is_empty() {
        bail!("Missing required field: mod_name");
    }
    if state.mod_id.trim().is_empty() {
        bail!("Missing required field: mod_id");
    }
    if state.package_name.trim().is_empty() {
        bail!("Missing required field: package_name");
    }
    if state.minecraft_version.trim().is_empty() {
        bail!("Missing required field: minecraft_version");
    }
    if state.java_path.trim().is_empty() {
        bail!("Missing required field: java_path");
    }

    validate_no_duplicate_ids("item", &state.items, |i| &i.id)?;
    validate_no_duplicate_ids("block", &state.blocks, |b| &b.id)?;
    validate_no_duplicate_ids("recipe", &state.recipes, |r| &r.id)?;

    let valid_ids: std::collections::HashSet<_> = state
        .items
        .iter()
        .map(|i| i.id.clone())
        .chain(state.blocks.iter().map(|b| b.id.clone()))
        .collect();

    for recipe in &state.recipes {
        for (key, value) in &recipe.ingredients {
            if !value.starts_with("minecraft:") && !valid_ids.contains(value) {
                bail!(
                    "Recipe '{}' ingredient '{}' (key '{}') references unknown item/block '{}'",
                    recipe.id,
                    value,
                    key,
                    value
                );
            }
        }
    }

    Ok(())
}

fn validate_no_duplicate_ids<T, F>(label: &str, collection: &[T], id_fn: F) -> Result<()>
where
    F: Fn(&T) -> &str,
{
    let mut seen = std::collections::HashSet::new();
    for item in collection {
        let id = id_fn(item);
        if !seen.insert(id) {
            bail!("Duplicate {} id: '{}'", label, id);
        }
    }
    Ok(())
}

/// CLI arguments for `fw save` and `fw load`.
#[derive(Parser)]
pub struct SaveLoadArgs {
    /// Save or load subcommand
    #[command(subcommand)]
    pub command: SaveLoadCommand,
}

/// Save and load subcommands.
#[derive(Subcommand)]
pub enum SaveLoadCommand {
    /// Export current state to a YAML file
    Save(SaveArgs),
    /// Import state from a YAML file and regenerate Java sources
    Load(LoadArgs),
}

/// Arguments for `fw save`.
#[derive(Parser)]
pub struct SaveArgs {
    /// Output path for the exported state file
    pub path: String,
}

/// Arguments for `fw load`.
#[derive(Parser)]
pub struct LoadArgs {
    /// Path to the state file to import
    pub path: String,

    /// Show which files were regenerated, skipped, or pruned
    #[arg(short = 'v', long, default_value_t = true)]
    pub verbose: bool,
}
