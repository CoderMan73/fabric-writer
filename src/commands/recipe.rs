use crate::java_writer::{DirtyFlags, regenerate_all};
use crate::state::{self, Entity, Recipe};
use anyhow::Result;
use clap::Parser;

/// Adds a recipe to the mod state and regenerates Java sources.
pub fn add(args: RecipeAddArgs) -> Result<()> {
    let verbose = args.verbose;
    let mut state = state::load()?;
    let recipe = build_recipe(&args)?;
    let entity = Entity::Recipe(recipe.clone());
    let recipe_label = format!("{} ({})", recipe.id, recipe.kind);
    let dirty = DirtyFlags::from_entity(&entity);
    state.add(entity)?;
    state.save()?;
    regenerate_all(&state, dirty, verbose)?;
    println!("Added recipe: {}", recipe_label);
    Ok(())
}

fn build_recipe(args: &RecipeAddArgs) -> Result<Recipe> {
    let mut recipe = Recipe::new(&args.id)?;
    if let Some(kind) = &args.kind {
        recipe.kind = kind.clone();
    }
    recipe.result = args.result.clone().unwrap_or_default();
    recipe.count = args.count.unwrap_or(1);
    Ok(recipe)
}

/// Removes a recipe from the mod state and regenerates Java sources.
pub fn remove(args: RecipeRemoveArgs) -> Result<()> {
    let verbose = args.verbose;
    let mut state = state::load()?;
    let entity = Entity::Recipe(Recipe::new(&args.id)?);
    let dirty = DirtyFlags::from_entity(&entity);
    state.remove(entity)?;
    state.save()?;
    regenerate_all(&state, dirty, verbose)?;
    println!("Removed recipe: {}", args.id);
    Ok(())
}

/// CLI arguments for `fw add recipe`.
#[derive(Parser)]
pub struct RecipeAddArgs {
    /// Recipe ID.
    pub id: String,

    /// Recipe type (e.g. `crafting_shaped`).
    #[arg(long)]
    pub kind: Option<String>,

    /// Result item ID.
    #[arg(long)]
    pub result: Option<String>,

    /// Number of items produced.
    #[arg(long)]
    pub count: Option<u32>,

    /// Show which files were regenerated, skipped, or pruned
    #[arg(short = 'v', long, default_value_t = false)]
    pub verbose: bool,
}

/// CLI arguments for `fw remove recipe`.
#[derive(Parser)]
pub struct RecipeRemoveArgs {
    /// Recipe ID to remove.
    pub id: String,

    /// Show which files were regenerated, skipped, or pruned
    #[arg(short = 'v', long, default_value_t = false)]
    pub verbose: bool,
}
