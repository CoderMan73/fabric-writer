# Architecture

This document provides the full architecture explanation for `fabric-writer`. See
`AGENTS.md` for build/test commands and coding standards.

## Overview

`fabric-writer` is a Rust CLI (edition 2024) that scaffolds Fabric mod projects.
It wraps the Fabric CLI (`fabric init` via Deno) to create the project, then
tracks mod state in `.fw/fabric-writer.yml` and regenerates Java source files
using `genco`. Every `add`/`remove` command loads state → mutates → saves →
calls `regenerate_all()` to emit Java.

## Source Layout

```
src/
├── main.rs          # clap CLI definition + command dispatch
├── lib.rs           # crate root, module declarations, re-exports
├── state.rs         # ModState, Entity, Item, ItemKind, Block, Recipe, load/save
├── imports.rs       # genco Java import helpers (LazyLock-based)
├── java_writer.rs   # regenerate_all() + write() — Java file emission
├── tokengen.rs      # genco quote! templates for Java classes
├── rustfmt.toml
└── commands/
    ├── mod.rs                # module declarations
    ├── advancement.rs        # fw add/remove advancement
    ├── biome.rs              # fw add/remove biome
    ├── block.rs              # fw add/remove block
    ├── creative_tab.rs       # fw add/remove creative tab
    ├── dimension.rs          # fw add/remove dimension, dimension type
    ├── feature.rs            # fw add/remove feature
    ├── init.rs               # fw init — fabric init wrapper
    ├── item.rs               # fw add/remove item
    ├── loot_table.rs         # fw add/remove loot table
    ├── mob.rs                # fw add/remove mob
    ├── recipe.rs             # fw add/remove recipe
    ├── regen.rs              # fw regen — regenerate all Java from state
    ├── run.rs                # fw run datagen/client/server (gradlew)
    ├── save_load.rs          # fw save/load — export/import state
    ├── sound_event.rs        # fw add/remove sound event
    ├── status.rs             # fw status — print mod summary
    ├── structure.rs          # fw add/remove structure
    ├── tag.rs                # fw add/remove tag
    └── test_project.rs       # fw test-project — preset content generation
```

## The State Flow

1. Each `add`/`remove` command calls `state::load()` → reads `.fw/fabric-writer.yml`
2. Mutates the `ModState` (adds/removes from the appropriate collection Vec)
3. Calls `state::save()` → writes YAML back
4. Calls `regenerate_all(&state, dirty, verbose)` → emits Java files via genco

**Key:** `regenerate_all` takes a `DirtyFlags` bitmask so only affected files are rewritten. `fw regen` passes `DirtyFlags::all()`. When a collection becomes empty, its files are **pruned** (deleted). When non-empty, they're **regenerated**.

## The Generated Java Files

`java_writer::regenerate_all()` emits 21 files into `src/main/java/<package>/` and `src/client/java/<package>/client/`:

| # | File | Location | Condition |
|---|---|---|---|
| 1 | `<ModName>.java` | `src/main/java/<package>/` | Always exists |
| 2 | `ModItemIds.java` | `src/main/java/<package>/` | Items exist |
| 3 | `ModItems.java` | `src/main/java/<package>/` | Items exist |
| 4 | `ModBlocks.java` | `src/main/java/<package>/` | Blocks exist |
| 5 | `ModBlockIds.java` | `src/main/java/<package>/` | Blocks exist |
| 6 | `ModBlockItemIds.java` | `src/main/java/<package>/` | Blocks exist |
| 7 | `ModCreativeTabs.java` | `src/main/java/<package>/` | Creative tabs exist |
| 8 | `ModMobs.java` | `src/main/java/<package>/` | Mobs exist |
| 9 | `ModBiomes.java` | `src/main/java/<package>/` | Biomes exist |
| 10 | `ModDimensions.java` | `src/main/java/<package>/` | Dimensions or dimension types exist |
| 11 | `ModStructures.java` | `src/main/java/<package>/` | Structures exist |
| 12 | `ModFeatures.java` | `src/main/java/<package>/` | Features exist |
| 13 | `ModLootTables.java` | `src/main/java/<package>/` | Loot tables exist |
| 14 | `ModAdvancements.java` | `src/main/java/<package>/` | Advancements exist |
| 15 | `ModSoundEvents.java` | `src/main/java/<package>/` | Sound events exist |
| 16 | `ModTags.java` | `src/main/java/<package>/` | Tags exist |
| 17 | `LangProvider.java` | `src/client/java/<package>/client/` | Items or blocks exist |
| 18 | `ModelProvider.java` | `src/client/java/<package>/client/` | Items or blocks exist |
| 19 | `<ModName>DataGenerator.java` | `src/client/java/<package>/client/` | Items or blocks exist |
| 20 | `<ModName>RecipeProvider.java` | `src/client/java/<package>/client/` | Recipes exist |
| 21 | `<ModName>WorldgenProvider.java` | `src/client/java/<package>/client/` | Biomes, features, structures, or dimensions/dimension types exist |

Additionally, one `<ItemId>Item.java` file is generated per item that has tooltip lines (e.g. `MySwordItem.java`).

## genco Code Generation Patterns

- `tokengen.rs` contains `BuildFn` functions (`fn(&ModState) -> Tokens`) that produce Java source via `quote!`
- `java_writer.rs` manages the file emission: `FileSpec` structs define a path, build function, and `should_exist` predicate
- **Dynamic identifiers:** Use `format!()` to build Java identifiers, then interpolate as `String` in `quote!`. genco treats `String` as a literal token (raw text, not a quoted string). Example: `$(format!("Items.{}", mc_constant))` → emits `Items.DIRT`
- **Imports:** Use `Import` types from `imports.rs` for automatic import tracking. Don't `format!` imports — genco won't track them
- **Repeatable blocks:** Use `$(for item in &collection => ... )` for loops, `$(if condition => ... )` for conditionals
- **Line endings:** Append `$['\r']` to emit `System.lineSeparator()` in Java (Windows-friendly)

## State Types Reference

Located in `src/state.rs`. The `ModState` struct is serialized to `.fw/fabric-writer.yml`:

```rust
ModState {
    mod_name: String,           // Human-readable name
    mod_id: String,             // Lowercase, alphanumeric + _ + -
    namespace: String,           // Defaults to mod_id
    package_name: String,       // Lowercase, drops -, keeps _
    minecraft_version: String,  // Currently "26.2"
    advanced_options: Vec<String>, // e.g. ["datagen", "splitSources"]
    java_path: String,          // JDK path for gradle.properties
    items: Vec<Item>,
    blocks: Vec<Block>,
    recipes: Vec<Recipe>,
    creative_tabs: Vec<CreativeTab>,
    mobs: Vec<Mob>,
    biomes: Vec<Biome>,
    dimensions: Vec<Dimension>,
    dimension_types: Vec<DimensionType>,
    structures: Vec<Structure>,
    features: Vec<Feature>,
    loot_tables: Vec<LootTable>,
    advancements: Vec<Advancement>,
    sound_events: Vec<SoundEvent>,
    tags: Vec<Tag>,
}
```

```rust
Block {
    id: String,
    creative_tab: Option<String>,
    model_kind: BlockModelKind,
    properties_from: Option<String>,
    block_class: String,
}
```

```rust
Item {
    id: String,
    kind: ItemKind,  // Basic, Tool, Axe, Shovel, Hoe, Food, SpawnEgg, Fuel,
                    // Compostable, Armor, Shield, Potion, MusicDisc,
                    // FireCharge, FlintAndSteel, Compass
    material: Option<String>,
    attack_damage: Option<f32>,
    attack_speed: Option<f32>,
    durability: Option<i32>,
    tooltip: Vec<String>,
    nutrition: Option<i32>,
    saturation: Option<f32>,
    always_edible: bool,
    entity_type: Option<String>,
    burn_time: Option<i32>,
    compost_chance: Option<f32>,
    creative_tab: Option<String>,
    armor_material: Option<String>,
    armor_slot: Option<String>,
    effects: Vec<PotionEffect>,
}
```

The `Entity` enum has 14 variants: `Item`, `Block`, `Recipe`, `CreativeTab`, `Mob`, `Biome`, `Dimension`, `DimensionType`, `Structure`, `Feature`, `LootTable`, `Advancement`, `SoundEvent`, `Tag`.

`BlockModelKind` has 24 variants: `CubeAll`, `CubeBottomTop`, `Cube`, `CubeColumn`, `Orientable`, `Slab`, `Stairs`, `Wall`, `Fence`, `FenceGate`, `Door`, `Trapdoor`, `Button`, `PressurePlate`, `Cross`, `TintedCross`, `Crop`, `SingleFace`, `Ore`, `Lantern`, `Chain`, `Carpet`, `Ladder`, `Lever`.

## Recipe System

Recipes use `RecipeProvider` (Fabric datagen API) — not raw JSON files.

### Recipe struct

```rust
Recipe {
    id: String,                 // e.g. "my_sword"
    kind: String,               // "crafting_shaped", "crafting_shapeless",
                                // "smelting", "blasting", "smoking",
                                // "campfire_cooking", "stonecutting", "smithing"
    pattern: Vec<String>,       // One string per row (shaped only)
    ingredients: HashMap<String, String>, // "S" -> "minecraft:stick"
    result: String,             // "minecraft:diamond" or "mymod:my_item"
    count: u32,                 // Default: 1
    cooking_time: Option<i32>,  // For cooking recipes (default 200)
    experience: Option<f32>,     // For cooking recipes (default 1.0)
    category: Option<String>, // e.g. "building_blocks", "combat", "food", "MISC"
}
```

### Generated output pattern

```java
shaped(RecipeCategory.MISC, Items.DIAMOND, 1)
    .pattern("S").pattern("S").pattern("S")
    .define('S', Ingredient.of(Items.STICK))
    .unlockedBy(getHasName(Items.DIAMOND), has(Items.DIAMOND))
    .save(exporter, "mymod:my_sword");
```

### CLI

```bash
fw add recipe <id> --kind crafting_shaped|crafting_shapeless|smelting|blasting|smoking|campfire_cooking|stonecutting|smithing \
  --result <item> --count <u32> \
  --pattern <line>... \
  --ingredients <key=value>...
```

- `--pattern` is repeatable — each string is one row of the crafting grid
- `--ingredients` is `key=value` where key is a single char and value is an item ID
- Vanilla items use `minecraft:` prefix (e.g. `minecraft:wood`)
- Mod items use the mod's namespace (e.g. `mymod:ingot`)
- `--cooking-time` and `--experience` are used for cooking recipes
- `--category` sets the recipe category

## ItemKind / Tool Implementation

Items have a `kind` field (`ItemKind` enum with 16 variants). This is **real**, not stubbed:

- `ItemKind::Tool` with `--material` generates `.sword(ToolMaterials.MATERIAL, damage, speed)` in `Item.Properties`
- `ItemKind::Axe` / `Shovel` / `Hoe` generate `.axe()` / `.shovel()` / `.hoe()` calls and use dedicated item classes (`AxeItem`, `ShovelItem`, `HoeItem`)
- `ItemKind::Basic` generates plain `new Item.Properties()`
- `ItemKind::Food` generates `FoodProperties.Builder` with nutrition/saturation
- `ItemKind::SpawnEgg` generates `SpawnEggItem` with `--entity-type`
- `ItemKind::Fuel` registers burn time via `FuelValueEvents`
- `ItemKind::Compostable` registers compost chance via `Composter`
- `ItemKind::Armor` generates `humanoidArmor()` with `--armor-material` and `--armor-slot`
- `ItemKind::Shield` generates `BlocksAttacks` component
- `ItemKind::Potion` generates `PotionContents` with `--effect type=duration:amplifier`
- `ItemKind::MusicDisc`, `FireCharge`, `FlintAndSteel`, `Compass` use corresponding item classes
- `--durability` adds `.durability(N)` override
- Materials are lowercase strings (e.g. `"diamond"`, `"iron"`, `"netherite"`)
- Items with `--tooltip` lines generate a custom item class extending `Item` with `appendHoverText`

The `item_properties()` and `item_factory()` functions in `src/tokengen.rs` handle this logic.
