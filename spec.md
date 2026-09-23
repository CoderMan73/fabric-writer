# fabric-writer (fw) — Specification

A CLI tool that scaffolds Fabric mod projects by delegating to the official Fabric CLI (`fabric init`) and tracking project state in `.fw/fabric-writer.yml`. State mutations trigger immediate regeneration of Java source files via `genco`.

## Problem

Creating even a basic Fabric mod project requires running the official scaffold tool, then manually tracking metadata and boilerplate registry classes so subsequent commands can extend the project consistently.

## Solution

`fabric-writer` wraps `fabric init` (via `deno run -A https://fabricmc.net/cli init`) to create the project, derives modid/package/namespace from the human-readable name, validates Java version, injects `org.gradle.java.home` into `gradle.properties`, and writes `.fw/fabric-writer.yml` so the project state is explicit. Later commands mutate state and call `java_writer::regenerate_all()` to regenerate all Java sources.

## Current Status (v0.1.0)

**Done:**

- `fw init` — create/validate Fabric project scaffold via deno + fabric CLI
- State file: `.fw/fabric-writer.yml` inside the mod project root (`.fw/` dir)
- Name derivation: `mod_id` keeps `_` and `-`; `package_name` drops `-`
- Version validation: only `26.2` accepted unless `--dangerous` is passed
- Java validation: `MIN_JAVA_BY_VERSION` map (`26.2` → Java 25); `--java-path` required
- Advanced options: defaults to `["datagen", "splitSources"]` if no `-o` flags given
- `--dir` flag: create the mod folder inside a chosen parent directory
- `gradle.properties` injection: `org.gradle.java.home=<java_path>`
- **Item add/remove** — `--kind tool|basic`, `--material`, `--attack-damage`, `--attack-speed`, `--durability`
- **Block add/remove** — bare id, defaults to `BlockBehaviour.Properties.ofFullCopy(Blocks.DIRT)`
- **Recipe add/remove** — `--kind` (default `crafting_shaped`), `--result`, `--count`, `--pattern` (repeatable), `--ingredients` (key=value pairs). Supports `crafting_shaped` and `crafting_shapeless`. Generates `<ModName>RecipeProvider.java` extending `FabricRecipeProvider`.
- `fw regen` — regenerate all Java from current state (alias: `g`)
- `fw status` — print mod summary; `-v` for verbose (alias: `s`)
- `fw run datagen|client|server` — exec gradlew tasks (aliases: `d`/`c`/`s`)
- Java codegen via `genco` — emits 9 source files (ModItems, ModBlocks, ModItemIds, ModBlockIds, ModBlockItemIds, main mod class, LangProvider, DataGenerator entrypoint, ModelProvider)
- Item tools: sword/material/damage/speed/durability support in `item_properties()`
- Block items: auto-registered as `BlockItem` with `useBlockDescriptionPrefix().setId()`
- Placeholder assets: `placeholder_item.png` / `placeholder_block.png` copied into `assets/<mod_id>/textures/{block,item}/` during `regenerate_all`, embedded via `include_bytes!`
- Integration test suite (`tests/`) with cache-based fixture, `serial_test`, `tempfile`, `dotenvy`
- CI workflow (`.github/workflows/ci.yml`) — fmt + clippy + test + build on Ubuntu/macOS/Windows
- `ci.bat` — local Windows CI-style check runner
- **Dirty-file tracking** — `regenerate_all` now takes a `DirtyFlags` bitmask and a `verbose: bool`. Only files whose dirty bit is set are regenerated; the rest are left untouched (preserving mtime). Each add/remove command derives dirty flags from the mutated entity via `DirtyFlags::from_entity()`. `fw regen` bypasses the check by passing `DirtyFlags::all()`. A `--verbose` / `-v` flag on add/remove/regen commands prints which files were `wrote` / `skipped` / `pruned`. Additionally, `write()` now compares new content against the existing file and skips the write if unchanged (belt-and-suspenders). Lazy file existence: files for items (ModItems, ModItemIds), blocks (ModBlocks, ModBlockIds, ModBlockItemIds), and data-provider classes (LangProvider, ModelProvider, DataGenerator) are automatically pruned when the underlying collection goes empty, and only regenerated when dirtied. The main mod class always exists. Placeholder textures are likewise pruned when their corresponding item/block is removed.
- **Rustdoc conventions** — `#![warn(rustdoc::all, missing_docs)]` added to crate roots (`lib.rs`, `main.rs`); crate-level `//!` docs, `///` docs on all `pub` items and fields, module-level `///` on `pub mod` declarations, intra-doc links, and `#![allow(missing_docs)]` on test files. `cargo doc --no-deps` and `cargo clippy ... -W rustdoc::all -W missing_docs` both pass with zero warnings. AGENTS.md updated with the full rustdoc linting workflow. See the AGENTS.md "Rustdoc" section.

**In progress / next:**

- **Blue Nether mod implementation** — Extend `fabric-writer` codegen to support the full content set defined in `blue_nether_spec.md`. This includes remaining block types, item kinds, recipe kinds, mob/entity types, loot tables, biomes, dimension type, structures, advancements, sounds, and tag codegen. See the "Blue Nether Mod" addendum below.
- `fw test-project --preset blue-nether-base` currently adds the base terrain blocks and textures. Remaining content will be added as codegen support lands.

## Blue Nether Mod

The current active development target for `fabric-writer` is implementing the
content spec defined in:

`E:\Coding_Projects\blue_nether_spec.md`

That document is the source of truth for the Blue Nether mod content. The
implementation plan below maps each spec section to the codegen work required
in `fabric-writer`.

### Completed codegen support

- Block add/remove with `--model-kind` and `--properties-from`
- Base terrain block preset + texture copy workflow
- Item kinds: `basic`, `tool`, `axe`, `shovel`, `hoe`, `food`, `spawn_egg`, `fuel`, `compostable`, `armor`, `shield`, `potion`
- Recipes: `crafting_shaped`, `crafting_shapeless`
- Dirty-file tracking, placeholder assets, creative tabs, datagen + splitSources

### Remaining codegen work by spec step

| Spec step | Required codegen additions |
| --- | --- |
| Step 1 — Dimension | New `Entity::Dimension` / `DimensionType` state + Java datagen template for `DimensionType` registration and dimension entrypoint |
| Step 2 — Blue Obsidian / Portal | Block model / property support; portal ignition behavior likely requires manual Java, but frame/portal blocks can be generated |
| Step 3 — Base terrain | Mostly done via `blue_nether_base` preset |
| Step 4 — Biomes | New `Entity::Biome` / `Biome` state + Java datagen templates for `Biome` registration + `BiomeSource` / `MultiNoiseBiomeSourceParameterList` |
| Step 5 — Blocks | Remaining block model kinds: `slab`, `stairs`, `wall`, `fence`, `fence_gate`, `trapdoor`, `door`, `button`, `pressure_plate`, `sign`, `bed`, `flower_pot`, `lantern`, `chain`, `crop`, `double_plant`, `sapling`, `mushroom`, `fire`, `soul_fire`, `portal`, `ore`, `deepslate_ore`, `nylium`, `fungus`, `roots`, `vines`, `nether_sprouts`, `nether_wart`, `shroomlight`, `crying_obsidian`, `lodestone`, `music_disc`, `spawn_egg` item form |
| Step 6 — Items | New item kinds: `spawn_egg` complete, `music_disc`, `compass`, `potion`/`splash_potion`/`lingering_potion` with `PotionContents`, `fire_charge`, `flint_and_steel`, `writable_book`/`written_book` if needed |
| Step 7 — Mobs | New entity model: `Entity::Mob` with spawn egg, entity type, attributes, drops, equipment, behavior if possible. Otherwise manual Java. |
| Step 8 — Structures | New `Entity::Structure` / `Structure` state + Java datagen for structure set + template pool registration, or manual JSON + `StructureFeature` registration |
| Step 9 — Terrain features | New `Entity::Feature` / `Feature` state + Java datagen for configured feature / placed feature registration |
| Step 10 — Fog / Particles / Lighting | Biome fog color support in biome generation; particle/sound mapping likely manual |
| Step 11 — Flora | Mostly covered by additional block/item kinds above |
| Step 12 — Audio | New sound event registration + asset copy workflow |
| Step 13 — Loot | New `Entity::LootTable` / `LootTable` state + Java datagen for `LootTable` provider |
| Step 14 — Recipes | New recipe kinds: `crafting_special`, `smelting`, `blasting`, `smoking`, `campfire_cooking`, `stonecutting`, `smithing` |
| Step 15 — Advancements | New `Entity::Advancement` / `Advancement` state + Java datagen for advancement tree |

### Implementation priority

1. Finish block model kinds needed by Steps 5/11 (`stairs`, `slab`, `wall`, `fence`, `fence_gate`, `door`, `trapdoor`, `fire`, `soul_fire`, `portal`, `nylium`, `vines`, `nether_wart`, `shroomlight`)
2. Add new recipe kinds needed by Step 14 (`stonecutting`, `smithing`, `smelting`)
3. Add entity/mob model needed by Step 7
4. Add loot table codegen needed by Step 13
5. Add biome/dimension codegen needed by Steps 1/4
6. Add structure/feature codegen needed by Steps 8/9
7. Add advancement codegen needed by Step 15
8. Add sound registration + asset copy workflow needed by Step 12

## Spec Workflow

This file tracks feature status in four distinct sections:

- **Done** — completed features that are fully implemented and verified.
- **In progress / next** — features currently being worked on or queued for implementation.
- **Non-Goals** — features explicitly out of scope for the current release.
- **Speculative concepts** — ideas being considered but not yet planned; may never be implemented.

When completing a feature, move it from "In progress / next" to "Done". When an idea moves from speculative to planned, move it to "In progress / next" and remove it from "Speculative concepts". Never have the same idea in two sections at once.

## Tech Stack

- **Rust** — CLI binary (edition 2024)
- `cargo`/`rustfmt` — build + formatting
- `clap` 4 — CLI argument parsing (derive API)
- `serde` + `serde_yaml` — state file serialization
- `genco` — Java code generation (Tokens, `quote!`, `Java` config)
- `anyhow` — error handling
- `which` — used by `init` to verify `deno` is installed
- `tempfile` + `serial_test` + `dotenvy` — dev/test dependencies

## File Structure

```
fabric-writer/
├── Cargo.toml
├── Cargo.lock
├── src/
│   ├── main.rs
│   ├── lib.rs
│   ├── state.rs
│   ├── imports.rs
│   ├── java_writer.rs
│   ├── tokengen.rs
│   ├── rustfmt.toml
│   └── commands/
│       ├── mod.rs
│       ├── init.rs
│       ├── item.rs
│       ├── block.rs
│       ├── recipe.rs
│       ├── regen.rs
│       ├── run.rs
│       └── status.rs
├── tests/
│   ├── tests.rs
│   └── common/
│       └── mod.rs
├── .github/
│   └── workflows/
│       └── ci.yml
├── .env.example
├── ci.bat
├── AGENTS.md
├── spec.md
├── IDEA.md
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
```

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
cargo run -- add item <id> [--kind <tool|basic>] [--material <MATERIAL>]
  [--attack-damage <f32>] [--attack-speed <f32>] [--durability <i32>]
```

- Loads state, builds `Item`, saves state, calls `regenerate_all()` with dirty flags for the item-affected files. `--verbose` / `-v` (default false) prints per-file `wrote`/`skipped`/`pruned`.
- Items default to `ItemKind::Basic` with no material/damage/speed/durability
- Tool items with `--material` generate `.sword(ToolMaterials.MATERIAL, damage, speed)` in `Item.Properties` (material is uppercased and prefixed with `ToolMaterials.`)

### `fw add block`

```
cargo run -- add block <id>
```

- Creates `Block` with id; `BlockBehaviour.Properties.ofFullCopy(Blocks.DIRT)`
- Auto-creates a `BlockItem` for inventory/creative tab

### `fw add recipe`

```
cargo run -- add recipe <id> [--kind <crafting_shaped|crafting_shapeless>] [--result <item>] [--count <u32>]
  [--pattern <line>...] [--ingredients <key=value>...]
```

- Defaults to `crafting_shaped` with empty pattern
- `--pattern` accepts multiple lines (repeatable flag), each line is one row of the recipe grid
- `--ingredients` accepts `key=value` pairs where key is a single character matching the pattern and value is an item ID (`minecraft:wood` for vanilla, `mymod:my_item` for mod items)
- Generates `<ModName>RecipeProvider.java` extending Fabric's `FabricRecipeProvider` with `shaped()` or `shapeless()` calls
- Recipe provider is conditionally registered in the DataGenerator entrypoint only when recipes exist
- Provider is pruned when all recipes are removed

### `fw remove item|block|recipe`

```
cargo run -- remove item|block|recipe <id>
```

- Loads state, removes entity by id, saves, regenerates all Java

### `fw regen`

Regenerates all Java source files from current state. Used by `init`/add/remove flows internally; also available standalone (alias: `g`). Verbose by default; pass `--verbose false` to suppress the per-file `wrote`/`skipped`/`pruned` log.

### `fw status`

Prints mod summary: name, mod_id, package, MC version, item/block counts, optionally advanced options with `-v` (alias: `s`).

### `fw run`

Runs Gradle tasks via `gradlew` (or `gradlew.bat` on Windows), injecting `JAVA_HOME` from state:
- `fw run datagen` (alias: `d`)
- `fw run client` (alias: `c`)
- `fw run server` (alias: `s`)

## Java Code Generation

`java_writer::regenerate_all()` emits 10 files into `src/main/java/<package>/` and `src/client/java/<package>/client/`:

| File | Purpose |
| --- | --- |
| `ModItemIds.java` | `ResourceKey<Item>` registration per item |
| `ModItems.java` | Item registration + `Item.Properties` + creative tab |
| `ModBlocks.java` | Block registration + creative tab |
| `ModBlockIds.java` | `ResourceKey<Block>` registration per block |
| `ModBlockItemIds.java` | `BlockItemId` per block |
| `<ModName>.java` | Main `ModInitializer` class |
| `LangProvider.java` | Translation provider (en_us) |
| `<ModName>DataGenerator.java` | Data generator entrypoint |
| `ModelProvider.java` | Model provider (block states + item models) |
| `<ModName>RecipeProvider.java` | Recipe data provider (shaped/shapeless recipes via `shaped()`/`shapeless()` inherited from `RecipeProvider`) ||

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
- **YAML load/save CLI** — Expose the ability to save/load full YAML state files via the CLI for sharing configs between users.
- **Minecraft JAR texture extraction** — Given a path to a Minecraft client jar, extract its textures into an "available textures" folder. Could pair with basic image processing (hue/saturation adjustments) to reuse vanilla textures as a starting point.
- **Git-based dirty tracking** — Use `git stash`/`git merge` to preserve user edits in generated files across regenerations. Unknown whether this is feasible without significant complexity.
