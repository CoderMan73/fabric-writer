use crate::java_writer::{DirtyFlags, regenerate_all};
use crate::state::{self, Entity};
use anyhow::{Context, Result};
use clap::Parser;

/// Adds a dimension to the mod state and regenerates Java sources.
pub fn add(args: DimensionAddArgs) -> Result<()> {
    let mut state = state::load().context("Failed to load project state")?;
    let dimension = build_dimension(&args);
    let entity = Entity::Dimension(dimension);
    let dirty = DirtyFlags::from_entity(&entity);
    state.add(entity)?;
    state.save().context("Failed to save state")?;
    regenerate_all(&state, dirty, args.verbose).context("Failed to regenerate Java sources")?;
    println!("Added dimension: {}", args.id);
    Ok(())
}

/// Removes a dimension from the mod state and regenerates Java sources.
pub fn remove(args: DimensionRemoveArgs) -> Result<()> {
    let mut state = state::load().context("Failed to load project state")?;
    use crate::state::Dimension;
    let entity = Entity::Dimension(Dimension {
        id: args.id.clone(),
        ..Default::default()
    });
    let dirty = DirtyFlags::from_entity(&entity);
    state.remove(entity)?;
    state.save().context("Failed to save state")?;
    regenerate_all(&state, dirty, args.verbose).context("Failed to regenerate Java sources")?;
    println!("Removed dimension: {}", args.id);
    Ok(())
}

fn build_dimension(args: &DimensionAddArgs) -> crate::state::Dimension {
    crate::state::Dimension {
        id: args.id.clone(),
        dimension_type: args.dimension_type.clone(),
        portal_frame: args.portal_frame.clone(),
        portal_igniter: args.portal_igniter.clone(),
    }
}

/// CLI arguments for `fw add dimension`.
#[derive(Parser)]
pub struct DimensionAddArgs {
    /// Dimension ID
    #[arg(short = 'i', long)]
    pub id: String,

    /// Dimension type key (e.g. `minecraft:overworld` or custom type id)
    #[arg(long)]
    pub dimension_type: String,

    /// Portal frame block id
    #[arg(long)]
    pub portal_frame: Option<String>,

    /// Portal igniter item id
    #[arg(long)]
    pub portal_igniter: Option<String>,

    /// Show which files were regenerated, skipped, or pruned
    #[arg(short = 'v', long, default_value_t = false)]
    pub verbose: bool,
}

/// CLI arguments for `fw remove dimension`.
#[derive(Parser)]
pub struct DimensionRemoveArgs {
    /// Dimension ID to remove.
    #[arg(short = 'i', long)]
    pub id: String,

    /// Show which files were regenerated, skipped, or pruned
    #[arg(short = 'v', long, default_value_t = false)]
    pub verbose: bool,
}
