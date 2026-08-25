use crate::state::{self, Item, ModState};
use anyhow::{bail, Context};
use clap::Parser;
use genco::prelude::*;
use genco::fmt;
use std::collections::HashSet;
use std::path::PathBuf;

#[derive(Parser)]
pub struct ItemAddArgs {
    pub id: String,
}

pub fn run_item(args: ItemAddArgs) -> anyhow::Result<()> {
    let mut state = match state::load() {
        Ok(s) => s,
        Err(_) => {
            bail!("A fabric-writer project doesn't seem to exist here.\n\
                   Did you forget to cd into your project after initiation?\n\
                   Are you running from the correct directory?\n\
                   Please either move to the correct directory, or initialize a new project, and try again.")
        }
    };

    let id = args.id.trim().to_lowercase();
    if id.is_empty() {
        bail!("Item id cannot be empty.");
    }

    let existing: HashSet<_> = state.items.iter().map(|i| i.id.as_str()).collect();
    if existing.contains(id.as_str()) {
        bail!("Item '{}' already exists.", id);
    }

    let item = Item {
        id: id.clone(),
        material: "minecraft:air".to_string(),
        damage: 0,
        durability: 0,
    };

    state.items.push(item.clone());

    let state_path = PathBuf::from(".fw/fabric-writer.yml");
    state::save_to(&state_path, &state)?;

    let java_root = PathBuf::from("src/main/java").join(state.package_name.replace('.', "/"));
    std::fs::create_dir_all(&java_root).context("Failed to create Java source directory")?;

    let item_ids_path = java_root.join("ModItemIds.java");
    let items_path = java_root.join("ModItems.java");
    let mod_class_path = java_root.join(format!("{}.java", state.mod_name));

    write_mod_item_ids(&item_ids_path, &state)?;
    write_mod_items(&items_path, &state)?;
    write_main_mod_class(&mod_class_path, &state)?;

    println!("Added item: {}", item.id);

    Ok(())
}

fn to_upper(item: &Item) -> String {
    item.id.to_uppercase().replace('-', "_")
}

fn write_mod_item_ids(path: &PathBuf, state: &ModState) -> anyhow::Result<()> {
    let registries = &java::import("net.minecraft.core.registries", "Registries");
    let identifier = &java::import("net.minecraft.resources", "Identifier");
    let resource_key = &java::import("net.minecraft.resources", "ResourceKey");
    let item = &java::import("net.minecraft.world.item", "Item");

    let mod_namespace = quoted(state.mod_name.as_str());

    let t: java::Tokens = quote! {
        public class ModItemIds {
            public static $resource_key<$item> create(String name) {
                return $resource_key.create($registries.ITEM, $identifier.fromNamespaceAndPath($mod_namespace, name));
            }

            $("// Item Resource Keys")
            $(for i in &state.items =>
                public static final $resource_key<$item> $(to_upper(i)) = create($(quoted(i.id.as_str()))); $['\r']
            )
        }
    };

    let config = java::Config::default().with_package(state.package_name.as_str());
    let fmt_config = fmt::Config::from_lang::<java::Java>();
    let mut buf = Vec::new();
    let mut writer = fmt::IoWriter::new(&mut buf);
    t.format_file(&mut writer.as_formatter(&fmt_config), &config)?;

    std::fs::write(path, buf).context("Failed to write ModItemIds.java")?;

    Ok(())
}

fn write_mod_items(path: &PathBuf, state: &ModState) -> anyhow::Result<()> {
    let function = &java::import("java.util.function", "Function");
    let registry = &java::import("net.minecraft.core", "Registry");
    let built_in_registries = &java::import("net.minecraft.core.registries", "BuiltInRegistries");
    let resource_key = &java::import("net.minecraft.resources", "ResourceKey");
    let item = &java::import("net.minecraft.world.item", "Item");

    let t: java::Tokens = quote! {
        public class ModItems {
            public static $item register($resource_key<Item> itemKey, $function<Item.Properties, Item> itemFactory, Item.Properties settings) {
                $item item = itemFactory.apply(settings.setId(itemKey));
                $registry.register($built_in_registries.ITEM, itemKey, item);
                return item;
            }

            public static void initialize() {}

            $(for i in &state.items =>
                public static final $item $(to_upper(i)) = register(ModItemIds.$(to_upper(i)), $item::new, new $item.Properties());$['\r'])
        }
    };

    let config = java::Config::default().with_package(state.package_name.as_str());
    let fmt_config = fmt::Config::from_lang::<java::Java>();
    let mut buf = Vec::new();
    let mut writer = fmt::IoWriter::new(&mut buf);
    t.format_file(&mut writer.as_formatter(&fmt_config), &config)?;

    std::fs::write(path, buf).context("Failed to write ModItems.java")?;

    Ok(())
}

fn write_main_mod_class(path: &PathBuf, state: &ModState) -> anyhow::Result<()> {
    let mod_class = state.mod_name.as_str();
    let mod_id = state.mod_id.as_str();

    let mod_initializer = &java::import("net.fabricmc.api", "ModInitializer");
    let identifier = &java::import("net.minecraft.resources", "Identifier");
    let logger = &java::import("org.slf4j", "Logger");
    let logger_factory = &java::import("org.slf4j", "LoggerFactory");

    let mod_id_literal = quoted(mod_id);

    let t: java::Tokens = quote! {
        public class $mod_class implements $mod_initializer {
            public static final String MOD_ID = $mod_id_literal;

            public static final $logger LOGGER = $logger_factory.getLogger(MOD_ID);

            @Override
            public void onInitialize() {
                LOGGER.info("Hello Fabric world!");

                $("// Initializing items")
                ModItems.initialize();
            }

            public static $identifier id(String path) {
                return Identifier.fromNamespaceAndPath(MOD_ID, path);
            }
        }
    };

    let config = java::Config::default().with_package(state.package_name.as_str());
    let fmt_config = fmt::Config::from_lang::<java::Java>();
    let mut buf = Vec::new();
    let mut writer = fmt::IoWriter::new(&mut buf);
    t.format_file(&mut writer.as_formatter(&fmt_config), &config)?;

    std::fs::write(path, buf).context("Failed to write main mod class")?;

    Ok(())
}
