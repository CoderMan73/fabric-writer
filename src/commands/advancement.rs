use crate::java_writer::{DirtyFlags, regenerate_all};
use crate::state::{self, Entity};
use anyhow::{Context, Result};
use clap::Parser;

/// Adds an advancement to the mod state and regenerates Java sources.
pub fn add(args: AdvancementAddArgs) -> Result<()> {
    let mut state = state::load().context("Failed to load project state")?;
    let advancement = build_advancement(&args);
    let entity = Entity::Advancement(advancement);
    let dirty = DirtyFlags::from_entity(&entity);
    state.add(entity)?;
    state.save().context("Failed to save state")?;
    regenerate_all(&state, dirty, args.verbose).context("Failed to regenerate Java sources")?;
    println!("Added advancement: {}", args.id);
    Ok(())
}

/// Removes an advancement from the mod state and regenerates Java sources.
pub fn remove(args: AdvancementRemoveArgs) -> Result<()> {
    let mut state = state::load().context("Failed to load project state")?;
    use crate::state::Advancement;
    let entity = Entity::Advancement(Advancement {
        id: args.id.clone(),
        ..Default::default()
    });
    let dirty = DirtyFlags::from_entity(&entity);
    state.remove(entity)?;
    state.save().context("Failed to save state")?;
    regenerate_all(&state, dirty, args.verbose).context("Failed to regenerate Java sources")?;
    println!("Removed advancement: {}", args.id);
    Ok(())
}

fn build_advancement(args: &AdvancementAddArgs) -> crate::state::Advancement {
    crate::state::Advancement {
        id: args.id.clone(),
        parent: args.parent.clone(),
        title: args.title.clone(),
        description: args.description.clone(),
        icon: args.icon.clone(),
        background: args.background.clone(),
        frame: args.frame.clone(),
        show_toast: args.show_toast,
        announce_to_chat: args.announce_to_chat,
        hidden: args.hidden,
        criteria: Vec::new(),
        rewards: None,
    }
}

/// CLI arguments for `fw add advancement`.
#[derive(Parser)]
pub struct AdvancementAddArgs {
    /// Advancement ID
    #[arg(short = 'i', long)]
    pub id: String,

    /// Parent advancement id
    #[arg(long)]
    pub parent: Option<String>,

    /// Display title
    #[arg(long)]
    pub title: String,

    /// Display description
    #[arg(long)]
    pub description: String,

    /// Icon item id
    #[arg(long)]
    pub icon: String,

    /// Background texture path
    #[arg(long)]
    pub background: Option<String>,

    /// Frame type (`task`, `goal`, `challenge`)
    #[arg(long)]
    pub frame: Option<String>,

    /// Show toast
    #[arg(long)]
    pub show_toast: Option<bool>,

    /// Announce to chat
    #[arg(long)]
    pub announce_to_chat: Option<bool>,

    /// Hidden
    #[arg(long)]
    pub hidden: Option<bool>,

    /// Show which files were regenerated, skipped, or pruned
    #[arg(short = 'v', long, default_value_t = false)]
    pub verbose: bool,
}

/// CLI arguments for `fw remove advancement`.
#[derive(Parser)]
pub struct AdvancementRemoveArgs {
    /// Advancement ID to remove.
    #[arg(short = 'i', long)]
    pub id: String,

    /// Show which files were regenerated, skipped, or pruned
    #[arg(short = 'v', long, default_value_t = false)]
    pub verbose: bool,
}
