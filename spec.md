# fabric-writer (fw) — Specification

## Overview

`fabric-writer` is a Rust CLI (edition 2024) that scaffolds Fabric mod projects. It wraps the official Fabric CLI (`fabric init` via Deno) to create the project structure, then tracks generated content in `.fw/fabric-writer.yml` so subsequent commands can regenerate Java sources automatically. Every `add`/`remove` command follows a consistent flow: load state → mutate → save → call `regenerate_all()` to emit Java via `genco`.

## Problem

Creating even a basic Fabric mod project requires running the official scaffold tool, then manually tracking metadata and boilerplate registry classes so subsequent commands can extend the project consistently.

## Solution

`fabric-writer` wraps `fabric init` (via `deno run -A https://fabricmc.net/cli init`) to create the project, derives modid/package/namespace from the human-readable name, validates Java version, injects `org.gradle.java.home` into `gradle.properties`, and writes `.fw/fabric-writer.yml` so the project state is explicit. Later commands mutate state and call `java_writer::regenerate_all()` to regenerate all Java sources.

## Current Status (v0.1.0)

### Done

- `fw init` — create/validate Fabric project scaffold via deno + fabric CLI
- `fw save` / `fw load` — export/import full project state to/from a YAML file, with validation of duplicate IDs and recipe references
- `fw test-project` — generate a complete test project with `--preset blue-nether-base` or `--preset default`, with optional `--reset`
- State file: `.fw/fabric-writer.yml` inside the mod project root (`.fw/` dir)
- Name derivation: `mod_id` keeps `_` and `-`; `package_name` drops `-`
- Version validation: only `26.2` accepted unless `--dangerous` is passed
- Java validation: `MIN_JAVA_BY_VERSION` map (`26.2` → Java 25); `--java-path` required
- Advanced options: defaults to `["datagen", "splitSources"]` if no `-o` flags given
- `--dir` flag: create the mod folder inside a chosen parent directory
- `gradle.properties` injection: `org.gradle.java.home=<java_path>`
- **Entity system** — `Entity` enum with 14 variants (`Item`, `Block`, `Recipe`, `CreativeTab`, `Mob`, `Biome`, `Dimension`, `DimensionType`, `Structure`, `Feature`, `LootTable`, `Advancement`, `SoundEvent`, `Tag`), each with add/remove commands, dirty-flag derivation, and codegen
- **Item add/remove** — 16 `ItemKind` variants: `basic`, `tool`, `axe`, `shovel`, `hoe`, `food`, `spawn_egg`, `fuel`, `compostable`, `armor`, `shield`, `potion`, `music_disc`, `fire_charge`, `flint_and_steel`, `compass`; supports `--material`, `--attack-damage`, `--attack-speed`, `--durability`, `--nutrition`, `--saturation`, `--always-edible`, `--entity-type`, `--burn-time`, `--compost-chance`, `--tooltip`, `--creative-tab`, `--armor-material`, `--armor-slot`, `--effect`
- **Block add/remove** — `--model-kind` (3 CLI variants: `CubeAll`, `CubeBottomTop`, `Cube`), `--properties-from` (vanilla block to copy `BlockBehaviour.Properties` from), `--creative-tab`; 24 `BlockModelKind` variants in state; auto-creates `BlockItem` with `useBlockDescriptionPrefix().setId()`
- **Recipe add/remove** — `--kind` (8 types: `crafting_shaped`, `crafting_shapeless`, `smelting`, `blasting`, `smoking`, `campfire_cooking`, `stonecutting`, `smithing`), `--result`, `--count`, `--pattern` (repeatable), `--ingredients` (key=value pairs), `--cooking-time`, `--experience`, `--category`; generates `<ModName>RecipeProvider.java` extending `FabricRecipeProvider`
- `fw regen` — regenerate all Java from current state (alias: `g`)
- `fw status` — print mod summary; `-v` for verbose (alias: `s`)
- `fw run datagen|client|server` — exec gradlew tasks (aliases: `d`/`c`/`s`)
- Java codegen via `genco` — emits 22 source files (all listed in the [Java Code Generation](#java-code-generation) section)
- Item tools: sword/material/damage/speed/durability support in `item_properties()`; axe/shovel/hoe use dedicated item classes (`AxeItem`, `ShovelItem`, `HoeItem`); armor uses `humanoidArmor()`; shields use `BlocksAttacks`; potions use `PotionContents`; food uses `FoodProperties`
- Block items: auto-registered as `BlockItem` with `useBlockDescriptionPrefix().setId()`
- Placeholder assets: `placeholder_item.png` / `placeholder_block.png` copied into `assets/<mod_id>/textures/{block,item}/` during `regenerate_all`, embedded via `include_bytes!`
- **Creative tabs** — `ModCreativeTabs.java` with icon, title, and display items; items/blocks auto-assigned to first tab or `ingredients`
- **Texture tinting pipeline** — `tint_texture()` in `test_project.rs` copies vanilla textures, converts to grayscale via Rec. 601 luminance (`0.30*R + 0.59*G + 0.11*B`), tints to blue palette `#5078FF` with `result[i] = gray[i] * tint[i] / 255`, preserves RGBA transparency
- **Sound events** — `ModSoundEvents.java` generates methods for each sound event; placeholder `empty.ogg` files copied for each sound
- **Tags** — `ModTags.java` generates `TagKey` creation methods for block/item/entity/fluid tags
- **Mobs** — `ModMobs.java` with entity type registration, spawn eggs, attributes, and drops
- **Biomes** — `ModBiomes.java` with `ResourceKey<Biome>` registration; JSON generation for biome JSON resources (climate settings, mob spawns, feature integration)
- **Dimensions & dimension types** — `ModDimensions.java` with `ResourceKey<LevelStem>` and `ResourceKey<DimensionType>` registration; JSON generation for dimension type JSONs and dimension JSONs; `<ModName>PortalBlock.java` generated for custom portal blocks that teleport entities to custom dimensions by extending `NetherPortalBlock`
- **Portal block** — `<ModName>PortalBlock.java` generated when a dimension has `portal_frame` and `portal_igniter`; extends `NetherPortalBlock`, overrides `getPortalDestination` and `getPortalTransitionTime` for custom dimension teleportation
- **Structures** — `ModStructures.java` with `ResourceKey<StructureSet>` registration; JSON generation for structure set resources
- **Features** — `ModFeatures.java` with `ResourceKey<ConfiguredFeature>` and `ResourceKey<PlacedFeature>` registration; JSON generation for configured and placed feature resources
- **Loot tables** — `ModLootTables.java` with `ResourceKey<LootTable>` registration; JSON generation for chest and entity loot table resources; entity loot tables auto-generated from `Mob.drops` field
- **Entity loot tables** — entity loot table JSON files generated from `Mob.drops` in `generate_loot_table_resources()`, producing `data/<mod>/loot_tables/entities/<mob>.json` for mobs with non-empty drops
- **Advancements** — `ModAdvancements.java` with `ResourceKey<Advancement>` registration; JSON generation for advancement JSONs
- **Worldgen provider** — `<ModName>WorldgenProvider.java` extends `FabricDynamicRegistryProvider` for biome/feature/structure/dimension registration
- **Dirty-file tracking** — `regenerate_all` takes a `DirtyFlags` bitmask and a `verbose: bool`. Only files whose dirty bit is set are regenerated; the rest are left untouched (preserving mtime). Each add/remove command derives dirty flags from the mutated entity via `DirtyFlags::from_entity()`. `fw regen` bypasses the check by passing `DirtyFlags::all()`. A `--verbose` / `-v` flag on add/remove/regen commands prints which files were `wrote` / `skipped` / `pruned`. Additionally, `write()` now compares new content against the existing file and skips the write if unchanged (belt-and-suspenders). Lazy file existence: files for items (ModItems, ModItemIds), blocks (ModBlocks, ModBlockIds, ModBlockItemIds), data-provider classes (LangProvider, ModelProvider, DataGenerator), and all entity-specific files (CreativeTabs, Mobs, Biomes, Dimensions, Structures, Features, LootTables, Advancements, SoundEvents, Tags, WorldgenProvider) are automatically pruned when the underlying collection goes empty, and only regenerated when dirtied. The main mod class always exists. Placeholder textures are likewise pruned when their corresponding item/block is removed.
- **Rustdoc conventions** — `#![warn(rustdoc::all, missing_docs)]` added to crate roots (`lib.rs`, `main.rs`); crate-level `//!` docs, `///` docs on all `pub` items and fields, module-level `///` on `pub mod` declarations, intra-doc links, and `#![allow(missing_docs)]` on test files. `cargo doc --no-deps` and `cargo clippy ... -W rustdoc::all -W missing_docs` both pass with zero warnings. AGENTS.md updated with the full rustdoc linting workflow. See the AGENTS.md "Rustdoc" section.

### In progress / next

- **Blue Nether mod implementation** — Extend `fabric-writer` codegen to support the full content set defined in `blue_nether_spec.md`. This includes remaining block types, item kinds, recipe kinds, mob/entity types, loot tables, biomes, dimension type, structures, advancements, sounds, and tag codegen. See the "Blue Nether Mod" addendum below.
- `fw test-project --preset blue-nether-base` currently adds the full Blue Nether content set (14 base terrain blocks, 22 variant blocks, 6 flora/nylium blocks, 5 special blocks, 31 items, 12 mobs, 13 recipes, 1 dimension, 5 biomes, 4 structures, 6 features, 8 sound events, 1 chest loot table, 12 entity loot tables, 1 tag, 10 advancements). Biomes, advancements, features, and entity loot tables are complete; dimension portal ignition Java codegen and mob AI remain in progress.

## Blue Nether Mod

The current active development target for `fabric-writer` is implementing the
content spec defined in:

`blue_nether_spec.md` (alongside this file in the repository root)

That document is the source of truth for the Blue Nether mod content. The
implementation plan below maps each spec section to the codegen work required
in `fabric-writer`.

### Completed codegen support

- Block add/remove with `--model-kind` and `--properties-from`
- Base terrain block preset + texture copy workflow with runtime tinting (`tint_texture()` at `src/commands/test_project.rs:229`)
- Item kinds: `basic`, `tool`, `axe`, `shovel`, `hoe`, `food`, `spawn_egg`, `fuel`, `compostable`, `armor`, `shield`, `potion`, `music_disc`, `fire_charge`, `flint_and_steel`, `compass`
- Recipes: `crafting_shaped`, `crafting_shapeless`, `smelting`, `blasting`, `smoking`, `campfire_cooking`, `stonecutting`, `smithing`
- Block model kinds: `CubeAll`, `CubeBottomTop`, `Cube`, `CubeColumn`, `Orientable`, `Slab`, `Stairs`, `Wall`, `Fence`, `FenceGate`, `Door`, `Trapdoor`, `Button`, `PressurePlate`, `Cross`, `TintedCross`, `Crop`, `SingleFace`, `Ore`, `Lantern`, `Chain`, `Carpet`, `Ladder`, `Lever`
- Entity enum: `Item`, `Block`, `Recipe`, `CreativeTab`, `Mob`, `Biome`, `Dimension`, `DimensionType`, `Structure`, `Feature`, `LootTable`, `Advancement`, `SoundEvent`, `Tag`
- Dirty-file tracking, placeholder assets, creative tabs, datagen + splitSources
- Sound event registration with JSON resource generation
- Tag registration (`TagKey`) with JSON resource generation
- Loot table codegen with JSON resource generation
- Advancement codegen with JSON resource generation
- Biome codegen with JSON resource generation
- Dimension/dimension type codegen with JSON resource generation
- Structure/structure set codegen with JSON resource generation
- Feature/configured feature/placed feature codegen with JSON resource generation
- Worldgen provider (`FabricDynamicRegistryProvider`) for biome modifiers, carvers, features, structures, dimension types
- Texture tinting pipeline (grayscale conversion + blue palette tint)
- Item model parent selection (`minecraft:item/generated` for basic, `minecraft:item/handheld` for tools, `minecraft:item/template_spawn_egg` for spawn eggs)

### Remaining codegen work by spec step

| Spec step | Name | Required codegen additions |
| --- | --- | --- |
| Step 1 | The Blue Nether Dimension | `Entity::Dimension` / `DimensionType` state + Java datagen template for `DimensionType` registration and dimension entrypoint — PARTIAL (dimension registered in datagen but portal ignition/teleportation logic requires manual Java) |
| Step 2 | Blue Obsidian / Portal | Block model / property support; portal ignition behavior likely requires manual Java, but frame/portal blocks can be generated — PARTIAL |
| Step 3 | Base terrain | Done via `blue-nether-base` preset |
| Step 4 | Biomes | `Entity::Biome` / `Biome` state + Java datagen templates for `Biome` registration + `BiomeSource` / `MultiNoiseBiomeSourceParameterList` — PARTIAL (1 biome registered, 4 remaining) |
| Step 5 | Blocks | All 24 `BlockModelKind` variants supported |
| Step 6 | Items | All 16 `ItemKind` variants supported |
| Step 7 | Mobs | `Entity::Mob` with spawn egg, entity type, attributes, drops, equipment, behavior — PARTIAL (12 mobs registered but no AI, spawn rules, or drop tables) |
| Step 8 | Structures | `Entity::Structure` / `Structure` state + Java datagen for structure set + template pool registration, or manual JSON + `StructureFeature` registration — PARTIAL |
| Step 9 | Terrain features | `Entity::Feature` / `Feature` state + Java datagen for configured feature / placed feature registration — PARTIAL (1 feature registered, 5 remaining) |
| Step 10 | Fog / Particles / Lighting | Biome fog color support in biome generation; particle/sound mapping likely manual — TODO |
| Step 11 | Flora | Covered by additional `BlockModelKind` variants (`Cross`, `TintedCross`, `CubeColumn`, etc.) |
| Step 12 | Audio | `Entity::SoundEvent` / `SoundEvent` state + Java registration + asset copy workflow for sound JSONs and OGG files — PARTIAL (8 sound events registered with placeholder `empty.ogg` files) |
| Step 13 | Loot | `Entity::LootTable` / `LootTable` state + Java datagen for `LootTable` provider — PARTIAL (1 loot table registered) |
| Step 14 | Recipes | All 8 recipe kinds supported |
| Step 15 | Advancements | `Entity::Advancement` / `Advancement` state + Java datagen for advancement tree — PARTIAL (1 advancement registered, 9 remaining) |

### Implementation priority

| Priority | Task | Status | Section reference |
|---|---|---|---|
| P1 | Fix creative tab icon model warning (texture rendering resolution) | DONE | Step 3.1 |
| P2 | Add remaining biomes with MobSpawn rules | DONE | Spec Step 4 |
| P3 | Add terrain feature codegen (configured/placed features) | DONE | Spec Step 9 |
| P4 | Add mob spawn rules, drop tables | DONE | Spec Step 7 |
| P5 | Add remaining 9 advancements | DONE | Spec Step 15 |
| P6 | Add dimension type JSON and portal ignition logic | DONE | Spec Steps 1-2 |
| P7 | Integrate real vanilla sound files | DONE | Spec Step 12 |
| P8 | Add structure schematics (fortress, bastion) | TODO | Spec Step 8 |
| P9 | Add mob AI behaviors | TODO | Spec Step 7 |

## Spec Workflow

This file tracks feature status in four distinct sections:

- **Done** — completed features that are fully implemented and verified.
- **In progress / next** — features currently being worked on or queued for implementation.
- **Non-Goals** — features explicitly out of scope for the current release.
- **Speculative concepts** — ideas being considered but not yet planned; may never be implemented.

When completing a feature, move it from "In progress / next" to "Done". When an idea moves from speculative to planned, move it to "In progress / next" and remove it from "Speculative concepts". Never have the same idea in two sections at once.

## Tech Stack

- **Rust** — CLI binary (edition 2024)
- `cargo` / `rustfmt` — build + formatting
- `clap` 4 — CLI argument parsing (derive API)
- `serde` + `serde_yaml` — state file serialization
- `genco` — Java code generation (Tokens, `quote!`, `Java` config)
- `anyhow` — error handling
- `which` — used by `init` to verify `deno` is installed
- `heck` — case conversion (e.g. `ToTitleCase` for display names)
- `clap-markdown` — markdown command reference generation (`--markdown-help`)
- `image` — texture processing (grayscale conversion + tinting)
- `tempfile` + `serial_test` + `dotenvy` — dev/test dependencies

## File Structure

```
fabric-writer/
├── Cargo.toml
├── Cargo.lock
├── src/
│   ├── main.rs                   # clap CLI definition + command dispatch
│   ├── lib.rs                    # crate root, module declarations, re-exports
│   ├── state.rs                  # ModState, Entity, Item, ItemKind, Block, Recipe, load/save
│   ├── imports.rs                # genco Java import helpers (LazyLock-based)
│   ├── java_writer.rs            # regenerate_all() + write() — Java file emission
│   ├── tokengen.rs               # genco quote! templates for Java classes
│   ├── rustfmt.toml
│   └── commands/
│       ├── mod.rs                # module declarations
│       ├── advancement.rs        # fw add/remove advancement
│       ├── biome.rs              # fw add/remove biome
│       ├── block.rs              # fw add/remove block
│       ├── creative_tab.rs       # fw add/remove creative tab
│       ├── dimension.rs          # fw add/remove dimension, dimension type
│       ├── feature.rs            # fw add/remove feature
│       ├── init.rs               # fw init — fabric init wrapper
│       ├── item.rs               # fw add/remove item
│       ├── loot_table.rs         # fw add/remove loot table
│       ├── mob.rs                # fw add/remove mob
│       ├── recipe.rs             # fw add/remove recipe
│       ├── regen.rs              # fw regen — regenerate all Java from state
│       ├── run.rs                # fw run datagen/client/server (gradlew)
│       ├── save_load.rs          # fw save/load — export/import state
│       ├── sound_event.rs        # fw add/remove sound event
│       ├── status.rs             # fw status — print mod summary
│       ├── structure.rs          # fw add/remove structure
│       ├── tag.rs                # fw add/remove tag
│       └── test_project.rs       # fw test-project — preset content generation
├── tests/
│   ├── tests.rs
│   └── common/
│       └── mod.rs
├── testing/
│   ├── TestMod/                 # Cached test project fixture
│   ├── blue_nether_test.log     # Validation results log
│   └── test-blue-nether.ps1 # Automated validation script
├── .github/
│   └── workflows/
│       └── ci.yml
├── .env.example
├── ci.bat
├── AGENTS.md
├── spec.md
├── README.md
└── LICENSE.md
```

## State File Schema

```yaml
# .fw/fabric-writer.yml
mod_name: MyDopeMod
mod_id: mydopemod
namespace: mydopemod
package_name: mydopemod
minecraft_version: "26.2"
advanced_options:
  - datagen
  - splitSources
java_path: "C:/Program Files/Eclipse Adoptium/jdk-25"
items: []
blocks: []
recipes: []
creative_tabs: []
mobs: []
biomes: []
dimensions: []
dimension_types: []
structures: []
features: []
loot_tables: []
advancements: []
sound_events: []
tags: []
```

The `ModState` struct in `src/state.rs` is the serialized form of this YAML. It contains 20 tracked collections, each corresponding to an `Entity` enum variant.

## Commands

### `fw init`

Creates a new Fabric project scaffold by delegating to `deno run -A https://fabricmc.net/cli init`.

```
cargo run -- init <name> --version <ver> --java-path <path> [--dir <dir>] [--dangerous] [-o <option>]
```

Behavior:
- Validates mod name is non-empty and contains no spaces
- Validates `--version` against `SUPPORTED_VERSIONS` (`["26.2"]`) unless `--dangerous`
- Validates `deno` is on PATH (via `which::which`)
- Validates Java at `--java-path` meets `MIN_JAVA_BY_VERSION` requirement (Java 25 for MC 26.2)
- Derives `mod_id` (lowercase, alphanumeric + `_` + `-`), `package_name` (drops `-`, keeps `_`)
- Derives `namespace` = `mod_id`
- Defaults advanced options to `["datagen", "splitSources"]` if none provided
- Creates `<dir>/<name>/` via Fabric CLI
- Injects `org.gradle.java.home=<java_path>` into `gradle.properties` (if not already present)
- Writes `.fw/fabric-writer.yml` inside the created project

### `fw add item`

```
cargo run -- add item <id> [--kind <tool|axe|shovel|hoe|food|spawn_egg|fuel|compostable|armor|shield|potion|music_disc|fire_charge|flint_and_steel|compass|basic>]
  [--material <MATERIAL>] [--attack-damage <f32>] [--attack-speed <f32>] [--durability <i32>]
  [--nutrition <i32>] [--saturation <f32>] [--always-edible] [--entity-type <id>]
  [--burn-time <i32>] [--compost-chance <f32>] [--tooltip <line>] [--creative-tab <tab>]
  [--armor-material <mat>] [--armor-slot <slot>] [--effect <type=duration:amp>] [-v]
```

- Loads state, builds `Item`, saves state, calls `regenerate_all()` with dirty flags for the item-affected files. `--verbose` / `-v` (default false) prints per-file `wrote`/`skipped`/`pruned`.
- Items default to `ItemKind::Basic` with no material/damage/speed/durability
- Tool/axe/shovel/hoe items with `--material` generate `.sword`/`.ax`/`shovel`/`.hoe` calls in `Item.Properties` (material is uppercased and prefixed with `ToolMaterials.`)
- Food items use `FoodProperties.Builder` with nutrition/saturation/always-edible
- Spawn egg items use `SpawnEggItem` with `--entity-type`
- Fuel/compostable items register burn time/compost chance
- Armor items use `humanoidArmor()` with `--armor-material` and `--armor-slot`
- Shield items use `BlocksAttacks`
- Potion items use `PotionContents` with `--effect type=duration:amplifier`
- Music disc items use `MusicDisc`
- Items with `--tooltip` lines generate a custom item class extending `Item` with `appendHoverText`

### `fw add block`

```
cargo run -- add block <id> [--model-kind <CubeAll|CubeBottomTop|Cube>]
  [--properties-from <block>] [--creative-tab <tab>] [-v]
```

- Creates `Block` with id; defaults to `BlockBehaviour.Properties.ofFullCopy(Blocks.DIRT)`
- `--model-kind` controls the generated model JSON (supports `CubeAll`, `CubeBottomTop`, `Cube` via CLI; 24 full `BlockModelKind` variants in state)
- `--properties-from` copies properties from a vanilla block (e.g. `netherrack`, `basalt`, `dirt`)
- Auto-creates a `BlockItem` for inventory/creative tab

### `fw add recipe`

```
cargo run -- add recipe <id> [--kind <crafting_shaped|crafting_shapeless|smelting|blasting|smoking|campfire_cooking|stonecutting|smithing>]
  [--result <item>] [--count <u32>] [--pattern <line>...] [--ingredients <key=value>...]
  [--cooking-time <i32>] [--experience <f32>] [--category <cat>] [-v]
```

- Defaults to `crafting_shaped` with empty pattern
- `--pattern` accepts multiple lines (repeatable flag), each line is one row of the recipe grid
- `--ingredients` accepts `key=value` pairs where key is a single character matching the pattern and value is an item ID (`minecraft:wood` for vanilla, `mymod:my_item` for mod items)
- `--cooking-time` and `--experience` are used for cooking recipes (`smelting`, `blasting`, `smoking`, `campfire_cooking`)
- `--category` sets the recipe category (e.g. `building_blocks`, `combat`, `food`, `MISC`)
- Generates `<ModName>RecipeProvider.java` extending Fabric's `FabricRecipeProvider` with `shaped()` / `shapeless()` / `smelting()` / `blasting()` / `smoking()` / `campfireCooking()` / `stonecutterResultFromBase()` / `smithing()` calls
- Recipe provider is conditionally registered in the DataGenerator entrypoint only when recipes exist
- Provider is pruned when all recipes are removed

### `fw remove item|block|recipe|...`

```
cargo run -- remove <entity-type> <id> [-v]
```

Supported entity types for removal: `item`, `block`, `recipe`, `creative-tab`, `mob`, `biome`, `dimension`, `feature`, `structure`, `loot-table`, `advancement`, `sound-event`, `tag`.

- Loads state, removes entity by id, saves, regenerates all Java

### `fw regen`

Regenerates all Java source files from current state. Used by `init`/add/remove flows internally; also available standalone (alias: `g`). Verbose by default; pass `--verbose false` to suppress the per-file `wrote`/`skipped`/`pruned` log.

### `fw status`

Prints mod summary: name, mod_id, package, MC version, item/block counts, optionally advanced options with `-v` (alias: `s`).

### `fw run`

Runs Gradle tasks via `gradlew` (or `gradlew.bat` on Windows), injecting `JAVA_HOME` from state:
- `fw run datagen` (alias: `d`)
- `fw run client` (alias: `c`)
- `fw run dc` — datagen then client (alias: `dc`)
- `fw run server` (alias: `s`)

### `fw save` / `fw load`

```
fw save <path>           # Export current state to a YAML file
fw load <path> [-v]      # Import state from a YAML file and regenerate all Java sources
```

- `fw save` exports the current `.fw/fabric-writer.yml` to an arbitrary path
- `fw load` imports state from a YAML file, validates duplicate IDs and recipe references, writes it back to `.fw/fabric-writer.yml`, and regenerates all Java sources
- Both commands support `--verbose` / `-v` to show per-file regeneration status

### `fw test-project`

```
fw test-project [--preset <default|blue-nether-base>] [--reset]
```

- `fw test-project` (no preset) or `--preset default` adds a default test project with mixed item kinds and recipes
- `--preset blue-nether-base` adds the full Blue Nether content set (terrain blocks, variant blocks, flora, items, mobs, recipes, biome, dimension, sounds, loot, advancements, tags, etc.) with texture tinting from vanilla sources
- `--reset` clears all tracked entities before adding preset content

## Java Code Generation

`java_writer::regenerate_all()` emits 22 files into `src/main/java/<package>/` and `src/client/java/<package>/client/`:

| # | File | Location | Condition |
| --- | --- | --- | --- |
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
| 22 | `<ModName>PortalBlock.java` | `src/main/java/<package>/` | Dimensions with portal_frame and portal_igniter exist |

Additionally, one `<ItemId>Item.java` file is generated per item that has tooltip lines (e.g. `MySwordItem.java`).

Templates live in `src/tokengen.rs` using `genco::prelude::quote!`. Imports are managed via `src/imports.rs` (LazyLock pattern for Minecraft/Fabric imports).

> `write()` in `java_writer.rs` now compares new content against the existing file before writing, preserving mtime when unchanged. Dirty-file tracking (`DirtyFlags`) governs which files are regenerated per mutation; see the Done section above. Files for empty collections are pruned automatically.

## Local Reference Materials

The following local paths contain authoritative reference code and docs for Minecraft 26.2 / Fabric API 26.2. Future agents should consult these before implementing item/block/recipe/codegen features.

- **Minecraft source:** `E:\Coding_Projects\MCSourceCode\26.2`
  - Contains `net/minecraft/...` classes used by codegen imports and API signatures.
- **Fabric API source:** `E:\Coding_Projects\MCSourceCode\fabric-api\26.2`
  - Contains `net/fabricmc/fabric/api/...` provider classes and client datagen APIs.
- **Fabric docs (Markdown):** `E:\Coding_Projects\MCSourceCode\fabric-docs\develop`
  - Canonical feature docs: items, blocks, recipes, data generation, etc.
- **Fabric docs example code (Java):** `E:\Coding_Projects\MCSourceCode\fabric-docs\reference\latest\src`
  - Complete working examples for every feature, including armor, shields, potions, tools, creative tabs, etc.

When generating Java, imports should be validated against the Minecraft/Fabric API source in `E:\Coding_Projects\MCSourceCode\...` rather than guessed.

## Non-Goals (v0.1.0)

- **Non-Fabric modloader support** — Forge/NeoForge support is out-of-scope. Potentially not that difficult (parallel init flow + different MC constants), but not interesting right now.
- **GUI or web UI** — CLI-only. No GUI planned for v0.1.0; the README mentions it as a future possibility but it's not a priority.
- **AI integration** — Not on the roadmap.

## Speculative Concepts

These are ideas that are being considered but not yet planned; not concrete and may never be implemented. An idea should only appear here if it is not also listed in another section.

- **Diff/merge engine for user edits** — If a user manually edits generated Java files, `regenerate_all` will overwrite their changes. A git-based diff/merge approach is a distant future idea.
- **Custom texture support** — Allow users to place their own PNG textures in `assets/` and reference them via a `--texture` flag on `add item`/`add block`. The `ModelProvider` would emit models using the custom texture path instead of the placeholder. Requires storing texture paths in state and modifying model-generation templates to accept a custom texture reference.
- **Modding DSL interpreter** — Use `fabric-writer`'s state model as the basis for a small interpreted language where users write `.fw` scripts instead of YAML, e.g. `item "my_sword" { kind = tool; material = "diamond" }`. A fun experiment but no practical purpose.
- **Minecraft JAR texture extraction** — Given a path to a Minecraft client jar, extract its textures into an "available textures" folder. Could pair with basic image processing (hue/saturation adjustments) to reuse vanilla textures as a starting point.
- **Git-based dirty tracking** — Use `git stash`/`git merge` to preserve user edits in generated files across regenerations. Unknown whether this is feasible without significant complexity.
