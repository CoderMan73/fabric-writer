use crate::java_writer::{DirtyFlags, regenerate_all};
use crate::state::{self, Entity};
use anyhow::{Context, Result};
use clap::Parser;

/// Adds a loot table to the mod state and regenerates Java sources.
pub fn add(args: LootTableAddArgs) -> Result<()> {
    let mut state = state::load().context("Failed to load project state")?;
    let loot_table = build_loot_table(&args);
    let entity = Entity::LootTable(loot_table);
    let dirty = DirtyFlags::from_entity(&entity);
    state.add(entity)?;
    state.save().context("Failed to save state")?;
    regenerate_all(&state, dirty, args.verbose).context("Failed to regenerate Java sources")?;
    println!("Added loot table: {}", args.id);
    Ok(())
}

/// Removes a loot table from the mod state and regenerates Java sources.
pub fn remove(args: LootTableRemoveArgs) -> Result<()> {
    let mut state = state::load().context("Failed to load project state")?;
    use crate::state::LootTable;
    let entity = Entity::LootTable(LootTable {
        id: args.id.clone(),
        ..Default::default()
    });
    let dirty = DirtyFlags::from_entity(&entity);
    state.remove(entity)?;
    state.save().context("Failed to save state")?;
    regenerate_all(&state, dirty, args.verbose).context("Failed to regenerate Java sources")?;
    println!("Removed loot table: {}", args.id);
    Ok(())
}

fn build_loot_table(args: &LootTableAddArgs) -> crate::state::LootTable {
    crate::state::LootTable {
        id: args.id.clone(),
        loot_type: args.loot_type.clone(),
        rolls: args.rolls,
        entries: Vec::new(),
    }
}

/// CLI arguments for `fw add loot_table`.
#[derive(Parser)]
pub struct LootTableAddArgs {
    /// Loot table ID
    #[arg(short = 'i', long)]
    pub id: String,

    /// Loot table type (e.g. `entity`, `block`, `chest`)
    #[arg(long)]
    pub loot_type: String,

    /// Roll count
    #[arg(long)]
    pub rolls: Option<u32>,

    /// Show which files were regenerated, skipped, or pruned
    #[arg(short = 'v', long, default_value_t = false)]
    pub verbose: bool,
}

/// CLI arguments for `fw remove loot_table`.
#[derive(Parser)]
pub struct LootTableRemoveArgs {
    /// Loot table ID to remove.
    #[arg(short = 'i', long)]
    pub id: String,

    /// Show which files were regenerated, skipped, or pruned
    #[arg(short = 'v', long, default_value_t = false)]
    pub verbose: bool,
}
