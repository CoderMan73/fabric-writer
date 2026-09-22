use crate::state;
use anyhow::{Context, Result, bail};
use std::env::current_dir;
use std::path::PathBuf;
use std::process::Command;

/// Runs the `runDatagen` Gradle task for the current mod project.
pub fn datagen() -> Result<()> {
    run_gradle("runDatagen")
}

/// Runs the `runClient` Gradle task for the current mod project.
pub fn client() -> Result<()> {
    run_gradle("runClient")
}

/// Runs datagen then client in succession.
pub fn client_with_datagen() -> Result<()> {
    datagen()?;
    client()
}

/// Runs the `runServer` Gradle task for the current mod project.
pub fn server() -> Result<()> {
    run_gradle("runServer")
}

fn run_gradle(task: &str) -> Result<()> {
    let state = state::load()?;
    let gradle_wrapper = if cfg!(target_os = "windows") {
        current_dir()?.join("gradlew.bat")
    } else {
        PathBuf::from("./gradlew")
    };

    let status = Command::new(&gradle_wrapper)
        .arg(task)
        .arg("--no-daemon")
        .env("JAVA_HOME", &state.java_path)
        .status()
        .with_context(|| format!("Failed to spawn {:?} for task {}", gradle_wrapper, task))?;

    if !status.success() {
        bail!(
            "Gradle task '{}' failed with exit code {:?}",
            task,
            status.code()
        );
    }

    Ok(())
}
