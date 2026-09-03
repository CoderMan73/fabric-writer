use anyhow::{Context, Result};
use genco::fmt::{self, IoWriter};
use genco::lang::java::{self, Java, Tokens};
use std::fs::{create_dir_all, write as fs_write};
use std::path::PathBuf;

use crate::state::ModState;
use crate::tokengen::{
    BuildFn, build_datagen_entrypoint, build_lang_provider, build_main_mod_class,
    build_mod_block_ids, build_mod_block_item_ids, build_mod_blocks, build_mod_item_ids,
    build_mod_items, build_model_provider,
};

pub fn write(path: &PathBuf, tokens: Tokens, package: &str) -> Result<()> {
    if let Some(parent) = path.parent() {
        create_dir_all(parent).context("Failed to create Java output directory")?;
    }
    let config = java::Config::default().with_package(package);
    let fmt_config = fmt::Config::from_lang::<Java>();
    let mut buf = Vec::new();
    let mut writer = IoWriter::new(&mut buf);
    tokens.format_file(&mut writer.as_formatter(&fmt_config), &config)?;
    fs_write(path, buf).context("Failed to write Java file")?;
    Ok(())
}

pub fn regenerate_all(state: &ModState) -> Result<()> {
    let java_root = PathBuf::from("src/main/java").join(state.package_name.replace('.', "/"));
    let client_root = PathBuf::from("src/client/java").join(state.package_name.replace('.', "/"));
    std::fs::create_dir_all(&java_root).context("Failed to create Java source directory")?;
    std::fs::create_dir_all(&client_root)
        .context("Failed to create client Java source directory")?;

    let package = state.package_name.as_str();
    let client_package = package.to_owned() + ".client";

    let ids = java_root.join("ModItemIds.java");
    let items = java_root.join("ModItems.java");
    let mod_class = java_root.join(format!("{}.java", state.mod_name));
    let lang = client_root.join("client/LangProvider.java");
    let datagen = client_root.join(format!("client/{}DataGenerator.java", state.mod_name));
    let models = client_root.join("client/ModelProvider.java");
    let blocks = java_root.join("ModBlocks.java");
    let block_ids = java_root.join("ModBlockIds.java");
    let block_item_ids = java_root.join("ModBlockItemIds.java");

    // TODO: extract somewhere and add a
    // dirty bit so we only rewrite changed files instead of everything
    let to_write: &[(&PathBuf, BuildFn, &str)] = &[
        (&ids, build_mod_item_ids, package),
        (&items, build_mod_items, package),
        (&mod_class, build_main_mod_class, package),
        (&lang, build_lang_provider, &client_package),
        (&datagen, build_datagen_entrypoint, &client_package),
        (&models, build_model_provider, &client_package),
        (&blocks, build_mod_blocks, package),
        (&block_ids, build_mod_block_ids, package),
        (&block_item_ids, build_mod_block_item_ids, package),
    ];

    for (path, build, pkg) in to_write {
        write(path, build(state), pkg)?;
    }

    Ok(())
}
