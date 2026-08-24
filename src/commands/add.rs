use crate::state::{self, Item};
use anyhow::bail;
use clap::Parser;
use std::path::PathBuf;

#[derive(Parser)]
pub struct ItemAddArgs {
    pub id: String,
    #[arg(long)]
    pub material: String,
    #[arg(long)]
    pub damage: i32,
    #[arg(long)]
    pub durability: i32,
}

//TODO: actually makle this work https://docs.fabricmc.net/develop/items/first-item
pub fn run_item(args: ItemAddArgs) -> anyhow::Result<()> {
    let mut state = match state::load() {
        Ok(s) => s,
        Err(_) => {
            bail!("A fabric-writer project doesn't seem to exist here.\n\
                   Did you forget to cd into your project after initiation?\n\
                   Are you running from the correct directory?\n\
                   Please either move to the correct directory, or initialize a new project, and try again.")
        }
    };

    state.items.push(Item {
        id: args.id,
        material: args.material,
        damage: args.damage,
        durability: args.durability,
    });

    let state_path = PathBuf::from(".fw/fabric-writer.yml");
    state::save_to(&state_path, &state)?;

    println!("Item added. Regeneration not yet implemented.");

    Ok(())
}
