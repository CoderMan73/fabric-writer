use crate::java_writer::{DirtyFlags, regenerate_all};
use crate::state::{self, Entity};
use anyhow::{Context, Result};
use clap::Parser;

/// Adds a biome to the mod state and regenerates Java sources.
pub fn add(args: BiomeAddArgs) -> Result<()> {
    let mut state = state::load().context("Failed to load project state")?;
    let biome = build_biome(&args);
    let entity = Entity::Biome(biome);
    let dirty = DirtyFlags::from_entity(&entity);
    state.add(entity)?;
    state.save().context("Failed to save state")?;
    regenerate_all(&state, dirty, args.verbose).context("Failed to regenerate Java sources")?;
    println!("Added biome: {}", args.id);
    Ok(())
}

/// Removes a biome from the mod state and regenerates Java sources.
pub fn remove(args: BiomeRemoveArgs) -> Result<()> {
    let mut state = state::load().context("Failed to load project state")?;
    use crate::state::Biome;
    let entity = Entity::Biome(Biome {
        id: args.id.clone(),
        ..Default::default()
    });
    let dirty = DirtyFlags::from_entity(&entity);
    state.remove(entity)?;
    state.save().context("Failed to save state")?;
    regenerate_all(&state, dirty, args.verbose).context("Failed to regenerate Java sources")?;
    println!("Removed biome: {}", args.id);
    Ok(())
}

fn build_biome(args: &BiomeAddArgs) -> crate::state::Biome {
    crate::state::Biome {
        id: args.id.clone(),
        temperature: args.temperature,
        downfall: args.downfall,
        sky_color: args.sky_color,
        water_color: args.water_color,
        water_fog_color: args.water_fog_color,
        fog_color: args.fog_color,
        has_precipitation: args.has_precipitation,
        features: Vec::new(),
        structures: Vec::new(),
        mob_spawns: Vec::new(),
        player_spawn_friendly: None,
    }
}

/// CLI arguments for `fw add biome`.
#[derive(Parser)]
pub struct BiomeAddArgs {
    /// Biome ID
    #[arg(short = 'i', long)]
    pub id: String,

    /// Biome temperature
    #[arg(long)]
    pub temperature: Option<f32>,

    /// Biome downfall
    #[arg(long)]
    pub downfall: Option<f32>,

    /// Sky color as integer RGB
    #[arg(long)]
    pub sky_color: Option<i32>,

    /// Water color as integer RGB
    #[arg(long)]
    pub water_color: Option<i32>,

    /// Water fog color as integer RGB
    #[arg(long)]
    pub water_fog_color: Option<i32>,

    /// Fog color as integer RGB
    #[arg(long)]
    pub fog_color: Option<i32>,

    /// Whether the biome has precipitation
    #[arg(long)]
    pub has_precipitation: Option<bool>,

    /// Show which files were regenerated, skipped, or pruned
    #[arg(short = 'v', long, default_value_t = false)]
    pub verbose: bool,
}

/// CLI arguments for `fw remove biome`.
#[derive(Parser)]
pub struct BiomeRemoveArgs {
    /// Biome ID to remove.
    #[arg(short = 'i', long)]
    pub id: String,

    /// Show which files were regenerated, skipped, or pruned
    #[arg(short = 'v', long, default_value_t = false)]
    pub verbose: bool,
}
