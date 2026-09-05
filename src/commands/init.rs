use crate::state::{ModState, save_to};
use anyhow::{Context, Result, bail};
use clap::{ArgAction, Parser};
use std::env::current_dir;
use std::fs::{create_dir_all, read_to_string};
use std::path::PathBuf;
use std::process::{Command, Stdio};

const SUPPORTED_VERSIONS: &[&str] = &["26.2"];

const MIN_JAVA_BY_VERSION: &[(&str, u32)] = &[("26.2", 25)];

/// Creates a new Fabric mod project via the official Fabric CLI, then writes
/// `.fw/fabric-writer.yml` and injects `org.gradle.java.home`.
pub fn run(args: InitArgs) -> Result<()> {
    let InitArgs {
        name,
        version,
        options,
        dangerous,
        dir,
        java_path,
    } = args;

    let base_dir = match dir {
        Some(d) => PathBuf::from(d),
        None => current_dir()?,
    };

    validate_name(&name)?;
    validate_version(&version, dangerous)?;

    if !deno_on_path() {
        bail!(
            "Deno is not installed or not on PATH.\n\
             Install it from https://deno.land/manual/getting_started/installation"
        );
    }

    validate_java(&java_path, &version)?;

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
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
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
    create_dir_all(&fw_dir).context("Failed to create .fw directory")?;

    let gradle_properties = target_dir.join("gradle.properties");
    let java_home_line = format!("org.gradle.java.home={}", java_path);

    let existing = read_to_string(&gradle_properties).unwrap_or_default();
    if !existing.contains("org.gradle.java.home") {
        let new_contents = format!("{existing}{java_home_line}\n");
        std::fs::write(&gradle_properties, new_contents)
            .context("Failed to write gradle.properties")?;
    }

    let state = ModState {
        mod_name: name,
        mod_id: mod_id.clone(),
        namespace: mod_id,
        package_name,
        minecraft_version: version,
        advanced_options,
        java_path,
        ..Default::default()
    };

    let state_path = fw_dir.join("fabric-writer.yml");
    save_to(&state_path, &state)?;

    println!("\nDone! Your mod project is at: {}", target_dir.display());
    println!("Next step: cd {}", target_dir.display());

    Ok(())
}

fn validate_name(name: &str) -> Result<()> {
    if name.trim().is_empty() {
        bail!("Mod name cannot be empty.");
    }

    if name.contains(' ') {
        bail!("Mod name cannot contain spaces.");
    }

    Ok(())
}

fn validate_version(version: &str, dangerous: bool) -> Result<()> {
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

fn validate_java(java_path: &str, minecraft_version: &str) -> Result<()> {
    let path = PathBuf::from(java_path);
    let java_bin = if cfg!(windows) {
        path.join("bin").join("java.exe")
    } else {
        path.join("bin").join("java")
    };

    if !java_bin.exists() {
        bail!(
            "Java runtime not found at: {}\n\
             Download a JDK from https://adoptium.net/",
            java_bin.display()
        );
    }

    let output = Command::new(&java_bin)
        .arg("-version")
        .stderr(Stdio::piped())
        .stdout(Stdio::null())
        .output()
        .context("Failed to run java -version")?;

    if !output.status.success() {
        bail!(
            "java -version failed for: {}\n\
             Download a JDK from https://adoptium.net/",
            java_bin.display()
        );
    }

    let version_text = String::from_utf8_lossy(&output.stderr);
    if !version_text.contains("version") {
        bail!(
            "Could not detect Java version from: {}\n\
             Download a JDK from https://adoptium.net/",
            java_bin.display()
        );
    }

    println!("Using Java: {}", version_text.trim());

    if let Some(major) = parse_java_major_version(&version_text) {
        let min_java = MIN_JAVA_BY_VERSION
            .iter()
            .find(|(v, _)| *v == minecraft_version)
            .map(|(_, min)| *min);

        let min_java = match min_java {
            Some(v) => v,
            None => bail!(
                "Unsupported Minecraft version: {mc}\n\
                 Supported versions for this build: {list}\n\
                 Use --dangerous to skip validation while testing.",
                mc = minecraft_version,
                list = SUPPORTED_VERSIONS.join(", ")
            ),
        };

        if major < min_java {
            bail!(
                "Java {major} is too old for Minecraft {mc}.\n\
                 Minimum required: Java {min_java}.\n\
                 Download a compatible JDK from https://adoptium.net/",
                major = major,
                mc = minecraft_version,
                min_java = min_java
            );
        }
    }

    Ok(())
}

fn parse_java_major_version(version_text: &str) -> Option<u32> {
    let version_str = version_text.lines().next()?.split('"').nth(1)?;

    let full = version_str.split('.').next()?;
    if let Ok(v) = full.parse::<u32>() {
        return Some(v);
    }

    None
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

/// CLI arguments for `fw init`.
#[derive(Parser)]
pub struct InitArgs {
    /// Name of your Minecraft Mod
    #[arg(short = 'n', long)]
    pub name: String,

    /// Minecraft Version
    #[arg(short = 'v', long)]
    pub version: String,

    /// Advanced Options, usually not recommended.
    #[arg(short = 'o', long = "option", action = ArgAction::Append)]
    pub options: Vec<String>,

    /// Subdirectory to create the mod in. Defaults to cwd
    #[arg(short = 'd', long)]
    pub dir: Option<String>,

    /// Path to the java install.
    #[arg(short = 'j', long)]
    pub java_path: String,

    /// Ignores various safety checks in the code. Not recommended.
    #[arg(long)]
    pub dangerous: bool,
}
