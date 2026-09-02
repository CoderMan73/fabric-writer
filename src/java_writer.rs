use anyhow::{Context, Result};
use genco::fmt::{self, IoWriter};
use genco::lang::java::{self, Import, Java, Tokens, import};
use std::fs::{create_dir_all, write};
use std::path::PathBuf;

use crate::state::ModState;
use crate::tokengen::{
    BuildFn, build_datagen_entrypoint, build_lang_provider, build_main_mod_class, build_mod_blocks,
    build_mod_item_ids, build_mod_items, build_model_provider,
};

pub struct JavaWriter {
    pub registries: Import,
    pub identifier: Import,
    pub resource_key: Import,
    pub item: Import,
    pub function: Import,
    pub registry: Import,
    pub built_in_registries: Import,
    pub mod_initializer: Import,
    pub logger: Import,
    pub logger_factory: Import,
    pub fabric_pack_output: Import,
    pub fabric_language_provider: Import,
    pub holder_lookup: Import,
    pub completable_future: Import,
    pub data_generator_entrypoint: Import,
    pub fabric_data_generator: Import,
    pub fabric_model_provider: Import,
    pub model_templates: Import,
    pub block: Import,
    pub block_item: Import,
    pub block_behaviour: Import,
}

impl JavaWriter {
    pub fn new() -> Self {
        Self {
            registries: import("net.minecraft.core.registries", "Registries"),
            identifier: import("net.minecraft.resources", "Identifier"),
            resource_key: import("net.minecraft.resources", "ResourceKey"),
            item: import("net.minecraft.world.item", "Item"),
            function: import("java.util.function", "Function"),
            registry: import("net.minecraft.core", "Registry"),
            built_in_registries: import("net.minecraft.core.registries", "BuiltInRegistries"),
            mod_initializer: import("net.fabricmc.api", "ModInitializer"),
            logger: import("org.slf4j", "Logger"),
            logger_factory: import("org.slf4j", "LoggerFactory"),
            holder_lookup: import("net.minecraft.core", "HolderLookup"),
            completable_future: import("java.util.concurrent", "CompletableFuture"),
            fabric_pack_output: import("net.fabricmc.fabric.api.datagen.v1", "FabricPackOutput"),
            fabric_language_provider: import(
                "net.fabricmc.fabric.api.datagen.v1.provider",
                "FabricLanguageProvider",
            ),
            data_generator_entrypoint: import(
                "net.fabricmc.fabric.api.datagen.v1",
                "DataGeneratorEntrypoint",
            ),
            fabric_data_generator: import(
                "net.fabricmc.fabric.api.datagen.v1",
                "FabricDataGenerator",
            ),
            fabric_model_provider: import(
                "net.fabricmc.fabric.api.datagen.v1.provider",
                "FabricModelProvider",
            ),
            model_templates: import("net.minecraft.data.client", "ModelTemplates"),
            block: import("net.minecraft.world.level.block", "Block"),
            block_item: import("net.minecraft.world.item", "BlockItem"),
            block_behaviour: import("net.minecraft.world.level.block", "BlockBehaviour"),
        }
    }

    pub fn write(path: &PathBuf, tokens: Tokens, package: &str) -> Result<()> {
        if let Some(parent) = path.parent() {
            create_dir_all(parent).context("Failed to create Java output directory")?;
        }
        let config = java::Config::default().with_package(package);
        let fmt_config = fmt::Config::from_lang::<Java>();
        let mut buf = Vec::new();
        let mut writer = IoWriter::new(&mut buf);
        tokens.format_file(&mut writer.as_formatter(&fmt_config), &config)?;
        write(path, buf).context("Failed to write Java file")?;
        Ok(())
    }
}

impl Default for JavaWriter {
    fn default() -> Self {
        Self::new()
    }
}

pub fn regenerate_all(state: &ModState) -> Result<()> {
    let java_root = PathBuf::from("src/main/java").join(state.package_name.replace('.', "/"));
    let client_root = PathBuf::from("src/client/java").join(state.package_name.replace('.', "/"));
    std::fs::create_dir_all(&java_root).context("Failed to create Java source directory")?;
    std::fs::create_dir_all(&client_root)
        .context("Failed to create client Java source directory")?;

    let codegen = JavaWriter::new();
    let package = state.package_name.as_str();
    let client_package = package.to_owned() + ".client";

    let ids = java_root.join("ModItemIds.java");
    let items = java_root.join("ModItems.java");
    let mod_class = java_root.join(format!("{}.java", state.mod_name));
    let lang = client_root.join("client/LangProvider.java");
    let datagen = client_root.join("client/DataGeneratorEntrypoint.java");
    let models = client_root.join("client/ModelProvider.java");
    let blocks = java_root.join("ModBlocks.java");

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
    ];

    for (path, build, pkg) in to_write {
        JavaWriter::write(path, build(state, &codegen), pkg)?;
    }

    Ok(())
}
