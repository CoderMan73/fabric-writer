#![warn(rustdoc::all, missing_docs)]

//! CLI tool for scaffolding Fabric Minecraft mods.
//!
//! `fabric-writer` delegates to the official Fabric CLI (`fabric init`) to create
//! the project structure, then tracks generated content in
//! `.fw/fabric-writer.yml` so subsequent commands can regenerate Java sources.
//!
//! Typical workflow:
//!
//! ```text
//! fw init --name MyMod --version 26.2 --java-path /path/to/jdk
//! fw add item --id my_item
//! fw add block --id my_block
//! fw regen
//! fw status
//! ```

/// Command implementations for `fw`.
pub mod commands;
/// LazyLock-based Java import helpers for genco templates.
pub mod imports;
/// Java file emission.
pub mod java_writer;
/// Serializable mod state loaded from and saved to `.fw/fabric-writer.yml`.
pub mod state;
/// genco `quote!` templates for Java classes.
pub mod tokengen;
