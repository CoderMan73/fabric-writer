use crate::{java_writer::regenerate_all, state};
use anyhow::Result;

pub fn run() -> Result<()> {
    let state = state::load()?;
    regenerate_all(&state)?;
    println!("Regenerated all Java files");
    Ok(())
}
