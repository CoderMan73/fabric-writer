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
    import_fn!(BLOCKS, blocks, "net.minecraft.world.level.block", "Blocks");
    import_fn!(BLOCK_ITEM, block_item, "net.minecraft.world.item", "BlockItem");
    import_fn!(BLOCK_BEHAVIOUR, block_behaviour, "net.minecraft.world.level.block.state", "BlockBehaviour");
    import_fn!(BLOCK_ITEM_ID, block_item_id, "net.minecraft.references", "BlockItemId");
    import_fn!(BLOCK_MODEL_GENERATORS, block_model_generators, "net.minecraft.client.data.models", "BlockModelGenerators");
    import_fn!(ITEM_MODEL_GENERATORS, item_model_generators, "net.minecraft.client.data.models", "ItemModelGenerators");
    // import_fn!(ITEMS, items, "net.minecraft.world.item", "Items");
    import_fn!(CREATIVE_MODE_TAB_EVENTS, creative_mode_tab_events, "net.fabricmc.fabric.api.creativetab.v1", "CreativeModeTabEvents");
    import_fn!(CREATIVE_MODE_TABS, creative_mode_tabs, "net.minecraft.world.item", "CreativeModeTabs");
    import_fn!(FABRIC_RECIPE_PROVIDER, fabric_recipe_provider, "net.fabricmc.fabric.api.datagen.v1.provider", "FabricRecipeProvider");
    import_fn!(RECIPE_PROVIDER, recipe_provider, "net.minecraft.data.recipes", "RecipeProvider");
    import_fn!(RECIPE_OUTPUT, recipe_output, "net.minecraft.data.recipes", "RecipeOutput");
    import_fn!(RECIPE_CATEGORY, recipe_category, "net.minecraft.data.recipes", "RecipeCategory");
    import_fn!(INGREDIENT, ingredient, "net.minecraft.world.item.crafting", "Ingredient");
    import_fn!(ITEMS, items, "net.minecraft.world.item", "Items");
    import_fn!(TOOL_MATERIALS, tool_materials, "net.minecraft.world.item", "ToolMaterial");
    import_fn!(ITEM_STACK, item_stack, "net.minecraft.world.item", "ItemStack");
    import_fn!(COMPONENT, component, "net.minecraft.network.chat", "Component");
    import_fn!(CHAT_FORMATTING, chat_formatting, "net.minecraft", "ChatFormatting");
    import_fn!(TOOLTIP_FLAG, tooltip_flag, "net.minecraft.world.item", "TooltipFlag");
    import_fn!(TOOLTIP_DISPLAY, tooltip_display, "net.minecraft.world.item.component", "TooltipDisplay");
    import_fn!(SPAWN_EGG_ITEM, spawn_egg_item, "net.minecraft.world.item", "SpawnEggItem");
    import_fn!(AXE_ITEM, axe_item, "net.minecraft.world.item", "AxeItem");
    import_fn!(SHOVEL_ITEM, shovel_item, "net.minecraft.world.item", "ShovelItem");
    import_fn!(HOE_ITEM, hoe_item, "net.minecraft.world.item", "HoeItem");
    import_fn!(FUEL_PROVIDER, fuel_provider, "net.fabricmc.fabric.api.datagen.v1.provider", "FabricFuelProvider");
    import_fn!(COMPOSTABLE_PROVIDER, compostable_provider, "net.fabricmc.fabric.api.datagen.v1.provider", "FabricCompostableProvider");
    import_fn!(FUEL_REGISTRY, fuel_registry, "net.minecraft.world.item", "FuelRegistry");
    import_fn!(COMPOSTABLE_REGISTRY, compostable_registry, "net.minecraft.world.item", "CompostableRegistry");
    import_fn!(FABRIC_CREATIVE_MODE_TAB, fabric_creative_mode_tab, "net.fabricmc.fabric.api.creativetab.v1", "FabricCreativeModeTab");
    import_fn!(CREATIVE_MODE_TAB, creative_mode_tab, "net.minecraft.world.item", "CreativeModeTab");
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
    import_fn!(mod_creative_tabs, "ModCreativeTabs");

    /// Creates an import for the main mod class (e.g. `import testmod.TestMod;`).
    pub fn mod_class(state: &ModState) -> Import {
        import(&state.mod_id, &state.mod_name)
    }
}

pub(crate) use import_fns::*;
pub(crate) use special_imports::*;
