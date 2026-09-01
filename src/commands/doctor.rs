// TODO: something like this, but its lowkey not working rn
use std::env::current_dir;

use crate::state;
use anyhow::Result;

#[allow(dead_code)]
pub fn run() -> Result<()> {
    let state = state::load()?;

    let root = current_dir()?;
    let java_root = root
        .join("src\\main\\java")
        .join(state.package_name.replace('.', "/"));
    let client_root = root
        .join("src\\client\\java")
        .join(state.package_name.replace('.', "/"));

    let mut issues = Vec::new();

    if !java_root.exists() {
        issues.push(format!("Missing main java dir: {}", java_root.display()));
    }
    if !client_root.exists() {
        issues.push(format!(
            "Missing client java dir: {}",
            client_root.display()
        ));
    }

    let required_main = [
        java_root.join("ModItems.java"),
        java_root.join("ModBlocks.java"),
        java_root.join("ModMainModClass.java"),
    ];
    let required_client = [
        client_root.join("LangProvider.java"),
        client_root.join("ModelProvider.java"),
        client_root.join("DataGeneratorEntrypoint.java"),
    ];

    for path in required_main.iter().chain(required_client.iter()) {
        if !path.exists() {
            issues.push(format!("Missing generated file: {}", path.display()));
        }
    }

    if issues.is_empty() {
        println!("Project state looks healthy.");
    } else {
        println!("Issues found:");
        for issue in issues {
            println!("- {}", issue);
        }
    }

    Ok(())
}
