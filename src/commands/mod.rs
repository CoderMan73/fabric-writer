//! Command implementations for `fw`.

/// Block related commands.
pub mod block;
/// Project initialization via the Fabric CLI.
pub mod init;
/// Item related commands.
pub mod item;
/// Recipe related commands.
pub mod recipe;
/// Full regeneration of all Java sources.
pub mod regen;
/// Gradle task execution.
pub mod run;
/// Status reporting for the current mod project.
pub mod status;
