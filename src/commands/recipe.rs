use crate::java_writer::{DirtyFlags, regenerate_all};
use crate::state::{self, Entity, Recipe};
use anyhow::{Result, bail};
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
    recipe.pattern = args.pattern.clone();
    for pair in &args.ingredients {
        let (key, value) = parse_ingredient_pair(pair)?;
        recipe.ingredients.insert(key, value);
    }
    Ok(recipe)
}

fn parse_ingredient_pair(pair: &str) -> Result<(String, String)> {
    let Some(idx) = pair.find('=') else {
        bail!("Ingredient pair '{}' must be in key=value format", pair);
    };
    let key = pair[..idx].trim().to_string();
    let value = pair[idx + 1..].trim().to_string();
    if key.is_empty() || value.is_empty() {
        bail!("Ingredient pair '{}' has an empty key or value", pair);
    }
    Ok((key, value))
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
#[command(after_help = "\
Examples:
  # Shaped recipe: 3 sticks in a T-pattern -> 1 iron_sword
  fw add recipe craft_iron_sword --kind crafting_shaped --result iron_sword \\
      --pattern \"S S\" --pattern \" S \" --pattern \" S \" \\
      --ingredients S=minecraft:stick

  # Shapeless recipe: copper_ingot + diamond -> 1 enchanted_apple
  fw add recipe enchanted_apple --kind crafting_shapeless --result minecraft:enchanted_golden_apple \\
      --ingredients C=copper_ingot --ingredients D=minecraft:diamond

  # Vanilla result with multiple ingredient types
  fw add recipe wood_apple --kind crafting_shaped --result minecraft:apple \\
      --pattern \"WWW\" --pattern \" S \" --pattern \" S \" \\
      --ingredients W=minecraft:logs --ingredients S=minecraft:stick

  Pattern notes:
  - Each --pattern line is one row of the crafting grid
  - Each unique character in the pattern is an ingredient key
  - --ingredients maps key=value where value is minecraft:<id> or <modid>:<id>")]
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

    /// Crafting pattern lines (e.g. `--pattern "WWW"` repeated).
    #[arg(long)]
    pub pattern: Vec<String>,

    /// Ingredient key→value mapping (e.g. `--ingredients W=minecraft:logs`).
    #[arg(long)]
    pub ingredients: Vec<String>,

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
