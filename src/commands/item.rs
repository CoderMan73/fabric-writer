use crate::java_writer::{DirtyFlags, regenerate_all};
use crate::state::{self, Entity, Item, ItemKind};
use anyhow::Result;
use clap::Parser;

/// Adds an item to the mod state and regenerates Java sources.
pub fn add(args: ItemAddArgs) -> Result<()> {
    let verbose = args.verbose;
    let mut state = state::load()?;
    let item = build_item(&args)?;
    let entity = Entity::Item(item.clone());
    let item_label = format!("{} ({})", item.id, kind_label(&item));
    let dirty = DirtyFlags::from_entity(&entity);
    state.add(entity)?;
    state.save()?;
    regenerate_all(&state, dirty, verbose)?;
    println!("Added item: {}", item_label);
    Ok(())
}

/// Removes an item from the mod state and regenerates Java sources.
pub fn remove(args: ItemRemoveArgs) -> Result<()> {
    let verbose = args.verbose;
    let mut state = state::load()?;
    let entity = Entity::Item(Item::new(&args.id)?);
    let dirty = DirtyFlags::from_entity(&entity);
    state.remove(entity)?;
    state.save()?;
    regenerate_all(&state, dirty, verbose)?;
    println!("Removed item: {}", args.id);
    Ok(())
}

fn build_item(args: &ItemAddArgs) -> Result<Item> {
    let mut item = Item::new(&args.id)?;
    if let Some(kind) = &args.kind {
        item.kind = match kind.to_lowercase().as_str() {
            "tool" => ItemKind::Tool,
            _ => ItemKind::Basic,
        };
    }
    item.material = args.material.clone();
    item.attack_damage = args.attack_damage;
    item.attack_speed = args.attack_speed;
    item.durability = args.durability;
    Ok(item)
}

fn kind_label(item: &Item) -> &'static str {
    match item.kind {
        ItemKind::Basic => "basic",
        ItemKind::Tool => "tool",
    }
}

/// CLI arguments for `fw add item`.
#[derive(Parser)]
pub struct ItemAddArgs {
    /// Item ID
    #[arg(short = 'i', long)]
    pub id: String,

    /// Item kind: `tool` or `basic` (defaults to `basic`).
    #[arg(long)]
    pub kind: Option<String>,

    /// Tool material (e.g. `diamond`); None for basic items.
    #[arg(long)]
    pub material: Option<String>,

    /// Attack damage for tool items.
    #[arg(long)]
    pub attack_damage: Option<f32>,

    /// Attack speed for tool items.
    #[arg(long)]
    pub attack_speed: Option<f32>,

    /// Durability for tool items (overrides material default).
    #[arg(long)]
    pub durability: Option<i32>,

    /// Show which files were regenerated, skipped, or pruned
    #[arg(short = 'v', long, default_value_t = false)]
    pub verbose: bool,
}

/// CLI arguments for `fw remove item`.
#[derive(Parser)]
pub struct ItemRemoveArgs {
    /// Item ID to remove.
    #[arg(short = 'i', long)]
    pub id: String,

    /// Show which files were regenerated, skipped, or pruned
    #[arg(short = 'v', long, default_value_t = false)]
    pub verbose: bool,
}
