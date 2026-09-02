use crate::java_writer::regenerate_all;
use crate::state::{self, Entity, Recipe};
use anyhow::Result;
use clap::Parser;

pub fn add(args: RecipeAddArgs) -> Result<()> {
    let mut state = state::load()?;

    let recipe = build_recipe(args)?;
    let recipe_label = format!("{} ({})", recipe.id, recipe.kind);
    state.add(Entity::Recipe(recipe))?;
    state.save()?;
    regenerate_all(&state)?;
    println!("Added recipe: {}", recipe_label);
    Ok(())
}

fn build_recipe(args: RecipeAddArgs) -> Result<Recipe> {
    let mut recipe = Recipe::new(&args.id)?;
    if let Some(kind) = args.kind {
        recipe.kind = kind;
    }
    recipe.result = args.result.unwrap_or_default();
    recipe.count = args.count.unwrap_or(1);
    Ok(recipe)
}

pub fn remove(args: RecipeRemoveArgs) -> Result<()> {
    let mut state = state::load()?;

    state.remove(Entity::Recipe(Recipe::new(&args.id)?))?;
    state.save()?;
    regenerate_all(&state)?;

    println!("Removed recipe: {}", args.id);
    Ok(())
}

#[derive(Parser)]
pub struct RecipeAddArgs {
    pub id: String,
    #[arg(long)]
    pub kind: Option<String>,
    #[arg(long)]
    pub result: Option<String>,
    #[arg(long)]
    pub count: Option<u32>,
}

#[derive(Parser)]
pub struct RecipeRemoveArgs {
    pub id: String,
}
