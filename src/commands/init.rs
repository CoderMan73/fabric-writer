use crate::state::{self, ModState};
use anyhow::{Context, bail};
use std::process::Command;

const SUPPORTED_VERSIONS: &[&str] = &["26.2"];

pub fn run(
    name: String,
    version: String,
    options: Vec<String>,
    dangerous: bool,
    dir: Option<String>,
) -> anyhow::Result<()> {
    let base_dir = match dir {
        Some(d) => std::path::PathBuf::from(d),
        None => std::env::current_dir()?,
    };

    validate_name(&name)?;
    validate_version(&version, dangerous)?;

    if !deno_on_path() {
        bail!(
            "Deno is not installed or not on PATH.\n\
             Install it from https://deno.land/manual/getting_started/installation"
        );
    }

    let mod_id = to_mod_id(&name);
    let package_name = to_package_name(&name);

    let target_dir = base_dir.join(&name);
    if target_dir.exists() {
        bail!(
            "Target directory '{}' already exists. Remove it or choose a different name.",
            target_dir.display()
        );
    }

    let advanced_options = if options.is_empty() {
        vec!["datagen".into(), "splitSources".into()]
    } else {
        options
    };

    let mut args = vec![
        "run".into(),
        "-A".into(),
        "https://fabricmc.net/cli".into(),
        "init".into(),
        "-n".into(),
        name.clone(),
        "-m".into(),
        mod_id.clone(),
        "-p".into(),
        package_name.clone(),
        "-v".into(),
        version.clone(),
    ];

    for opt in &advanced_options {
        args.push("-o".into());
        args.push(opt.clone());
    }
    args.push(target_dir.display().to_string());

    println!("Running: deno {}", args.join(" "));
    let status = Command::new("deno")
        .args(&args[1..])
        .stdin(std::process::Stdio::inherit())
        .stdout(std::process::Stdio::inherit())
        .stderr(std::process::Stdio::inherit())
        .status()
        .context("Failed to spawn deno process")?;

    if !status.success() {
        bail!(
            "fabric init failed with exit code {}\n\
             A partial project may exist at '{}'.",
            status.code().unwrap_or(-1),
            target_dir.display()
        );
    }

    let fw_dir = target_dir.join(".fw");
    std::fs::create_dir_all(&fw_dir).context("Failed to create .fw directory")?;

    let state = ModState {
        mod_name: name,
        mod_id: mod_id.clone(),
        namespace: mod_id,
        package_name,
        minecraft_version: version,
        advanced_options,
        ..Default::default()
    };

    let state_path = fw_dir.join("fabric-writer.yml");
    state::save_to(&state_path, &state)?;

    println!("\nDone! Your mod project is at: {}", target_dir.display());
    println!("Next step: cd {}", target_dir.display());

    Ok(())
}

fn validate_name(name: &str) -> anyhow::Result<()> {
    if name.trim().is_empty() {
        bail!("Mod name cannot be empty.");
    }

    if name.contains(' ') {
        bail!("Mod name cannot contain spaces.");
    }

    Ok(())
}

fn validate_version(version: &str, dangerous: bool) -> anyhow::Result<()> {
    if dangerous {
        return Ok(());
    }

    if !SUPPORTED_VERSIONS.contains(&version) {
        bail!(
            "Unsupported Minecraft version: {version}\n\
             Supported versions for this build: {list}\n\
             Use --dangerous to skip validation while testing.",
            list = SUPPORTED_VERSIONS.join(", ")
        );
    }

    Ok(())
}

fn deno_on_path() -> bool {
    which::which("deno").is_ok()
}

// Fabric modid: lowercase, keep alphanumeric + `_` + `-`.
fn to_mod_id(name: &str) -> String {
    name.to_lowercase()
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '_' || *c == '-')
        .collect()
}

// Java package name: lowercase, keep alphanumeric + `_`, drop `-`.
fn to_package_name(name: &str) -> String {
    name.to_lowercase()
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '_')
        .collect()
}
