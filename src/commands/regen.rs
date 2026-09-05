use anyhow::Result;
use clap::Parser;

use crate::java_writer::{DirtyFlags, regenerate_all};
use crate::state;

/// Regenerates all Java sources from the current mod state, regardless of dirty flags.
pub fn run(args: RegenArgs) -> Result<()> {
    let state = state::load()?;
    regenerate_all(&state, DirtyFlags::all(), args.verbose)?;
    println!("Regenerated all Java files");
    Ok(())
}

/// CLI arguments for `fw regen`.
#[derive(Parser)]
pub struct RegenArgs {
    /// Show which files were regenerated, skipped, or pruned [default: true]
    #[arg(short = 'v', long, default_value_t = true)]
    pub verbose: bool,
}
