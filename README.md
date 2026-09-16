# fabric-writer

[![License](https://img.shields.io/github/license/CoderMan73/fabric-writer)](https://github.com/CoderMan73/fabric-writer/blob/main/LICENSE.md)
[![Status](https://img.shields.io/badge/status-beta-yellow)](https://github.com/CoderMan73/fabric-writer)
[![Rust](https://img.shields.io/badge/rust-1.85%2B-orange?logo=rust)](https://www.rust-lang.org)
[![CI](https://github.com/CoderMan73/fabric-writer/actions/workflows/ci.yml/badge.svg)](https://github.com/CoderMan73/fabric-writer/actions/workflows/ci.yml)
[![Issues](https://img.shields.io/github/issues/CoderMan73/fabric-writer)](https://github.com/CoderMan73/fabric-writer/issues)

CLI for quickly scaffolding and drafting the basic functionality of a Fabric mod project. It wraps the official Fabric CLI (`fabric init`) to create the project structure, then tracks generated content in `.fw/fabric-writer.yml` so subsequent commands can regenerate Java sources automatically.

## What it does

`fabric-writer` (alias `fw`) eliminates the boilerplate of setting up a Fabric mod from scratch. It:

1. Scaffolds a new Fabric mod project via `deno run -A https://fabricmc.net/cli init`
2. Tracks your mod's state (items, blocks, recipes) in a YAML file so you can iterate
3. Generates clean, formatted Java source files via `genco`

## Requirements

- Rust 1.85+ (edition 2024)
- Deno
- JDK 25+

## Installation

### From source

```bash
git clone https://github.com/CoderMan73/fabric-writer.git
cd fabric-writer
cargo install --path .
```

### Via Cargo

```bash
cargo install --git https://github.com/CoderMan73/fabric-writer
```

## Quick Start

```bash
# 1. Create a new mod project
fw init MyAwesomeMod --version 26.2 --java-path "C:/Program Files/Eclipse Adoptium/jdk-25"

cd MyAwesomeMod

# 2. Add some content
fw add item my_sword --kind tool --material diamond --attack-damage 7 --attack-speed 1.6
fw add block my_ore
fw add recipe my_recipe --kind crafting_shaped --result minecraft:diamond --pattern "SSS" --pattern "S S" --pattern "SSS" --ingredients S=minecraft:stick

# 3. Watch the generated Java
fw status
fw regen

# 4. Run the Minecraft datagen / client / server
fw run datagen
fw run client
fw run server
```

## Commands

See [COMMANDS.md](COMMANDS.md) for the full auto-generated command reference.

## State Management

All mod content is tracked in `.fw/fabric-writer.yml` inside your project root:

```yaml
mod_name: MyMod
mod_id: mymod
package_name: mymod
minecraft_version: "26.2"
java_path: "C:/Program Files/JDK-25"
items:
  - id: my_sword
    kind: tool
    material: diamond
    attack_damage: 7.0
    attack_speed: 1.6
blocks: []
recipes:
  - id: my_recipe
    type: crafting_shaped
    pattern: ["SSS", "S S", "SSS"]
    ingredients:
      S: minecraft:stick
    result: minecraft:diamond
    count: 1
```

You can export and import this state with `fw save` and `fw load`:

```bash
# Export current state to a portable YAML file
fw save backup.yml

# Import state from a YAML file and regenerate all Java sources
fw load backup.yml
```

## Development

See [CONTRIBUTING.md](CONTRIBUTING.md) for the full contribution guide, and the
**Development** section below for quick setup.

### Prerequisites

- Rust 1.85+ with `rustfmt` and `clippy` components
- Deno
- JDK 25+

### Quick Dev

```bash
# Clone and build
git clone https://github.com/CoderMan73/fabric-writer.git
cd fabric-writer

# Run checks (inside Docker — see Dockerfile + Makefile)
docker build -t fabric-writer .
make test  # runs fmt --check, clippy, doc, test, build
```

See `ci.ps1` for a local CI-style runner.

### Test Cache

Integration tests use a cached Fabric project at `.testing-cache/26.2/TestMod/` to avoid re-running `fw init` on every test. To rebuild the cache from scratch:

```bash
cargo test -- --ignored
```

This requires `FABRIC_WRITER_TEST_JAVA` to be set (see `.env.example`).

## Contributing

Read [CONTRIBUTING.md](CONTRIBUTING.md) to get started. All contributions are welcome.

## License

Licensed under the GNU GPL v3. See [LICENSE.md](LICENSE.md).
