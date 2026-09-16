use crate::state::{self, ItemKind};
use anyhow::Result;
use clap::Parser;

/// Prints a summary of the current mod project.
pub fn run(args: StatusArgs) -> Result<()> {
    let state = state::load()?;

    println!("Mod: {} ({})", state.mod_name, state.mod_id);
    println!("Package: {}", state.package_name);
    println!("MC: {}", state.minecraft_version);

    print_items(&state.items, args.verbose);
    print_blocks(&state.blocks, args.verbose);
    print_creative_tabs(&state.creative_tabs, args.verbose);
    print_recipes(&state.recipes, args.verbose);

    if args.verbose {
        println!("Advanced options: {:?}", state.advanced_options);
    }

    Ok(())
}

fn print_items(items: &[state::Item], verbose: bool) {
    println!("Items: {}", items.len());
    for item in items {
        if verbose {
            print_item(item);
        } else {
            println!("  - {}", item.id);
        }
    }
}

fn print_item(item: &state::Item) {
    let kind = match item.kind {
        ItemKind::Basic => "basic",
        ItemKind::Tool => "tool",
        ItemKind::Axe => "axe",
        ItemKind::Shovel => "shovel",
        ItemKind::Hoe => "hoe",
        ItemKind::Food => "food",
        ItemKind::SpawnEgg => "spawn_egg",
        ItemKind::Fuel => "fuel",
        ItemKind::Compostable => "compostable",
    };
    let mut detail = format!("  - {} ({kind}", item.id);
    if let Some(material) = &item.material {
        detail.push_str(format!(", material={material}").as_str());
    }
    if let Some(damage) = item.attack_damage {
        detail.push_str(format!(", damage={damage}").as_str());
    }
    if let Some(speed) = item.attack_speed {
        detail.push_str(format!(", speed={speed}").as_str());
    }
    if let Some(durability) = item.durability {
        detail.push_str(format!(", durability={durability}").as_str());
    }
    if let Some(burn_time) = item.burn_time {
        detail.push_str(format!(", burn_time={burn_time}").as_str());
    }
    if let Some(compost_chance) = item.compost_chance {
        detail.push_str(format!(", compost_chance={compost_chance}").as_str());
    }
    if !item.tooltip.is_empty() {
        detail.push_str(format!(", tooltip_lines={}", item.tooltip.len()).as_str());
    }
    let tab = item.creative_tab.as_deref().unwrap_or("default");
    detail.push_str(format!(", tab={tab}").as_str());
    detail.push(')');
    println!("{detail}");
}

fn print_blocks(blocks: &[state::Block], verbose: bool) {
    println!("Blocks: {}", blocks.len());
    if verbose {
        for block in blocks {
            let tab = block.creative_tab.as_deref().unwrap_or("default");
            println!("  - {} (block, tab={})", block.id, tab);
        }
    } else {
        for block in blocks {
            println!("  - {}", block.id);
        }
    }
}

fn print_creative_tabs(tabs: &[state::CreativeTab], verbose: bool) {
    println!("Creative Tabs: {}", tabs.len());
    if verbose {
        for tab in tabs {
            println!("  - {} (creative_tab)", tab.id);
        }
    } else {
        for tab in tabs {
            println!("  - {}", tab.id);
        }
    }
}

fn print_recipes(recipes: &[state::Recipe], verbose: bool) {
    println!("Recipes: {}", recipes.len());
    for recipe in recipes {
        if verbose {
            print_recipe(recipe);
        } else {
            println!("  - {} ({})", recipe.id, recipe.kind);
        }
    }
}

fn print_recipe(recipe: &state::Recipe) {
    print!("  - {} ({})", recipe.id, recipe.kind);
    if !recipe.pattern.is_empty() {
        let grid = recipe.pattern.join(" / ");
        print!(" pattern=[{grid}]");
    }
    if !recipe.ingredients.is_empty() {
        let keys: Vec<String> = recipe
            .ingredients
            .keys()
            .map(|k| format!("{k}={}", recipe.ingredients[k]))
            .collect();
        print!(" ingredients=[{}]", keys.join(", "));
    }
    println!(" -> {} x{}", recipe.result, recipe.count);
}

/// CLI arguments for `fw status`.
#[derive(Parser)]
pub struct StatusArgs {
    /// Print verbose status
    #[arg(short = 'v', long)]
    pub verbose: bool,
}
