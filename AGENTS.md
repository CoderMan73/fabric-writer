# AGENTS.md — Guide for Coding Agents

> A file for [guiding coding agents](https://agents.md/).

This document is intended for AI coding agents and human developers working with
AI assistance. It provides the project-specific knowledge needed to navigate
`fabric-writer` with confidence.

## Build & Test Commands

```bash
# Format
cargo fmt

# Lint (strict — zero warnings)
cargo clippy --all-targets --all-features -- -D warnings -W rustdoc::all -W missing_docs

# Documentation (must be warning-free)
cargo doc --no-deps

# Run tests
cargo test

# Full CI suite (Docker — recommended on Windows)
docker build -t fabric-writer .
make test   # fmt --check, clippy, doc, test, build

# Windows builds:
# If cargo fails with LNK1104, ensure TEMP and TMP point to real Windows
# paths, not MSYS /tmp. For local CI-style runs, use `ci.ps1`, which refreshes
# PATH before invoking cargo.
#
# Rebuild test cache (requires Deno + JDK 25)
cp .env.example .env
cargo test -- --ignored
```

## Project Architecture

### Overview

`fabric-writer` is a Rust CLI (edition 2024) that scaffolds Fabric mod projects.
It wraps the Fabric CLI (`fabric init` via Deno) to create the project, then
tracks mod state in `.fw/fabric-writer.yml` and regenerates Java source files
using `genco`. Every `add`/`remove` command loads state → mutates → saves →
calls `regenerate_all()` to emit Java.

### Source Layout

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
    ├── mod.rs       # module declarations
    ├── init.rs      # fw init — fabric init wrapper
    ├── item.rs      # add/remove items
    ├── block.rs     # add/remove blocks
    ├── recipe.rs    # add/remove recipes
    ├── regen.rs     # fw regen — regenerate all Java from state
    ├── run.rs       # fw run datagen/client/server (gradlew)
    └── status.rs    # fw status — print mod summary
```

### The State Flow

1. Each `add`/`remove` command calls `state::load()` → reads `.fw/fabric-writer.yml`
2. Mutates the `ModState` (adds/removes from `items`, `blocks`, or `recipes` Vec)
3. Calls `state::save()` → writes YAML back
4. Calls `regenerate_all(&state, dirty, verbose)` → emits Java files via genco

**Key:** `regenerate_all` takes a `DirtyFlags` bitmask so only affected files are rewritten. `fw regen` passes `DirtyFlags::all()`. When a collection becomes empty, its files are **pruned** (deleted). When non-empty, they're **regenerated**.

### The 10 Generated Java Files

| File | Location | Condition |
|---|---|---|
| `<ModName>.java` | `src/main/java/<package>/` | Always exists |
| `ModItemIds.java` | `src/main/java/<package>/` | Items exist |
| `ModItems.java` | `src/main/java/<package>/` | Items exist |
| `ModBlocks.java` | `src/main/java/<package>/` | Blocks exist |
| `ModBlockIds.java` | `src/main/java/<package>/` | Blocks exist |
| `ModBlockItemIds.java` | `src/main/java/<package>/` | Blocks exist |
| `LangProvider.java` | `src/client/java/<package>/client/` | Items or blocks exist |
| `ModelProvider.java` | `src/client/java/<package>/client/` | Items or blocks exist |
| `<ModName>DataGenerator.java` | `src/client/java/<package>/client/` | Items or blocks exist |
| `<ModName>RecipeProvider.java` | `src/client/java/<package>/client/` | Recipes exist |

### genco Code Generation Patterns

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
}
```

```rust
Block {
    id: String,  // Lowercase block identifier
}
```

## Recipe System

Recipes use `RecipeProvider` (Fabric datagen API) — not raw JSON files.

### Recipe struct

```rust
Recipe {
    id: String,                 // e.g. "my_sword"
    kind: String,               // "crafting_shaped" or "crafting_shapeless"
    pattern: Vec<String>,       // One string per row (shaped only)
    ingredients: HashMap<String, String>, // "S" -> "minecraft:stick"
    result: String,             // "minecraft:diamond" or "mymod:my_item"
    count: u32,                 // Default: 1
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
fw add recipe <id> --kind crafting_shaped|crafting_shapeless \
  --result <item> --count <u32> \
  --pattern <line>... \
  --ingredients <key=value>...
```

- `--pattern` is repeatable — each string is one row of the crafting grid
- `--ingredients` is `key=value` where key is a single char and value is an item ID
- Vanilla items use `minecraft:` prefix (e.g. `minecraft:wood`)
- Mod items use the mod's namespace (e.g. `mymod:ingot`)

## ItemKind / Tool Implementation

Items have a `kind` field (`ItemKind::Basic` or `ItemKind::Tool`). This is **real**, not stubbed:

- `ItemKind::Tool` with a `--material` generates `.sword(ToolMaterials.MATERIAL, damage, speed)` in `Item.Properties`
- `ItemKind::Basic` generates plain `new Item.Properties()`
- `--durability` adds `.durability(N)` override
- Materials are lowercase strings (e.g. `"diamond"`, `"iron"`, `"netherite"`)

The `item_properties()` function in `tokengen.rs` handles this logic.

## File Editing Guidelines

When making changes, focus on **only what the task needs** — no drive-by refactors.

### What to change

- **New entity attribute:** Add to `state.rs` types → extend `tokengen.rs` template → add CLI flag in `commands/<entity>.rs` → add to `DirtyFlags` if it affects new files in `java_writer.rs`
- **New file spec:** Add to `file_specs()` in `java_writer.rs` with a `build` fn, `should_exist` predicate, and dirty flag in `DirtyFlags`
- **New import:** Add to `imports.rs` (LazyLock pattern)

### What NOT to do

- Don't parse or edit Fabric-generated files directly
- Don't use inline `//` comments unless absolutely necessary — write self-documenting code
- Don't modify `.env` or credential files
- Don't stash or pop git changes

## Issue and PR Workflow

### Before making a PR

```bash
cargo fmt
cargo clippy --all-targets --all-features -- -D warnings -W rustdoc::all -W missing_docs
cargo doc --no-deps
cargo test
cargo build
```

> See `spec.md` for the spec workflow (feature tracking, status sections, etc.).

## Common Pitfalls

1. **Java path:** `fw init` validates JDK version (25+ for MC 26.2). The path must point to the JDK root, not the `bin` directory.
2. **Deno not found:** `fw init` requires Deno on PATH. Install from [deno.land](https://deno.land/manual/getting_started/installation).
3. **Test cache corruption:** If tests fail with "mod already exists" or similar, delete `.testing-cache/26.2/` and rebuild with `cargo test -- --ignored`.
