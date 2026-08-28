use crate::item::Item;
use crate::java_codegen::JavaCodegen;
use crate::state::{self, ModState};
use anyhow::Context;
use clap::Parser;
use genco::prelude::*;
use std::path::PathBuf;

#[derive(Parser)]
pub struct ItemAddArgs {
    pub id: String,
}

pub fn run_item(args: ItemAddArgs) -> anyhow::Result<()> {
    let mut state = state::load().context(
        "No fabric-writer project found. Are you in the right directory?",
    )?;

    let item = Item::new(&args.id)?;
    state.add_item(item.clone())?;
    state.save()?;

    let java_root = PathBuf::from("src/main/java").join(state.package_name.replace('.', "/"));
    let client_root = PathBuf::from("src/client/java").join(state.package_name.replace('.', "/"));
    std::fs::create_dir_all(&java_root).context("Failed to create Java source directory")?;
    std::fs::create_dir_all(&client_root).context("Failed to create client Java source directory")?;

    let item_ids_path = java_root.join("ModItemIds.java");
    let items_path = java_root.join("ModItems.java");
    let mod_class_path = java_root.join(format!("{}.java", state.mod_name));
    let lang_provider_path = client_root.join("client/LangProvider.java");
    let datagen_path = client_root.join("client/MyDopeModDataGenerator.java");

    let codegen = JavaCodegen::new();

    write_mod_item_ids(&item_ids_path, &state, &codegen)?;
    write_mod_items(&items_path, &state, &codegen)?;
    write_main_mod_class(&mod_class_path, &state, &codegen)?;
    write_lang_provider(&lang_provider_path, &state, &codegen)?;
    write_datagen_entrypoint(&datagen_path, &state, &codegen)?;

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

    let mod_namespace = quoted(state.mod_name.as_str().to_lowercase());

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

fn write_mod_items(path: &PathBuf, state: &ModState, codegen: &JavaCodegen) -> anyhow::Result<()> {
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

fn write_main_mod_class(path: &PathBuf, state: &ModState, codegen: &JavaCodegen) -> anyhow::Result<()> {
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
                return $identifier.fromNamespaceAndPath(MOD_ID, path);
            }
        }
    };

    JavaCodegen::write(path, t, state.package_name.as_str())
}

fn write_lang_provider(path: &PathBuf, state: &ModState, codegen: &JavaCodegen) -> anyhow::Result<()> {
    let fabric_pack_output = &codegen.fabric_pack_output;
    let fabric_language_provider = &codegen.fabric_language_provider;
    let holder_lookup = &codegen.holder_lookup;
    let completable_future = &codegen.completable_future;

    let t: java::Tokens = quote! {
        public class LangProvider extends $fabric_language_provider {
            protected LangProvider($fabric_pack_output dataOutput, $completable_future<$holder_lookup.Provider> registryLookup) {
                super(dataOutput, "en_us", registryLookup);
            }

            @Override
            public void generateTranslations($holder_lookup.Provider holderLookup, TranslationBuilder translationBuilder) {
                $(for i in &state.items =>
                    //$(&state.mod_id.as_str())$(".")$(_i.id.as_str())", "$(to_upper(_i))
                    translationBuilder.add($(quoted("item.".to_owned() + &state.mod_id.as_str() + "." + i.id.as_str())), $(quoted(to_upper(i))));$['\r']
                )
            }
        }
    };

    JavaCodegen::write(path, t, &(state.package_name.as_str().to_owned() + ".client"))
}

fn write_datagen_entrypoint(path: &PathBuf, state: &ModState, codegen: &JavaCodegen) -> anyhow::Result<()> {
    let mod_generator_class = format!("{}DataGenerator", state.mod_name.as_str());

    let data_generator_entrypoint = &codegen.data_generator_entrypoint;
    let fabric_data_generator = &codegen.fabric_data_generator;

    let t: java::Tokens = quote! {
        public class $mod_generator_class implements $data_generator_entrypoint {
            @Override
            public void onInitializeDataGenerator($fabric_data_generator fabricDataGenerator) {
                $fabric_data_generator.Pack pack = fabricDataGenerator.createPack();

                pack.addProvider(LangProvider::new);
            }
        }
    };

    JavaCodegen::write(path, t, &(state.package_name.as_str().to_owned() + ".client"))
}