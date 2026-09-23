use crate::java_writer::{DirtyFlags, regenerate_all};
use crate::state::{self, Entity};
use anyhow::{Context, Result};
use clap::Parser;

/// Adds a structure to the mod state and regenerates Java sources.
pub fn add(args: StructureAddArgs) -> Result<()> {
    let mut state = state::load().context("Failed to load project state")?;
    let structure = build_structure(&args);
    let entity = Entity::Structure(structure);
    let dirty = DirtyFlags::from_entity(&entity);
    state.add(entity)?;
    state.save().context("Failed to save state")?;
    regenerate_all(&state, dirty, args.verbose).context("Failed to regenerate Java sources")?;
    println!("Added structure: {}", args.id);
    Ok(())
}

/// Removes a structure from the mod state and regenerates Java sources.
pub fn remove(args: StructureRemoveArgs) -> Result<()> {
    let mut state = state::load().context("Failed to load project state")?;
    use crate::state::Structure;
    let entity = Entity::Structure(Structure {
        id: args.id.clone(),
        ..Default::default()
    });
    let dirty = DirtyFlags::from_entity(&entity);
    state.remove(entity)?;
    state.save().context("Failed to save state")?;
    regenerate_all(&state, dirty, args.verbose).context("Failed to regenerate Java sources")?;
    println!("Removed structure: {}", args.id);
    Ok(())
}

fn build_structure(args: &StructureAddArgs) -> crate::state::Structure {
    crate::state::Structure {
        id: args.id.clone(),
        structure_type: args.structure_type.clone(),
        spawn: args.spawn.clone(),
        spacing: args.spacing,
        separation: args.separation,
        salt: args.salt,
    }
}

/// CLI arguments for `fw add structure`.
#[derive(Parser)]
pub struct StructureAddArgs {
    /// Structure ID
    #[arg(short = 'i', long)]
    pub id: String,

    /// Structure type key
    #[arg(long)]
    pub structure_type: String,

    /// Biome id for spawning
    #[arg(long)]
    pub spawn: Option<String>,

    /// Spacing (chunks)
    #[arg(long)]
    pub spacing: Option<i32>,

    /// Separation (chunks)
    #[arg(long)]
    pub separation: Option<i32>,

    /// Structure salt
    #[arg(long)]
    pub salt: Option<i32>,

    /// Show which files were regenerated, skipped, or pruned
    #[arg(short = 'v', long, default_value_t = false)]
    pub verbose: bool,
}

/// CLI arguments for `fw remove structure`.
#[derive(Parser)]
pub struct StructureRemoveArgs {
    /// Structure ID to remove.
    #[arg(short = 'i', long)]
    pub id: String,

    /// Show which files were regenerated, skipped, or pruned
    #[arg(short = 'v', long, default_value_t = false)]
    pub verbose: bool,
}
