use crate::java_writer::{DirtyFlags, regenerate_all};
use crate::state::{self, Entity};
use anyhow::{Context, Result};
use clap::Parser;

/// Adds a tag to the mod state and regenerates Java sources.
pub fn add(args: TagAddArgs) -> Result<()> {
    let mut state = state::load().context("Failed to load project state")?;
    let tag = build_tag(&args);
    let entity = Entity::Tag(tag);
    let dirty = DirtyFlags::from_entity(&entity);
    state.add(entity)?;
    state.save().context("Failed to save state")?;
    regenerate_all(&state, dirty, args.verbose).context("Failed to regenerate Java sources")?;
    println!("Added tag: {}", args.id);
    Ok(())
}

/// Removes a tag from the mod state and regenerates Java sources.
pub fn remove(args: TagRemoveArgs) -> Result<()> {
    let mut state = state::load().context("Failed to load project state")?;
    use crate::state::Tag;
    let entity = Entity::Tag(Tag {
        id: args.id.clone(),
        ..Default::default()
    });
    let dirty = DirtyFlags::from_entity(&entity);
    state.remove(entity)?;
    state.save().context("Failed to save state")?;
    regenerate_all(&state, dirty, args.verbose).context("Failed to regenerate Java sources")?;
    println!("Removed tag: {}", args.id);
    Ok(())
}

fn build_tag(args: &TagAddArgs) -> crate::state::Tag {
    crate::state::Tag {
        id: args.id.clone(),
        tag_type: args.tag_type.clone(),
        values: Vec::new(),
        replace: args.replace,
    }
}

/// CLI arguments for `fw add tag`.
#[derive(Parser)]
pub struct TagAddArgs {
    /// Tag ID
    #[arg(short = 'i', long)]
    pub id: String,

    /// Tag type (`block`, `item`, `entity`, `fluid`)
    #[arg(long)]
    pub tag_type: String,

    /// Replace existing tag
    #[arg(long)]
    pub replace: Option<bool>,

    /// Show which files were regenerated, skipped, or pruned
    #[arg(short = 'v', long, default_value_t = false)]
    pub verbose: bool,
}

/// CLI arguments for `fw remove tag`.
#[derive(Parser)]
pub struct TagRemoveArgs {
    /// Tag ID to remove.
    #[arg(short = 'i', long)]
    pub id: String,

    /// Show which files were regenerated, skipped, or pruned
    #[arg(short = 'v', long, default_value_t = false)]
    pub verbose: bool,
}
