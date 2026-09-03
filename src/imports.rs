#[rustfmt::skip]
mod import_fns {
    use genco::lang::java::{import, Import};
    use std::sync::LazyLock;

    macro_rules! import_fn {
        ($const_name:ident, $fn_name:ident, $pkg:literal, $cls:literal) => {
            static $const_name: LazyLock<Import> = LazyLock::new(|| import($pkg, $cls));
            pub(crate) fn $fn_name() -> &'static Import {
                &$const_name
            }
        };
    }

    import_fn!(REGISTRIES, registries, "net.minecraft.core.registries", "Registries");
    import_fn!(IDENTIFIER, identifier, "net.minecraft.resources", "Identifier");
    import_fn!(RESOURCE_KEY, resource_key, "net.minecraft.resources", "ResourceKey");
    import_fn!(ITEM, item, "net.minecraft.world.item", "Item");
    import_fn!(FUNCTION, function, "java.util.function", "Function");
    import_fn!(REGISTRY, registry, "net.minecraft.core", "Registry");
    import_fn!(BUILT_IN_REGISTRIES, built_in_registries, "net.minecraft.core.registries", "BuiltInRegistries");
    import_fn!(MOD_INITIALIZER, mod_initializer, "net.fabricmc.api", "ModInitializer");
    import_fn!(LOGGER, logger, "org.slf4j", "Logger");
    import_fn!(LOGGER_FACTORY, logger_factory, "org.slf4j", "LoggerFactory");
    import_fn!(FABRIC_PACK_OUTPUT, fabric_pack_output, "net.fabricmc.fabric.api.datagen.v1", "FabricPackOutput");
    import_fn!(FABRIC_LANGUAGE_PROVIDER, fabric_language_provider, "net.fabricmc.fabric.api.datagen.v1.provider", "FabricLanguageProvider");
    import_fn!(HOLDER_LOOKUP, holder_lookup, "net.minecraft.core", "HolderLookup");
    import_fn!(COMPLETABLE_FUTURE, completable_future, "java.util.concurrent", "CompletableFuture");
    import_fn!(DATA_GENERATOR_ENTRYPOINT, data_generator_entrypoint, "net.fabricmc.fabric.api.datagen.v1", "DataGeneratorEntrypoint");
    import_fn!(FABRIC_DATA_GENERATOR, fabric_data_generator, "net.fabricmc.fabric.api.datagen.v1", "FabricDataGenerator");
    import_fn!(FABRIC_MODEL_PROVIDER, fabric_model_provider, "net.fabricmc.fabric.api.client.datagen.v1.provider", "FabricModelProvider");
    import_fn!(MODEL_TEMPLATES, model_templates, "net.minecraft.client.data.models.model", "ModelTemplates");
    import_fn!(BLOCK, block, "net.minecraft.world.level.block", "Block");
    import_fn!(BLOCK_ITEM, block_item, "net.minecraft.world.item", "BlockItem");
    import_fn!(BLOCK_BEHAVIOUR, block_behaviour, "net.minecraft.world.level.block.state", "BlockBehaviour");
    import_fn!(BLOCK_ITEM_ID, block_item_id, "net.minecraft.references", "BlockItemId");
    import_fn!(BLOCKS, blocks, "net.minecraft.world.level.block", "Blocks");
    import_fn!(BLOCK_MODEL_GENERATORS, block_model_generators, "net.minecraft.client.data.models", "BlockModelGenerators");
    import_fn!(ITEM_MODEL_GENERATORS, item_model_generators, "net.minecraft.client.data.models", "ItemModelGenerators");
}

mod special_imports {
    use crate::state::ModState;
    use genco::lang::java::{Import, import};

    macro_rules! import_fn {
        ($fn_name:ident, $name:literal) => {
            pub fn $fn_name(state: &ModState) -> Import {
                import(&state.mod_id, $name)
            }
        };
    }

    // TODO: lowkey realized, could I have just done this for both sets of imports?
    // Thats a question for another day. Its functional now.
    import_fn!(mod_blocks, "ModBlocks");
    import_fn!(mod_items, "ModItems");
}

pub(crate) use import_fns::*;
pub(crate) use special_imports::*;
