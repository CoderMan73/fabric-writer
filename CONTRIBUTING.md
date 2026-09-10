# Contributing to fabric-writer

Thank you for considering a contribution to `fabric-writer`! This document covers everything you need to know to get set up, run tests, and submit changes.

## Table of Contents

- [Code of Conduct](#code-of-conduct)
- [Getting Started](#getting-started)
- [Development Workflow](#development-workflow)
- [Running Tests](#running-tests)
- [Code Style](#code-style)
- [Submitting Changes](#submitting-changes)
- [Issue Templates](#issue-templates)

## Code of Conduct

This project follows the [Rust Code of Conduct](https://www.rust-lang.org/policies/code-of-conduct). Be respectful and constructive in all interactions.

## Getting Started

### Prerequisites

- **Rust** 1.85+ (edition 2024) with `rustfmt` and `clippy`:
  ```bash
  rustup component add rustfmt clippy
  ```
- **Docker** (recommended) — the project includes a `Dockerfile` and `Makefile` for containerized builds. This avoids build issues.
- **Deno** and **JDK 25** — only needed for `fw init` and the `--ignored` cache-rebuild test.

### Clone and Build

```bash
git clone https://github.com/CoderMan73/fabric-writer.git
cd fabric-writer

# Build the binary
cargo build

# Build with Docker (cross-platform, cached)
docker build -t fabric-writer .
make run CMD="cargo build"
```

## Development Workflow

1. **Make your changes** in `src/` (Rust) or `tests/` (integration tests).
2. **Format and lint**:
   ```bash
   cargo fmt
   cargo clippy --all-targets --all-features -- -D warnings -W rustdoc::all -W missing_docs
   ```
3. **Build docs** (must be warning-free):
   ```bash
   cargo doc --no-deps
   ```
4. **Run tests**:
   ```bash
   cargo test
   ```
5. **Run the full CI suite locally**:
   ```bash
   make test  # Docker: fmt --check, clippy, doc, test, build
   # or on Windows:
    ci.ps1
   ```

### Project Structure

```
fabric-writer/
├── src/
│   ├── main.rs          # CLI definition + dispatch
│   ├── lib.rs           # crate root, module declarations
│   ├── state.rs         # ModState, Entity, Item, Block, Recipe, load/save
│   ├── imports.rs       # genco Java import helpers (LazyLock-based)
│   ├── java_writer.rs   # regenerate_all() + write() — Java file emission
│   ├── tokengen.rs      # genco quote! templates for Java classes
│   └── commands/
│       ├── init.rs      # fw init, fabric init wrapper
│       ├── item.rs      # add/remove items
│       ├── block.rs     # add/remove blocks
│       ├── recipe.rs    # add/remove recipes
│       ├── regen.rs     # fw regen, regenerate all Java from state
│       ├── run.rs       # fw run datagen/client/server (gradlew)
│       └── status.rs    # fw status, print mod summary
├── tests/
│   ├── tests.rs         # integration tests
│   └── common/mod.rs    # TestEnv, cache-based test fixture
├── spec.md              # feature specification and roadmap
├── AGENTS.md            # guide for coding agents
└── .github/workflows/ci.yml
```

### Key Concepts

- **State:** The single source of truth is `.fw/fabric-writer.yml`. Every add/remove command loads → modifies → saves state, then calls `regenerate_all()`.
- **Dirty flags:** `DirtyFlags` in `java_writer.rs` tracks which generated files need updating. Each command derives flags from the mutated `Entity`. `fw regen` bypasses this with `DirtyFlags::all()`.
- **File pruning:** When a collection (items, blocks, recipes) becomes empty, its associated Java files are pruned (deleted). When it has content, files are regenerated.
- **genco patterns:** Java code is generated via `genco::quote!`. Use `String` interpolation for dynamic identifiers (e.g. `$(format!("Items.{}", mc_constant))` emits raw `Items.DIRT` tokens). See `src/tokengen.rs` for patterns.

## Running Tests

```bash
cargo test
```

### Test Strategy

Tests are **serial** (filesystem-dependent) and use a **cached Fabric project** at `.testing-cache/26.2/TestMod/`. `TestEnv::new()` copies this cache into a fresh `tempfile::TempDir` for each test, so tests are isolated but fast.

### Rebuilding the Test Cache

To rebuild the cache from scratch (requires Deno + JDK 25):

```bash
# Copy .env.example to .env and set FABRIC_WRITER_TEST_JAVA
cp .env.example .env
# Edit .env to point to your JDK 25

cargo test -- --ignored
```

This runs `fw init` to create a fresh scaffold at `.testing-cache/26.2/TestMod/`.

## Code Style

### Formatting

```bash
cargo fmt        # format
cargo fmt -- --check  # verify (run by CI)
```

### Linting

```bash
cargo clippy --all-targets --all-features -- -D warnings -W rustdoc::all -W missing_docs
```

### Documentation Rules

- **Crate roots** (`src/lib.rs`, `src/main.rs`) must carry `#![warn(rustdoc::all, missing_docs)]` and a crate-level `//!` doc comment.
- **Every `pub` item** (functions, structs, enums, variants, fields) must have a `///` doc comment that explains *why* and *what it does*, not just restates the signature.
- **Module declarations** (`pub mod foo;`) should have a one-line `///` doc comment.
- **Intra-doc links** are encouraged (`[ModState]`, `[regenerate_all]`).
- **Test files** use `#![allow(missing_docs)]` (not part of public API).

### Comments

- Use `///` doc comments on public items.
- **Avoid inline `//` comments** unless absolutely necessary for clarity. Code should be self-documenting.
- A multi-line comment explaining what `item.id` or `create_dir_all` does will be removed in review.

### Naming

- `snake_case` for functions/variables, `PascalCase` for types, `UPPER_SNAKE_CASE` for constants
- CLI-facing names use kebab-case (clap's default)

### Error Handling

- Use `anyhow::Result` in `main` and command handlers
- Use `anyhow::bail!` for early returns with clear messages
- Use `.context("...")` when wrapping errors

## Submitting Changes

1. **Fork** the repo and create a branch from `main`.
2. **Write tests**, new features should include integration tests in `tests/tests.rs`.
3. **Run the full check suite**, `cargo fmt`, `cargo clippy`, `cargo doc --no-deps`, `cargo test`, `cargo build` must all pass.
4. **Open a PR**, include a clear description of what you're solving and why. Make PRs about one thing only.
5. **Update `spec.md`** if your change is behavioral, move the feature from "In progress" to "Done" or add to "Speculative concepts" as appropriate.

### PR Guidelines

- Make PRs small and focused, one feature or bug fix per PR
- Include tests for behavioral changes
- Update `spec.md` for behavioral changes
- Follow the code style above, PRs that don't pass `cargo fmt` or `cargo clippy` will be rejected
- If your change affects generated Java output, verify the output manually

## Issue Templates

When reporting issues, please use the provided templates:

| Type | When to use |
|---|---|
| [Bug report](.github/ISSUE_TEMPLATE/bug_report.md) | Something doesn't work as expected |
| [Feature request](.github/ISSUE_TEMPLATE/feature_request.md) | A new command, flag, or entity type |
| [Question](.github/ISSUE_TEMPLATE/question.md) | Need help understanding or using the tool |

All issues should include:
- Your OS and Rust version (`rustc --version`)
- The `fw` command you ran and its output
- For bugs: a minimal reproduction if possible

---

**Note for AI coding agents:** If you're working on this codebase via an AI coding agent, see [AGENTS.md](AGENTS.md) for project-specific guidance.
