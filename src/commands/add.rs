use crate::commands::file_util;
use crate::state::{self, Item, ModState};
use anyhow::{bail, Context};
use clap::Parser;
use genco::prelude::*;
use std::path::PathBuf;

#[derive(Parser)]
pub struct ItemAddArgs {
    pub id: String,
}

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

    let item = Item {
        id: args.id.clone(),
        material: "minecraft:air".to_string(),
        damage: 0,
        durability: 0,
    };

    state.items.push(item.clone());

    let state_path = PathBuf::from(".fw/fabric-writer.yml");
    state::save_to(&state_path, &state)?;

    let java_root = PathBuf::from("src/main/java").join(state.package_name.replace('.', "/"));

    ensure_java_template(
        &java_root,
        "ModItemIds.java",
        "templates/java/ModItemIds.java",
        &state,
    )?;
    ensure_java_template(
        &java_root,
        "ModItems.java",
        "templates/java/ModItems.java",
        &state,
    )?;
    ensure_java_template(
        &java_root,
        &format!("{}.java", state.mod_name),
        "templates/java/__MOD_CLASS__.java",
        &state,
    )?;

    let mut item_id_entries = java::Tokens::new();
    for i in &state.items {
        let name = to_upper(i);
        let id = i.id.as_str();
        quote_in!(item_id_entries => public static final ResourceKey<Item> $name = create($id););
    }
    let item_id_content = item_id_entries.to_file_string()?;

    let mut item_entries = java::Tokens::new();
    for i in &state.items {
        let name = to_upper(i);
        quote_in!(item_entries => public static final Item $name = register(ModItemIds.$&name, Item::new, new Item.Properties()););
    }
    let item_content = item_entries.to_file_string()?;

    let item_ids_path = java_root.join("ModItemIds.java");
    let items_path = java_root.join("ModItems.java");

    file_util::write_managed_section(&item_ids_path, &item_id_content)
        .context("Failed to update ModItemIds.java")?;
    file_util::write_managed_section(&items_path, &item_content)
        .context("Failed to update ModItems.java")?;

    println!("Added item: {}", item.id);

    Ok(())
}

fn to_upper(item: &Item) -> String {
    item.id.to_uppercase().replace('-', "_")
}

fn ensure_java_template(
    dir: &PathBuf,
    filename: &str,
    template_rel: &str,
    state: &ModState,
) -> anyhow::Result<()> {
    let dest = dir.join(filename);
    if dest.exists() {
        return Ok(());
    }

    let template_path = PathBuf::from(template_rel);
    let mut content = std::fs::read_to_string(&template_path)
        .with_context(|| format!("Failed to read template {}", template_rel))?;

    content = content.replace("__PACKAGE__", &state.package_name);
    content = content.replace("__MOD_CLASS__", &state.mod_name);
    content = content.replace("__MOD_ID__", &state.mod_id);

    std::fs::write(&dest, content).with_context(|| format!("Failed to write {}", dest.display()))?;
    Ok(())
}
