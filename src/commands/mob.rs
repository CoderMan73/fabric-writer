use crate::java_writer::{DirtyFlags, regenerate_all};
use crate::state::{self, Entity, Mob};
use anyhow::{Context, Result};
use clap::Parser;

/// Adds a mob to the mod state and regenerates Java sources.
pub fn add(args: MobAddArgs) -> Result<()> {
    let mut state = state::load().context("Failed to load project state")?;
    let mob = build_mob(&args);
    let entity = Entity::Mob(mob);
    let dirty = DirtyFlags::from_entity(&entity);
    state.add(entity)?;
    state.save().context("Failed to save state")?;
    regenerate_all(&state, dirty, args.verbose).context("Failed to regenerate Java sources")?;
    println!("Added mob: {}", args.id);
    Ok(())
}

/// Removes a mob from the mod state and regenerates Java sources.
pub fn remove(args: MobRemoveArgs) -> Result<()> {
    let mut state = state::load().context("Failed to load project state")?;
    let mob = Mob {
        id: args.id.clone(),
        ..Default::default()
    };
    let entity = Entity::Mob(mob);
    let dirty = DirtyFlags::from_entity(&entity);
    state.remove(entity)?;
    state.save().context("Failed to save state")?;
    regenerate_all(&state, dirty, args.verbose).context("Failed to regenerate Java sources")?;
    println!("Removed mob: {}", args.id);
    Ok(())
}

fn build_mob(args: &MobAddArgs) -> Mob {
    Mob {
        id: args.id.clone(),
        entity_type: args
            .entity_type
            .clone()
            .unwrap_or_else(|| format!("{}:{}", args.id, args.id)),
        spawn_category: args.spawn_category.clone(),
        spawn_egg_id: Some(args.id.clone()),
        baby_spawn_egg_id: None,
        attributes: Vec::new(),
        drops: Vec::new(),
        ai_type: args.ai_type.clone(),
    }
}

/// CLI arguments for `fw add mob`.
#[derive(Parser)]
pub struct MobAddArgs {
    /// Mob ID
    #[arg(short = 'i', long)]
    pub id: String,

    /// Entity type id (e.g. `minecraft:phantom` or `mymod:blue_ghost`)
    #[arg(long)]
    pub entity_type: Option<String>,

    /// Spawn category (e.g. `monster`, `creature`, `water_creature`)
    #[arg(long)]
    pub spawn_category: Option<String>,

    /// AI behavior type for entity class generation (e.g. `blaze`, `ghast`, `zombie`)
    #[arg(long)]
    pub ai_type: Option<String>,

    /// Show which files were regenerated, skipped, or pruned
    #[arg(short = 'v', long, default_value_t = false)]
    pub verbose: bool,
}

/// CLI arguments for `fw remove mob`.
#[derive(Parser)]
pub struct MobRemoveArgs {
    /// Mob ID to remove.
    #[arg(short = 'i', long)]
    pub id: String,

    /// Show which files were regenerated, skipped, or pruned
    #[arg(short = 'v', long, default_value_t = false)]
    pub verbose: bool,
}
