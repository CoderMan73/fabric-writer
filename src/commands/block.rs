use crate::java_writer::{regenerate_all, DirtyFlags};
use crate::state::{self, Block, Entity};
use anyhow::Result;
use clap::Parser;

/// Adds a block to the mod state and regenerates Java sources.
pub fn add(args: BlockAddArgs) -> Result<()> {
    let verbose = args.verbose;
    let mut state = state::load()?;
    let block = Block::new(&args.id)?;
    let entity = Entity::Block(block.clone());
    let dirty = DirtyFlags::from_entity(&entity);
    state.add(entity)?;
    state.save()?;
    regenerate_all(&state, dirty, verbose)?;
    println!("Added block: {}", args.id);
    Ok(())
}

/// Removes a block from the mod state and regenerates Java sources.
pub fn remove(args: BlockRemoveArgs) -> Result<()> {
    let verbose = args.verbose;
    let mut state = state::load()?;
    let entity = Entity::Block(Block::new(&args.id)?);
    let dirty = DirtyFlags::from_entity(&entity);
    state.remove(entity)?;
    state.save()?;
    regenerate_all(&state, dirty, verbose)?;
    println!("Removed block: {}", args.id);
    Ok(())
}

/// CLI arguments for `fw add block`.
#[derive(Parser)]
pub struct BlockAddArgs {
    /// Block ID
    #[arg(short = 'i', long)]
    pub id: String,

    /// Show which files were regenerated, skipped, or pruned
    #[arg(short = 'v', long, default_value_t = false)]
    pub verbose: bool,
}

/// CLI arguments for `fw remove block`.
#[derive(Parser)]
pub struct BlockRemoveArgs {
    /// Block ID
    #[arg(short = 'i', long)]
    pub id: String,

    /// Show which files were regenerated, skipped, or pruned
    #[arg(short = 'v', long, default_value_t = false)]
    pub verbose: bool,
}
