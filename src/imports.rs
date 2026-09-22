#[rustfmt::skip]
mod imports_inner {
    use genco::lang::java::{import, Import};
    use std::sync::LazyLock;

    macro_rules! static_import {
        ($const_name:ident, $fn_name:ident, $pkg:literal, $cls:literal) => {
            static $const_name: LazyLock<Import> = LazyLock::new(|| import($pkg, $cls));
            pub(crate) fn $fn_name() -> &'static Import {
                &$const_name
            }
        };
    }

    macro_rules! dynamic_import {
        ($fn_name:ident, $name:literal) => {
            pub fn $fn_name(state: &crate::state::ModState) -> Import {
                import(&state.mod_id, $name)
            }
        };
    }

    static_import!(REGISTRIES, registries, "net.minecraft.core.registries", "Registries");
    static_import!(IDENTIFIER, identifier, "net.minecraft.resources", "Identifier");
    static_import!(RESOURCE_KEY, resource_key, "net.minecraft.resources", "ResourceKey");
    static_import!(ITEM, item, "net.minecraft.world.item", "Item");
    static_import!(FUNCTION, function, "java.util.function", "Function");
    static_import!(CONSUMER, consumer, "java.util.function", "Consumer");
    static_import!(REGISTRY, registry, "net.minecraft.core", "Registry");
    static_import!(BUILT_IN_REGISTRIES, built_in_registries, "net.minecraft.core.registries", "BuiltInRegistries");
    static_import!(MOD_INITIALIZER, mod_initializer, "net.fabricmc.api", "ModInitializer");
    static_import!(LOGGER, logger, "org.slf4j", "Logger");
    static_import!(LOGGER_FACTORY, logger_factory, "org.slf4j", "LoggerFactory");
    static_import!(FABRIC_PACK_OUTPUT, fabric_pack_output, "net.fabricmc.fabric.api.datagen.v1", "FabricPackOutput");
    static_import!(FABRIC_LANGUAGE_PROVIDER, fabric_language_provider, "net.fabricmc.fabric.api.datagen.v1.provider", "FabricLanguageProvider");
    static_import!(HOLDER_LOOKUP, holder_lookup, "net.minecraft.core", "HolderLookup");
    static_import!(COMPLETABLE_FUTURE, completable_future, "java.util.concurrent", "CompletableFuture");
    static_import!(DATA_GENERATOR_ENTRYPOINT, data_generator_entrypoint, "net.fabricmc.fabric.api.datagen.v1", "DataGeneratorEntrypoint");
    static_import!(FABRIC_DATA_GENERATOR, fabric_data_generator, "net.fabricmc.fabric.api.datagen.v1", "FabricDataGenerator");
    static_import!(FABRIC_MODEL_PROVIDER, fabric_model_provider, "net.fabricmc.fabric.api.client.datagen.v1.provider", "FabricModelProvider");
    static_import!(MODEL_TEMPLATES, model_templates, "net.minecraft.client.data.models.model", "ModelTemplates");
    static_import!(BLOCK, block, "net.minecraft.world.level.block", "Block");
    static_import!(BLOCKS, blocks, "net.minecraft.world.level.block", "Blocks");
    static_import!(BLOCK_ITEM, block_item, "net.minecraft.world.item", "BlockItem");
    static_import!(BLOCK_BEHAVIOUR, block_behaviour, "net.minecraft.world.level.block.state", "BlockBehaviour");
    static_import!(BLOCK_ITEM_ID, block_item_id, "net.minecraft.references", "BlockItemId");
    static_import!(BLOCK_MODEL_GENERATORS, block_model_generators, "net.minecraft.client.data.models", "BlockModelGenerators");
    static_import!(ITEM_MODEL_GENERATORS, item_model_generators, "net.minecraft.client.data.models", "ItemModelGenerators");
    static_import!(CREATIVE_MODE_TAB_EVENTS, creative_mode_tab_events, "net.fabricmc.fabric.api.creativetab.v1", "CreativeModeTabEvents");
    static_import!(CREATIVE_MODE_TABS, creative_mode_tabs, "net.minecraft.world.item", "CreativeModeTabs");
    static_import!(FABRIC_RECIPE_PROVIDER, fabric_recipe_provider, "net.fabricmc.fabric.api.datagen.v1.provider", "FabricRecipeProvider");
    static_import!(RECIPE_PROVIDER, recipe_provider, "net.minecraft.data.recipes", "RecipeProvider");
    static_import!(RECIPE_OUTPUT, recipe_output, "net.minecraft.data.recipes", "RecipeOutput");
    static_import!(RECIPE_CATEGORY, recipe_category, "net.minecraft.data.recipes", "RecipeCategory");
    static_import!(INGREDIENT, ingredient, "net.minecraft.world.item.crafting", "Ingredient");
    static_import!(ITEMS, items, "net.minecraft.world.item", "Items");
    static_import!(TOOL_MATERIALS, tool_materials, "net.minecraft.world.item", "ToolMaterial");
    static_import!(ITEM_STACK, item_stack, "net.minecraft.world.item", "ItemStack");
    static_import!(COMPONENT, component, "net.minecraft.network.chat", "Component");
    static_import!(CHAT_FORMATTING, chat_formatting, "net.minecraft", "ChatFormatting");
    static_import!(TOOLTIP_FLAG, tooltip_flag, "net.minecraft.world.item", "TooltipFlag");
    static_import!(TOOLTIP_DISPLAY, tooltip_display, "net.minecraft.world.item.component", "TooltipDisplay");
    static_import!(SPAWN_EGG_ITEM, spawn_egg_item, "net.minecraft.world.item", "SpawnEggItem");
    static_import!(AXE_ITEM, axe_item, "net.minecraft.world.item", "AxeItem");
    static_import!(SHOVEL_ITEM, shovel_item, "net.minecraft.world.item", "ShovelItem");
    static_import!(HOE_ITEM, hoe_item, "net.minecraft.world.item", "HoeItem");
    static_import!(ARMOR_MATERIALS, armor_materials, "net.minecraft.world.item.equipment", "ArmorMaterials");
    static_import!(ARMOR_TYPE, armor_type, "net.minecraft.world.item.equipment", "ArmorType");
    static_import!(SHIELD_ITEM, shield_item, "net.minecraft.world.item", "ShieldItem");
    static_import!(ENTITY_TYPES, entity_types, "net.minecraft.world.entity", "EntityTypes");
    static_import!(DATA_COMPONENTS, data_components, "net.minecraft.core.component", "DataComponents");
    static_import!(FOOD_PROPERTIES, food_properties, "net.minecraft.world.food", "FoodProperties");
    static_import!(CONSUMABLES, consumables, "net.minecraft.world.item.component", "Consumables");
    static_import!(POTION_CONTENTS, potion_contents, "net.minecraft.world.item.alchemy", "PotionContents");
    static_import!(MOB_EFFECT_INSTANCE, mob_effect_instance, "net.minecraft.world.effect", "MobEffectInstance");
    static_import!(MOB_EFFECTS, mob_effects, "net.minecraft.world.effect", "MobEffects");
    static_import!(LIST, list, "java.util", "List");
    static_import!(COMPOSTABLE_REGISTRY, compostable_registry, "net.fabricmc.fabric.api.registry", "CompostableRegistry");
    static_import!(FUEL_VALUE_EVENTS, fuel_value_events, "net.fabricmc.fabric.api.registry", "FuelValueEvents");
    static_import!(FABRIC_CREATIVE_MODE_TAB, fabric_creative_mode_tab, "net.fabricmc.fabric.api.creativetab.v1", "FabricCreativeModeTab");
    static_import!(CREATIVE_MODE_TAB, creative_mode_tab, "net.minecraft.world.item", "CreativeModeTab");

    dynamic_import!(mod_blocks, "ModBlocks");
    dynamic_import!(mod_items, "ModItems");
    dynamic_import!(mod_creative_tabs, "ModCreativeTabs");

    /// Creates an import for the main mod class (e.g. `import testmod.TestMod;`).
    pub fn mod_class(state: &crate::state::ModState) -> Import {
        import(&state.mod_id, &state.mod_name)
    }
}

pub(crate) use imports_inner::*;
