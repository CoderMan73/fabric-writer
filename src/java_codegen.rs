use genco::lang::java::{self, Import};
use genco::fmt;
use anyhow::Context;
use std::path::PathBuf;

pub struct JavaCodegen {
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
}

impl JavaCodegen {
    pub fn new() -> Self {
        Self {
            registries: java::import("net.minecraft.core.registries", "Registries"),
            identifier: java::import("net.minecraft.resources", "Identifier"),
            resource_key: java::import("net.minecraft.resources", "ResourceKey"),
            item: java::import("net.minecraft.world.item", "Item"),
            function: java::import("java.util.function", "Function"),
            registry: java::import("net.minecraft.core", "Registry"),
            built_in_registries: java::import("net.minecraft.core.registries", "BuiltInRegistries"),
            mod_initializer: java::import("net.fabricmc.api", "ModInitializer"),
            logger: java::import("org.slf4j", "Logger"),
            logger_factory: java::import("org.slf4j", "LoggerFactory"),
            holder_lookup: java::import("net.minecraft.core", "HolderLookup"),
            completable_future: java::import("java.util.concurrent", "CompletableFuture"),
            fabric_pack_output: java::import("net.fabricmc.fabric.api.datagen.v1", "FabricPackOutput"),
            fabric_language_provider: java::import("net.fabricmc.fabric.api.datagen.v1.provider", "FabricLanguageProvider"),
            data_generator_entrypoint: java::import("net.fabricmc.fabric.api.datagen.v1", "DataGeneratorEntrypoint"),
            fabric_data_generator: java::import("net.fabricmc.fabric.api.datagen.v1", "FabricDataGenerator"),
        }
    }

    pub fn write(path: &PathBuf, tokens: genco::lang::java::Tokens, package: &str) -> anyhow::Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).context("Failed to create Java output directory")?;
        }
        let config = java::Config::default().with_package(package);
        let fmt_config = fmt::Config::from_lang::<java::Java>();
        let mut buf = Vec::new();
        let mut writer = fmt::IoWriter::new(&mut buf);
        tokens.format_file(&mut writer.as_formatter(&fmt_config), &config)?;
        std::fs::write(path, buf).context("Failed to write Java file")?;
        Ok(())
    }
}
