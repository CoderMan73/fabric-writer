use crate::java_writer::{DirtyFlags, regenerate_all};
use crate::state::{self, Entity};
use anyhow::{Context, Result};
use clap::Parser;

/// Adds a feature to the mod state and regenerates Java sources.
pub fn add(args: FeatureAddArgs) -> Result<()> {
    let mut state = state::load().context("Failed to load project state")?;
    let feature = build_feature(&args);
    let entity = Entity::Feature(feature);
    let dirty = DirtyFlags::from_entity(&entity);
    state.add(entity)?;
    state.save().context("Failed to save state")?;
    regenerate_all(&state, dirty, args.verbose).context("Failed to regenerate Java sources")?;
    println!("Added feature: {}", args.id);
    Ok(())
}

/// Removes a feature from the mod state and regenerates Java sources.
pub fn remove(args: FeatureRemoveArgs) -> Result<()> {
    let mut state = state::load().context("Failed to load project state")?;
    use crate::state::Feature;
    let entity = Entity::Feature(Feature {
        id: args.id.clone(),
        ..Default::default()
    });
    let dirty = DirtyFlags::from_entity(&entity);
    state.remove(entity)?;
    state.save().context("Failed to save state")?;
    regenerate_all(&state, dirty, args.verbose).context("Failed to regenerate Java sources")?;
    println!("Removed feature: {}", args.id);
    Ok(())
}

fn build_feature(args: &FeatureAddArgs) -> crate::state::Feature {
    crate::state::Feature {
        id: args.id.clone(),
        feature_type: args.feature_type.clone(),
        block: args.block.clone(),
        state: args.state.clone(),
        radius: args.radius,
    }
}

/// CLI arguments for `fw add feature`.
#[derive(Parser)]
pub struct FeatureAddArgs {
    /// Feature ID
    #[arg(short = 'i', long)]
    pub id: String,

    /// Feature type key
    #[arg(long)]
    pub feature_type: String,

    /// Block id for feature
    #[arg(long)]
    pub block: Option<String>,

    /// Optional state override
    #[arg(long)]
    pub state: Option<String>,

    /// Optional radius
    #[arg(long)]
    pub radius: Option<i32>,

    /// Show which files were regenerated, skipped, or pruned
    #[arg(short = 'v', long, default_value_t = false)]
    pub verbose: bool,
}

/// CLI arguments for `fw remove feature`.
#[derive(Parser)]
pub struct FeatureRemoveArgs {
    /// Feature ID to remove.
    #[arg(short = 'i', long)]
    pub id: String,

    /// Show which files were regenerated, skipped, or pruned
    #[arg(short = 'v', long, default_value_t = false)]
    pub verbose: bool,
}
