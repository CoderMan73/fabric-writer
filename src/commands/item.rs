use crate::commands::write_fns::regenerate_all;
use crate::state::{self, Entity, Item, ItemKind};
use anyhow::Result;
use clap::Parser;

pub fn add(args: ItemAddArgs) -> Result<()> {
    let mut state = state::load()?;

    let item = build_item(args)?;
    let item_label = format!("{} ({})", item.id, kind_label(&item));
    state.add(Entity::Item(item))?;
    state.save()?;
    regenerate_all(&state)?;
    println!("Added item: {}", item_label);
    Ok(())
}

pub fn remove(args: ItemRemoveArgs) -> Result<()> {
    let mut state = state::load()?;

    state.remove(Entity::Item(Item::new(&args.id)?))?;
    state.save()?;
    regenerate_all(&state)?;

    println!("Removed item: {}", args.id);
    Ok(())
}

fn build_item(args: ItemAddArgs) -> Result<Item> {
    let mut item = Item::new(&args.id)?;
    if let Some(kind) = args.kind {
        item.kind = match kind.to_lowercase().as_str() {
            "tool" => ItemKind::Tool,
            _ => ItemKind::Basic,
        };
    }
    item.material = args.material;
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

#[derive(Parser)]
pub struct ItemAddArgs {
    pub id: String,
    #[arg(long)]
    pub kind: Option<String>,
    #[arg(long)]
    pub material: Option<String>,
    #[arg(long)]
    pub attack_damage: Option<f32>,
    #[arg(long)]
    pub attack_speed: Option<f32>,
    #[arg(long)]
    pub durability: Option<i32>,
}

#[derive(Parser)]
pub struct ItemRemoveArgs {
    pub id: String,
}
