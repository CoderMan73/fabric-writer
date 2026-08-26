use crate::java_codegen::JavaCodegen;
use crate::state::{self, Item, ModState};
use anyhow::{bail, Context};
use clap::Parser;
use genco::prelude::*;
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

    let codegen = JavaCodegen::new();

    write_mod_item_ids(&item_ids_path, &state, &codegen)?;
    write_mod_items(&items_path, &state, &codegen)?;
    write_main_mod_class(&mod_class_path, &state, &codegen)?;

    println!("Added item: {}", item.id);

    Ok(())
}

fn to_upper(item: &Item) -> String {
    item.id.to_uppercase().replace('-', "_")
}

fn write_mod_item_ids(path: &PathBuf, state: &ModState, codegen: &JavaCodegen) -> anyhow::Result<()> {
    let registries = &codegen.registries;
    let identifier = &codegen.identifier;
    let resource_key = &codegen.resource_key;
    let item = &codegen.item;

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

    JavaCodegen::write(path, t, state.package_name.as_str())
}

fn write_mod_items(
    path: &PathBuf,
    state: &ModState,
    codegen: &JavaCodegen,
) -> anyhow::Result<()> {
    let function = &codegen.function;
    let registry = &codegen.registry;
    let built_in_registries = &codegen.built_in_registries;
    let resource_key = &codegen.resource_key;
    let item = &codegen.item;

    let t: java::Tokens = quote! {
        public class ModItems {
            public static $item register($resource_key<$item> itemKey, $function<$item.Properties, $item> itemFactory, $item.Properties settings) {
                $item item = itemFactory.apply(settings.setId(itemKey));
                $registry.register($built_in_registries.ITEM, itemKey, item);
                return item;
            }

            public static void initialize() {}

            $("// Item Registration")
            $(for i in &state.items =>
                public static final $item $(to_upper(i)) = register(ModItemIds.$(to_upper(i)), $item::new, new $item.Properties());$['\r']
            )
        }
    };

    JavaCodegen::write(path, t, state.package_name.as_str())
}

fn write_main_mod_class(
    path: &PathBuf,
    state: &ModState,
    codegen: &JavaCodegen,
) -> anyhow::Result<()> {
    let mod_class = state.mod_name.as_str();
    let mod_id = state.mod_id.as_str();

    let mod_initializer = &codegen.mod_initializer;
    let identifier = &codegen.identifier;
    let logger = &codegen.logger;
    let logger_factory = &codegen.logger_factory;

    let t: java::Tokens = quote! {
        public class $mod_class implements $mod_initializer {
            public static final String MOD_ID = $(quoted(mod_id));

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

    JavaCodegen::write(path, t, state.package_name.as_str())
}
