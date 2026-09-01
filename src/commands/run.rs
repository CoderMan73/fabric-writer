use crate::state::{self, ModState};
use anyhow::{Context, Result, bail};
use std::process::Command;

pub fn datagen() -> Result<()> {
    run_gradle("runDatagen")
}

pub fn client() -> Result<()> {
    run_gradle("runClient")
}

fn run_gradle(task: &str) -> Result<()> {
    let state = state::load()?;
    let gradle_wrapper = if cfg!(target_os = "windows") {
        "gradlew.bat"
    } else {
        "./gradlew"
    };

    let status = Command::new(gradle_wrapper)
        .arg(task)
        .env("JAVA_HOME", &state.java_path)
        .current_dir(".")
        .status()
        .with_context(|| format!("Failed to spawn {} for task {}", gradle_wrapper, task))?;

    if !status.success() {
        bail!(
            "Gradle task '{}' failed with exit code {:?}",
            task,
            status.code()
        );
    }

    Ok(())
}
