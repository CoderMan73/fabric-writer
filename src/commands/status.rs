use crate::state::{self};
use anyhow::Result;
use clap::Parser;

/// Prints a summary of the current mod project.
pub fn run(args: StatusArgs) -> Result<()> {
    let state = state::load()?;

    println!("Mod: {} ({})", state.mod_name, state.mod_id);
    println!("Package: {}", state.package_name);
    println!("MC: {}", state.minecraft_version);
    println!("Items: {}", state.items.len());
    for item in &state.items {
        println!("  - {}", item.id);
    }
    println!("Blocks: {}", state.blocks.len());
    for block in &state.blocks {
        println!("  - {}", block.id);
    }
    if args.verbose {
        println!("Advanced options: {:?}", state.advanced_options);
    }

    Ok(())
}

/// CLI arguments for `fw status`.
#[derive(Parser)]
pub struct StatusArgs {
    /// Print verbose status
    #[arg(short = 'v', long)]
    pub verbose: bool,
}
