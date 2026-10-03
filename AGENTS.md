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

## Research-First Rule

Always use the docs and reference examples as the primary source of truth.

- Search `E:\Coding_Projects\MCSourceCode\fabric-docs` first for the exact feature being implemented.
- Use the reference example code in those docs as the implementation pattern.
- Do not inspect or edit Fabric-generated files directly.
- Do not invent parallel workarounds in Java/Rust to bypass documented APIs.
- If the documented path is unclear or insufficient, stop and surface that instead of hacking around it.

For full architecture details, see [docs/architecture.md](docs/architecture.md).
For Blue Nether development workflow, see [docs/blue-nether-guide.md](docs/blue-nether-guide.md).
For codegen patterns, see [docs/codegen-guide.md](docs/codegen-guide.md).

## Common Pitfalls

1. **Java path:** `fw init` validates JDK version (25+ for MC 26.2). The path must point to the JDK root, not the `bin` directory.
2. **Deno not found:** `fw init` requires Deno on PATH. Install from [deno.land](https://deno.land/manual/getting_started/installation).
3. **Test cache corruption:** If tests fail with "mod already exists" or similar, delete `.testing-cache/26.2/` and rebuild with `cargo test -- --ignored`.
