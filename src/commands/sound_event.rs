use crate::java_writer::{DirtyFlags, regenerate_all};
use crate::state::{self, Entity};
use anyhow::{Context, Result};
use clap::Parser;

/// Adds a sound event to the mod state and regenerates Java sources.
pub fn add(args: SoundEventAddArgs) -> Result<()> {
    let mut state = state::load().context("Failed to load project state")?;
    let sound_event = build_sound_event(&args);
    let entity = Entity::SoundEvent(sound_event);
    let dirty = DirtyFlags::from_entity(&entity);
    state.add(entity)?;
    state.save().context("Failed to save state")?;
    regenerate_all(&state, dirty, args.verbose).context("Failed to regenerate Java sources")?;
    println!("Added sound event: {}", args.id);
    Ok(())
}

/// Removes a sound event from the mod state and regenerates Java sources.
pub fn remove(args: SoundEventRemoveArgs) -> Result<()> {
    let mut state = state::load().context("Failed to load project state")?;
    use crate::state::SoundEvent;
    let entity = Entity::SoundEvent(SoundEvent {
        id: args.id.clone(),
        ..Default::default()
    });
    let dirty = DirtyFlags::from_entity(&entity);
    state.remove(entity)?;
    state.save().context("Failed to save state")?;
    regenerate_all(&state, dirty, args.verbose).context("Failed to regenerate Java sources")?;
    println!("Removed sound event: {}", args.id);
    Ok(())
}

fn build_sound_event(args: &SoundEventAddArgs) -> crate::state::SoundEvent {
    crate::state::SoundEvent {
        id: args.id.clone(),
        sound_path: args.sound_path.clone(),
    }
}

/// CLI arguments for `fw add sound_event`.
#[derive(Parser)]
pub struct SoundEventAddArgs {
    /// Sound event ID
    #[arg(short = 'i', long)]
    pub id: String,

    /// Sound path relative to sounds directory
    #[arg(long)]
    pub sound_path: String,

    /// Show which files were regenerated, skipped, or pruned
    #[arg(short = 'v', long, default_value_t = false)]
    pub verbose: bool,
}

/// CLI arguments for `fw remove sound_event`.
#[derive(Parser)]
pub struct SoundEventRemoveArgs {
    /// Sound event ID to remove.
    #[arg(short = 'i', long)]
    pub id: String,

    /// Show which files were regenerated, skipped, or pruned
    #[arg(short = 'v', long, default_value_t = false)]
    pub verbose: bool,
}
