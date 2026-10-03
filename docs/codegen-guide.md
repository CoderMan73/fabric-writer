# Codegen Guide

This document is a contributor guide for adding new entity types to
`fabric-writer`'s code generation system. It covers the `genco` patterns, the
`FileSpec` system, `DirtyFlags`, and the extension workflow.

## Architecture Overview

Code generation flows through three modules:

1. **`state.rs`** — Serializable state types (`ModState`, `Entity` enum, entity structs)
2. **`tokengen.rs`** — `BuildFn` functions (`fn(&ModState) -> Tokens`) that produce Java source via `quote!`
3. **`java_writer.rs`** — `FileSpec` structs, `DirtyFlags` bitmask, `regenerate_all()`, `write()`

## Extension Pattern for New Entity Types

To add a new entity type, follow these steps:

### 1. State (`src/state.rs`)

Add the entity struct, its `Entity` enum variant, and the collection `Vec` in `ModState`:

```rust
// Add to Entity enum:
Entity::MyType(MyType),

// Add to ModState struct:
#[serde(default, skip_serializing_if = "Vec::is_empty")]
pub my_types: Vec<MyType>,

// Add match arms to has_id(), has_cross_type_id(), add(), remove()
// (All 4 match expressions must be updated — the compiler will error if any is missing)

// Add #[serde(default)] to new ModState fields to avoid breaking deserialization of older state files
```

### 2. CLI (`src/commands/<entity>.rs`)

Create a new command module following the existing pattern:

```rust
// Add to src/commands/mod.rs:
pub mod my_type;

// In src/commands/my_type.rs:
pub fn add(args: MyTypeAddArgs) -> Result<()> {
    let verbose = args.verbose;
    let mut state = state::load()?;
    let entity = Entity::MyType(MyType::new(&args.id)?);
    let dirty = DirtyFlags::from_entity(&entity);
    state.add(entity)?;
    state.save()?;
    regenerate_all(&state, dirty, verbose)?;
    println!("Added my type: {}", args.id);
    Ok(())
}

// Register in src/main.rs Commands::Add and Commands::Remove
```

### 3. Dirty Flags (`src/java_writer.rs`)

Add a bit to `DirtyFlags` for each new generated file:

```rust
pub struct DirtyFlags {
    // ... existing fields ...
    /// `ModMyTypes.java`
    pub mod_my_types: bool,
}
```

Update `all()`, `from_entity()`, and `is_set()`.

### 4. Template (`src/tokengen.rs`)

Add a `BuildFn`:

```rust
pub(crate) fn build_mod_my_types(state: &ModState) -> Tokens {
    if state.my_types.is_empty() {
        return quote! {
            public class ModMyTypes {
                public static void initialize() {}
            }
        };
    }
    quote! {
        public class ModMyTypes {
            // ... registration code ...
        }
    }
}
```

Import it in `java_writer.rs`:

```rust
use crate::tokengen::{BuildFn, build_mod_my_types, /* ... */};
```

### 5. Imports (`src/imports.rs`)

Add `Import` entries for new Java types using the LazyLock pattern:

```rust
pub static my_type: LazyLock<Import> = LazyLock::new(|| {
    Import::from("net.minecraft.world.level.block", "MyType")
});
```

Use `Import` types (not `format!()`) for imports — genco tracks them automatically.

### 6. Writer (`src/java_writer.rs`)

Add a `FileSpec` entry to `file_specs()` and a match arm to `resolve_path()`:

```rust
// In file_specs():
(
    "ModMyTypes.java",
    FileSpec {
        field: "mod_my_types",
        build: build_mod_my_types,
        should_exist: |state| !state.my_types.is_empty(),
    },
),

// In resolve_path():
"ModMyTypes.java" => java_root.join(name),
```

## genco Patterns

### Dynamic Identifiers

Use `format!()` to build Java identifiers, then interpolate as `String` in `quote!`.
genco treats `String` as a literal token (raw text, not a quoted string):

```rust
$(format!("Items.{}", mc_constant)) → emits Items.DIRT
```

### Imports

Use `Import` types from `imports.rs` for automatic import tracking. Don't `format!`
imports — genco won't track them.

### Repeatable Blocks

Use `$(for item in &collection => ... )` for loops, `$(if condition => ... )` for conditionals.

### Line Endings

Append `$['\r']` to emit `System.lineSeparator()` in Java (Windows-friendly).

### Empty Collection Handling

Every `BuildFn` must handle empty collections gracefully by generating valid (possibly
empty) Java, not broken syntax. Use an early return:

```rust
if state.my_types.is_empty() {
    return quote! {
        public class ModMyTypes {
            public static void initialize() {}
        }
    };
}
```

## FileSpec System

Each `FileSpec` defines:

- `field`: The `DirtyFlags` field name that controls regeneration
- `build`: A `BuildFn` that generates Java `Tokens`
- `should_exist`: A predicate that decides whether the file should exist

When `should_exist` returns `false`:
- If the file exists on disk, it is **pruned** (deleted)
- If the file doesn't exist, it is **skipped**

When `should_exist` returns `true`:
- If the dirty bit is set, the file is **regenerated**
- If the dirty bit is not set, the file is **skipped** (preserving mtime)

The `write()` function compares new content against existing file content byte-for-byte
and skips the write if unchanged (belt-and-suspenders optimization).

## DirtyFlags Bitmask

`DirtyFlags` is a struct of booleans (not a true bitmask). It works as follows:

- `DirtyFlags::all()` — marks every file dirty (used by `fw regen`)
- `DirtyFlags::from_entity(&entity)` — marks only the files affected by that entity type
- `DirtyFlags::default()` — no files dirty

Each entity type's `from_entity` implementation determines which files need regeneration.
For example, adding an `Item` dirties `mod_item_ids`, `mod_items`, `mod_class`,
`lang_provider`, `model_provider`, and `datagen_entrypoint`.
