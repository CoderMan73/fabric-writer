use crate::java_writer::{DirtyFlags, regenerate_all};
use crate::state::{self, CreativeTab, Entity};
use anyhow::Result;
use clap::Parser;

/// Adds a creative tab to the mod state and regenerates Java sources.
pub fn add(args: CreativeTabAddArgs) -> Result<()> {
    let verbose = args.verbose;
    let mut state = state::load()?;
    let tab = CreativeTab::new(&args.id)?;
    let entity = Entity::CreativeTab(tab.clone());
    let dirty = DirtyFlags::from_entity(&entity);
    state.add(entity)?;
    state.save()?;
    regenerate_all(&state, dirty, verbose)?;
    println!("Added creative tab: {}", tab.id);
    Ok(())
}

/// Removes a creative tab from the mod state and regenerates Java sources.
pub fn remove(args: CreativeTabRemoveArgs) -> Result<()> {
    let verbose = args.verbose;
    let mut state = state::load()?;
    let tab = CreativeTab::new(&args.id)?;
    let entity = Entity::CreativeTab(tab.clone());
    let dirty = DirtyFlags::from_entity(&entity);
    state.remove(entity)?;
    state.save()?;
    regenerate_all(&state, dirty, verbose)?;
    println!("Removed creative tab: {}", tab.id);
    Ok(())
}

/// CLI arguments for `fw add creative-tab`.
#[derive(Parser)]
pub struct CreativeTabAddArgs {
    /// Creative tab ID
    #[arg(short = 'i', long)]
    pub id: String,

    /// Show which files were regenerated, skipped, or pruned
    #[arg(short = 'v', long, default_value_t = false)]
    pub verbose: bool,
}

/// CLI arguments for `fw remove creative-tab`.
#[derive(Parser)]
pub struct CreativeTabRemoveArgs {
    /// Creative tab ID to remove.
    #[arg(short = 'i', long)]
    pub id: String,

    /// Show which files were regenerated, skipped, or pruned
    #[arg(short = 'v', long, default_value_t = false)]
    pub verbose: bool,
}
