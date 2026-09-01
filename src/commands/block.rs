use crate::commands::write_fns::regenerate_all;
use crate::state::{self, Block, Entity};
use anyhow::Result;
use clap::Parser;

pub fn add(args: BlockAddArgs) -> Result<()> {
    let mut state = state::load()?;

    let block = Block::new(&args.id)?;
    state.add(Entity::Block(block))?;
    state.save()?;
    regenerate_all(&state)?;

    println!("Added block: {}", args.id);
    Ok(())
}

pub fn remove(args: BlockRemoveArgs) -> Result<()> {
    let mut state = state::load()?;

    state.remove(Entity::Block(Block::new(&args.id)?))?;
    state.save()?;
    regenerate_all(&state)?;

    println!("Removed block: {}", args.id);
    Ok(())
}

#[derive(Parser)]
pub struct BlockAddArgs {
    pub id: String,
}

#[derive(Parser)]
pub struct BlockRemoveArgs {
    pub id: String,
}
