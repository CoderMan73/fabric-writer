use crate::imports::*;
use crate::state::{CreativeTab, Item, ItemKind, ModState, Recipe};
use genco::lang::java::Tokens;
use genco::lang::java::import;
use genco::prelude::*;
use genco::tokens::Register;
use heck::ToTitleCase;

pub(crate) type BuildFn = fn(&ModState) -> Tokens;

pub(crate) fn build_mod_item_ids(state: &ModState) -> Tokens {
    quote! {
        public class ModItemIds {
            private static $(resource_key())<$(item())> create(String name) {
                $(identifier()) id = $(identifier()).fromNamespaceAndPath($(&state.mod_name).MOD_ID, name);
                return $(resource_key()).create($(registries()).ITEM, id);
            }

            $("// Mod Item ID Registration")
            $(for i in &state.items =>
                public static final $(resource_key())<$(item())> $(to_upper(&i.id)) = create(
                    $(quoted(&i.id))
                );
            )
        }
    }
}

pub(crate) fn build_mod_items(state: &ModState) -> Tokens {
    let vanilla_items = vanilla_entities(
        &state.items,
        &state.creative_tabs,
        |i| &i.id,
        |i| i.creative_tab.as_deref() == Some("ingredients"),
    );

    quote! {
        public class ModItems {
            private static $(item()) register($(resource_key())<$(item())> itemKey, $(function())<$(item()).Properties, $(item())> itemFactory, $(item()).Properties settings) {
                $(item()) item = itemFactory.apply(settings.setId(itemKey));
                $(registry()).register($(built_in_registries()).ITEM, itemKey, item);
                return item;
            }

            $("// Item Registration")
            $(for i in &state.items =>
                $(if !i.tooltip.is_empty() =>
                    public static final $(item()) $(to_upper(&i.id)) = register(ModItemIds.$(to_upper(&i.id)), $(item_factory(i, true)), $(item_properties(i, true)));$['\r']
                )
                $(if i.tooltip.is_empty() =>
                    public static final $(item()) $(to_upper(&i.id)) = register(ModItemIds.$(to_upper(&i.id)), $(item_factory(i, false)), $(item_properties(i, false)));$['\r']
                )
            )

            public static void initialize() {
                $(if !&state.items.is_empty() && state.creative_tabs.is_empty() =>
                    $(creative_mode_tab_events()).modifyOutputEvent($(creative_mode_tabs()).INGREDIENTS)
                        .register((creativeTab) ->
                        {
                            $(for i in &state.items =>
                                creativeTab.accept($(mod_items(state)).$(to_upper(&i.id)));$['\r']
                            )
                        });
                )
                $(if !vanilla_items.is_empty() =>
                    $(creative_mode_tab_events()).modifyOutputEvent($(creative_mode_tabs()).INGREDIENTS)
                        .register((creativeTab) ->
                        {
                            $(for i in &vanilla_items =>
                                creativeTab.accept($(mod_items(state)).$(to_upper(&i.id)));$['\r']
                            )
                        });
                )
            }
        }
    }
}

pub(crate) fn build_mod_blocks(state: &ModState) -> Tokens {
    let vanilla_blocks = vanilla_entities(
        &state.blocks,
        &state.creative_tabs,
        |b| &b.id,
        |b| b.creative_tab.as_deref() == Some("ingredients"),
    );

    let props_sources: Vec<&str> = state
        .blocks
        .iter()
        .map(|b| b.properties_from.as_deref().unwrap_or("dirt"))
        .collect();

    quote! {
        public class ModBlocks {
            private static $(block()) register($(block_item_id()) id, $(function())<$(block_behaviour()).Properties, $(block())> blockFactory, $(block_behaviour()).Properties properties) {
                $(block()) block = register(id.block(), blockFactory, properties);
                $(block_item()) blockItem = new $(block_item())(block, new $(item()).Properties().useBlockDescriptionPrefix().setId(id.item()));
                $(registry()).register($(built_in_registries()).ITEM, id.item(), blockItem);
                return block;
            }

            private static $(block()) register($(resource_key())<$(block())> id, $(function())<$(block_behaviour()).Properties, $(block())> blockFactory, $(block_behaviour()).Properties properties) {
                $(block()) block = blockFactory.apply(properties.setId(id));
                return $(registry()).register($(built_in_registries()).BLOCK, id, block);
            }

            $("// Block Registration")
            $(for (b, props_src) in state.blocks.iter().zip(&props_sources) =>
                public static final $(block()) $(to_upper(&b.id)) = register(
                    ModBlockItemIds.$(to_upper(&b.id)),
                    $(block())::new,
                    $(block_behaviour()).Properties.ofFullCopy($(blocks()).$(to_upper(props_src)))
                );$['\r']
            )

            public static void initialize() {
                $(if !&state.blocks.is_empty() && state.creative_tabs.is_empty() =>
                    $(creative_mode_tab_events()).modifyOutputEvent($(creative_mode_tabs()).INGREDIENTS)
                        .register((creativeTab) ->
                        {
                            $(for b in &state.blocks =>
                                creativeTab.accept($(mod_blocks(state)).$(to_upper(&b.id)));$['\r']
                            )
                        });
                )
                $(if !vanilla_blocks.is_empty() =>
                    $(creative_mode_tab_events()).modifyOutputEvent($(creative_mode_tabs()).INGREDIENTS)
                        .register((creativeTab) ->
                        {
                            $(for b in &vanilla_blocks =>
                                creativeTab.accept($(mod_blocks(state)).$(to_upper(&b.id)));$['\r']
                            )
                        });
                )
            }
        }
    }
}

pub(crate) fn build_mod_block_item_ids(state: &ModState) -> Tokens {
    quote! {
        public class ModBlockItemIds {
            private static BlockItemId create(String name) {
                $(identifier()) id = $(identifier()).fromNamespaceAndPath($(&state.mod_name).MOD_ID, name);
                return $(block_item_id()).create(id, id);
            }

            $("// Mod Block Item ID Registration")
            $(for b in &state.blocks =>
                public static final $(block_item_id()) $(to_upper(&b.id)) = create(
                    $(quoted(&b.id))
                );
            )
        }
    }
}

pub(crate) fn build_mod_block_ids(state: &ModState) -> Tokens {
    quote! {
        public class ModBlockIds {
            private static $(resource_key())<$(block())> create(String name) {
                $(identifier()) id = $(identifier()).fromNamespaceAndPath($(&state.mod_name).MOD_ID, name);
                return $(resource_key()).create($(registries()).BLOCK, id);
            }
        }
    }
}

pub(crate) fn build_mod_creative_tabs(state: &ModState) -> Tokens {
    if state.creative_tabs.is_empty() {
        return quote! {};
    }

    let first_tab_id = &state.creative_tabs[0].id;

    let mut tab_defs = Vec::new();
    for tab in &state.creative_tabs {
        let tab_name = to_upper(&tab.id);
        let tab_id = &tab.id;

        let icon_item = if !state.items.is_empty() {
            let first_item = to_upper(&state.items[0].id);
            quote! { new $(item_stack())($(mod_items(state)).$(first_item)) }
        } else {
            quote! { new $(item_stack())($(items()).DIAMOND) }
        };

        let mut display_items = quote! {};

        for item in &state.items {
            if let Some(ct) = &item.creative_tab
                && *ct == tab.id
                && ct != "ingredients"
                && ct != "default"
            {
                quote_in! { display_items =>
                    output.accept($(mod_items(state)).$(to_upper(&item.id)));$['\r']
                }
            }
        }

        if tab.id == *first_tab_id {
            for item in &state.items {
                if item.creative_tab.is_none() || item.creative_tab.as_deref() == Some("default") {
                    quote_in! { display_items =>
                        output.accept($(mod_items(state)).$(to_upper(&item.id)));$['\r']
                    }
                }
            }
            for block in &state.blocks {
                if block.creative_tab.is_none() || block.creative_tab.as_deref() == Some("default")
                {
                    quote_in! { display_items =>
                        output.accept($(mod_blocks(state)).$(to_upper(&block.id)));$['\r']
                    }
                }
            }
        }

        for block in &state.blocks {
            if let Some(ct) = &block.creative_tab
                && *ct == tab.id
                && ct != "ingredients"
                && ct != "default"
            {
                quote_in! { display_items =>
                    output.accept($(mod_blocks(state)).$(to_upper(&block.id)));$['\r']
                }
            }
        }

        tab_defs.push(quote! {
            public static final $(resource_key())<$(creative_mode_tab())> $(&tab_name)_KEY = $(resource_key()).create(
                $(built_in_registries()).CREATIVE_MODE_TAB.key(), $(mod_class(state)).id($(quoted(&tab.id)))
            );
            public static final $(creative_mode_tab()) $(tab_name) = $(fabric_creative_mode_tab()).builder()
                .icon(() -> $(icon_item))
                .title($(component()).translatable($(quoted(&format!("creativeTab.{}.{}", state.mod_id, tab_id)))))
                .displayItems((params, output) -> {
                    $display_items
                })
                .build();
        });
    }

    let mut registration = quote! {};
    for tab in &state.creative_tabs {
        let tab_name = to_upper(&tab.id);
        quote_in! { registration =>
            $(registry()).register($(built_in_registries()).CREATIVE_MODE_TAB, $(mod_creative_tabs(state)).$(&tab_name)_KEY, $(mod_creative_tabs(state)).$(tab_name));$['\r']
        }
    }

    quote! {
        public class ModCreativeTabs {
            $("// Creative Tab Definitions")
            $(for def in &tab_defs => $def)

            public static void initialize() {
                $registration
            }
        }
    }
}

pub(crate) fn build_main_mod_class(state: &ModState) -> Tokens {
    quote! {
        public class $(&state.mod_name) implements $(mod_initializer()) {
            public static final String MOD_ID = $(quoted(&state.mod_id));

            public static final $(logger()) LOGGER = $(logger_factory()).getLogger(MOD_ID);

            @Override
            public void onInitialize() {
                LOGGER.info("Hello Fabric world!");

                $("// Initialize Mod")
                $(if !&state.items.is_empty() => $(mod_items(state)).initialize();)
                $(if !&state.blocks.is_empty() => $(mod_blocks(state)).initialize();)
                $(if !state.creative_tabs.is_empty() => $(mod_creative_tabs(state)).initialize();)
                $(if !state.mobs.is_empty() => $(format!("ModMobs")).initialize();)
                $(if !state.biomes.is_empty() => $(format!("ModBiomes")).initialize();)
                $(if !state.dimensions.is_empty() => $(format!("ModDimensions")).initialize();)
                $(if !state.structures.is_empty() => $(format!("ModStructures")).initialize();)
                $(if !state.features.is_empty() => $(format!("ModFeatures")).initialize();)
                $(if !state.loot_tables.is_empty() => $(format!("ModLootTables")).initialize();)
                $(if !state.advancements.is_empty() => $(format!("ModAdvancements")).initialize();)
                $(if !state.sound_events.is_empty() => $(format!("ModSoundEvents")).initialize();)
                $(if !state.tags.is_empty() => $(format!("ModTags")).initialize();)

                $("// Fuel and compostable events")
                $(for i in &state.items =>
                    $(if i.kind == ItemKind::Fuel =>
                        $(if let Some(burn_time) = i.burn_time =>
                            $(fuel_value_events()).BUILD.register((builder, context) -> { builder.add($(mod_items(state)).$(to_upper(&i.id)), $(burn_time)); });$['\r']
                        )
                    )
                )
                $(for i in &state.items =>
                    $(if i.kind == ItemKind::Compostable =>
                        $(if let Some(chance) = i.compost_chance =>
                            $(compostable_registry()).INSTANCE.add($(mod_items(state)).$(to_upper(&i.id)), $(format!("{}f", chance)));$['\r']
                        )
                    )
                )
            }

            public static $(identifier()) id(String path) {
                return $(identifier()).fromNamespaceAndPath(MOD_ID, path);
            }
        }
    }
}

pub(crate) fn build_lang_provider(state: &ModState) -> Tokens {
    quote! {
        public class LangProvider extends $(fabric_language_provider()) {
            protected LangProvider($(fabric_pack_output()) dataOutput, $(completable_future())<$(holder_lookup()).Provider> registryLookup) {
                super(dataOutput, "en_us", registryLookup);
            }

            @Override
            public void generateTranslations($(holder_lookup()).Provider holderLookup, TranslationBuilder translationBuilder) {
                $(for i in &state.items =>
                    translationBuilder.add($(quoted(&format!("item.{}.{}", state.mod_id, i.id))), $(quoted(&display_name(&i.id))));$['\r']
                )
                $(for b in &state.blocks =>
                    translationBuilder.add($(quoted(&format!("block.{}.{}", state.mod_id, b.id))), $(quoted(&display_name(&b.id))));$['\r']
                )
                $(for i in &state.items =>
                    $(if !i.tooltip.is_empty() =>
                        $(for idx in 0..i.tooltip.len() =>
                            translationBuilder.add($(quoted(&format!("itemTooltip.{}.{}.{}", state.mod_id, i.id, idx))), $(quoted(&i.tooltip[idx])));$['\r']
                        )
                    )
                )
                $(for i in &state.items =>
                    $(if i.kind == ItemKind::Potion =>
                        $(for effect in &i.effects =>
                            translationBuilder.add($(quoted(&format!("potion.effect.{}.{}.{}", state.mod_id, i.id, effect.effect_type.split(':').next_back().unwrap_or(&effect.effect_type)))), $(quoted(&effect.effect_type.split(':').next_back().unwrap_or(&effect.effect_type).to_string())));$['\r']
                        )
                    )
                )
                $(for tab in &state.creative_tabs =>
                    translationBuilder.add($(quoted(&format!("creativeTab.{}.{}", state.mod_id, tab.id))), $(quoted(&display_name(&tab.id))));$['\r']
                )
            }
        }
    }
}

pub(crate) fn build_datagen_entrypoint(state: &ModState) -> Tokens {
    quote! {
        public class $(format!("{}DataGenerator", state.mod_name)) implements $(data_generator_entrypoint()) {
            @Override
            public void onInitializeDataGenerator($(fabric_data_generator()) fabricDataGenerator) {
                $(fabric_data_generator()).Pack pack = fabricDataGenerator.createPack();

                $(if !(state.items.is_empty() && state.blocks.is_empty()) =>
                    pack.addProvider(LangProvider::new);
                    pack.addProvider(ModelProvider::new);
                )
                $(if !&state.recipes.is_empty() =>
                    pack.addProvider($(format!("{}RecipeProvider", state.mod_name))::new);
                )
            }
        }
    }
}

pub(crate) fn build_model_provider(_state: &ModState) -> Tokens {
    quote! {
        public class ModelProvider extends $(fabric_model_provider()) {
            protected ModelProvider($(fabric_pack_output()) output) {
                super(output);
            }

            @Override
            public void generateBlockStateModels($(block_model_generators()) blockStateModelGenerator) {
            }

            @Override
            public void generateItemModels($(item_model_generators()) itemModelGenerator) {
            }

            @Override
            public String getName() {
                return "ModelProvider";
            }
        }
    }
}

pub(crate) fn build_recipe_provider(state: &ModState) -> Tokens {
    quote! {
        public class $(format!("{}RecipeProvider", state.mod_name)) extends $(fabric_recipe_provider()) {
            public $(format!("{}RecipeProvider", state.mod_name))($(fabric_pack_output()) output, $(completable_future())<$(holder_lookup()).Provider> registriesFuture) {
                super(output, registriesFuture);
            }

            @Override
            protected $(recipe_provider()) createRecipeProvider($(holder_lookup()).Provider registryLookup, $(recipe_output()) exporter) {
                return new $(recipe_provider())(registryLookup, exporter) {
                    @Override
                    public void buildRecipes() {
                        $(for r in &state.recipes =>
                            $(recipe_call(r, state))$['\r']
                        )
                    }
                };
            }

            @Override
            public String getName() {
                return $(quoted(&format!("{}RecipeProvider", state.mod_name)));
            }
        }
    }
}

fn recipe_call(recipe: &Recipe, state: &ModState) -> Tokens {
    let result_ref = result_item(recipe, state);
    let result_ref2 = result_ref.clone();
    let result_ref3 = result_ref.clone();
    let count = recipe.count;
    let recipe_id = strip_namespace(&recipe.id);
    let full_id = format!("{}:{}", state.mod_id, recipe_id);
    match recipe.kind.as_str() {
        "crafting_shaped" => {
            let pattern_lines = &recipe.pattern;
            let defines = &recipe.ingredients;
            quote! {
                shaped($(recipe_category()).MISC, $(result_ref), $(count))
                    $(for p in pattern_lines => .pattern($(quoted(p))))
                    $(for (k, v) in defines => .define($(format!("'{}'", k.chars().next().unwrap_or('?'))), $(ingredient_ref(v, state))))
                    .unlockedBy(getHasName($(result_ref2)), has($(result_ref3)))
                    .save(exporter, $(quoted(&full_id)));
            }
        }
        "crafting_shapeless" => {
            let ingredients = &recipe.ingredients;
            quote! {
                shapeless($(recipe_category()).MISC, $(result_ref), $(count))
                    $(for (_, v) in ingredients => .requires($(ingredient_ref(v, state))))
                    .unlockedBy(getHasName($(result_ref2)), has($(result_ref3)))
                    .save(exporter, $(quoted(&full_id)));
            }
        }
        "smelting" | "blasting" | "smoking" | "campfire_cooking" => {
            let input = recipe
                .ingredients
                .values()
                .next()
                .map(|v| ingredient_ref(v, state))
                .unwrap_or_else(|| quote!("null"));
            let cooking_time = recipe.cooking_time.unwrap_or(200);
            let experience = format!("{}f", recipe.experience.unwrap_or(1.0));
            let category = recipe.category.as_deref().unwrap_or("MISC");
            let book_category = if recipe.kind == "campfire_cooking" {
                quote!($(cooking_book_category()).FOOD)
            } else {
                quote!($(cooking_book_category()).$(category.to_uppercase()))
            };
            match recipe.kind.as_str() {
                "smelting" => quote! {
                    $(simple_cooking_recipe_builder()).smelting($(input), $(recipe_category()).$(category.to_uppercase()), $(book_category), $(result_ref), $(experience), $(cooking_time))
                        .unlockedBy(getHasName($(result_ref2)), has($(result_ref3)))
                        .save(exporter, $(quoted(&full_id)));
                },
                "blasting" => quote! {
                    $(simple_cooking_recipe_builder()).blasting($(input), $(recipe_category()).$(category.to_uppercase()), $(book_category), $(result_ref), $(experience), $(cooking_time))
                        .unlockedBy(getHasName($(result_ref2)), has($(result_ref3)))
                        .save(exporter, $(quoted(&full_id)));
                },
                "smoking" => quote! {
                    $(simple_cooking_recipe_builder()).smoking($(input), $(recipe_category()).$(category.to_uppercase()), $(book_category), $(result_ref), $(experience), $(cooking_time))
                        .unlockedBy(getHasName($(result_ref2)), has($(result_ref3)))
                        .save(exporter, $(quoted(&full_id)));
                },
                "campfire_cooking" => quote! {
                    $(simple_cooking_recipe_builder()).campfireCooking($(input), $(recipe_category()).$(category.to_uppercase()), $(book_category), $(result_ref), $(experience), $(cooking_time))
                        .unlockedBy(getHasName($(result_ref2)), has($(result_ref3)))
                        .save(exporter, $(quoted(&full_id)));
                },
                _ => unreachable!(),
            }
        }
        "stonecutting" => {
            let input = recipe
                .ingredients
                .values()
                .next()
                .map(|v| ingredient_ref(v, state))
                .unwrap_or_else(|| quote!("null"));
            let category = recipe.category.as_deref().unwrap_or("BUILDING_BLOCKS");
            quote! {
                stonecutterResultFromBase($(recipe_category()).$(category.to_uppercase()), $(result_ref), $(input), $(count))
                    .unlockedBy(getHasName($(result_ref2)), has($(result_ref3)))
                    .save(exporter, $(quoted(&full_id)));
            }
        }
        "smithing" => {
            let base = recipe
                .ingredients
                .values()
                .next()
                .map(|v| ingredient_ref(v, state))
                .unwrap_or_else(|| quote!("null"));
            let template = recipe
                .ingredients
                .values()
                .nth(1)
                .map(|v| ingredient_ref(v, state))
                .unwrap_or_else(|| quote!("null"));
            let category = recipe.category.as_deref().unwrap_or("MISC");
            quote! {
                $(smithing_transform_recipe_builder()).smithing(
                    $(template),
                    $(base),
                    $(item_tags()).NETHERITE_TOOL_MATERIALS,
                    $(recipe_category()).$(category.to_uppercase()),
                    $(result_ref)
                ).unlocks("has_netherite_ingot", has($(item_tags()).NETHERITE_TOOL_MATERIALS))
                 .save(exporter, $(quoted(&full_id)));
            }
        }
        _ => quote! { /* unsupported recipe type: $(quoted(&recipe.kind)) */ },
    }
}

fn result_item(recipe: &Recipe, state: &ModState) -> Tokens {
    let id = &recipe.result;
    if let Some(vanilla) = id.strip_prefix("minecraft:") {
        let mc_constant = vanilla.to_uppercase().replace('-', "_");
        quote! { $(items()).$(mc_constant) }
    } else {
        let item_id = strip_namespace(id);
        quote! { $(mod_items(state)).$(to_upper(item_id)) }
    }
}

fn ingredient_ref(value: &str, state: &ModState) -> Tokens {
    if let Some(vanilla) = value.strip_prefix("minecraft:") {
        let mc_constant = vanilla.to_uppercase().replace('-', "_");
        quote! { $(ingredient()).of($(items()).$(mc_constant)) }
    } else {
        let item_id = strip_namespace(value);
        let const_name = to_upper(item_id);
        if state.items.iter().any(|i| i.id == item_id) {
            quote! { $(ingredient()).of($(mod_items(state)).$(const_name)) }
        } else if state.blocks.iter().any(|b| b.id == item_id) {
            quote! { $(ingredient()).of($(mod_blocks(state)).$(const_name)) }
        } else {
            let id_ref: Tokens = quote! {
                $(identifier()).fromNamespaceAndPath(
                    $(mod_class(state)).MOD_ID,
                    $(quoted(item_id))
                )
            };
            quote! { $(ingredient()).of($id_ref) }
        }
    }
}

/// Strips the `namespace:` prefix from an item identifier, returning the bare id.
fn strip_namespace(id: &str) -> &str {
    id.split_once(':').map_or(id, |(_, path)| path)
}

fn item_properties(item: &Item, use_custom_class: bool) -> Tokens {
    let mut out: Tokens = quote! { new Item.Properties() };

    if item.kind == ItemKind::Potion && !item.effects.is_empty() {
        import("java.util", "Optional").register(&mut out);
        import("java.util", "List").register(&mut out);
        import("net.minecraft.world.item.alchemy", "PotionContents").register(&mut out);
        import("net.minecraft.world.effect", "MobEffectInstance").register(&mut out);
        import("net.minecraft.world.effect", "MobEffects").register(&mut out);
        data_components().register(&mut out);
        for effect in &item.effects {
            let effect_const = effect
                .effect_type
                .split(':')
                .next_back()
                .unwrap_or(&effect.effect_type)
                .to_uppercase()
                .replace('-', "_");
            let dur = effect.duration;
            let amp = effect.amplifier;
            quote_in! { out => .component($(data_components()).POTION_CONTENTS, new $(potion_contents())(Optional.empty(), Optional.empty(), $(list()).of(new $(mob_effect_instance())($(mob_effects()).$(effect_const), $(dur), $(amp))), Optional.empty())) };
        }
        quote_in! { out => .component($(data_components()).CONSUMABLE, $(consumables()).DEFAULT_DRINK) };
    }

    if !use_custom_class {
        match item.kind {
            ItemKind::Tool => {
                if let Some(mat) = &item.material {
                    let material_const = mat.to_uppercase().replace('-', "_");
                    let damage = format!("{}f", item.attack_damage.unwrap_or(1.0));
                    let speed = format!("{}f", item.attack_speed.unwrap_or(1.6));
                    quote_in! { out => .sword($(tool_materials()).$(material_const), $(damage), $(speed)) }
                }
            }
            ItemKind::Axe | ItemKind::Shovel | ItemKind::Hoe => {
                if let Some(mat) = &item.material {
                    let material_const = mat.to_uppercase().replace('-', "_");
                    let speed = format!("{}f", item.attack_speed.unwrap_or(1.6));
                    let damage = format!("{}f", item.attack_damage.unwrap_or(1.0));
                    match item.kind {
                        ItemKind::Axe => {
                            quote_in! { out => .axe($(tool_materials()).$(material_const), $(damage), $(speed)) }
                        }
                        ItemKind::Shovel => {
                            quote_in! { out => .shovel($(tool_materials()).$(material_const), $(damage), $(speed)) }
                        }
                        ItemKind::Hoe => {
                            quote_in! { out => .hoe($(tool_materials()).$(material_const), $(damage), $(speed)) }
                        }
                        _ => unreachable!(),
                    }
                }
            }
            ItemKind::SpawnEgg => {
                if let Some(entity_type_str) = &item.entity_type {
                    let vanilla_prefix = "minecraft:";
                    let entity_const =
                        if let Some(stripped) = entity_type_str.strip_prefix(vanilla_prefix) {
                            stripped.to_uppercase().replace('-', "_")
                        } else {
                            entity_type_str.to_uppercase().replace('-', "_")
                        };
                    quote_in! { out => .spawnEgg($(entity_types()).$(entity_const)) };
                }
            }
            ItemKind::Fuel | ItemKind::Compostable => {}
            ItemKind::Basic => {}
            ItemKind::Armor => {
                if let Some(mat) = &item.armor_material {
                    let material_const = mat.to_uppercase().replace('-', "_");
                    let slot = match item.armor_slot.as_deref() {
                        Some("helmet") => "HELMET",
                        Some("chestplate") => "CHESTPLATE",
                        Some("leggings") => "LEGGINGS",
                        Some("boots") => "BOOTS",
                        _ => "HELMET",
                    };
                    quote_in! { out => .humanoidArmor($(armor_materials()).$(material_const), $(armor_type()).$(slot)) };
                }
            }
            ItemKind::Shield => {
                import("java.util", "List").register(&mut out);
                import("java.util", "Optional").register(&mut out);
                import("net.minecraft.world.item.component", "BlocksAttacks").register(&mut out);
                data_components().register(&mut out);
                quote_in! { out => .component($(data_components()).BLOCKS_ATTACKS, new BlocksAttacks(0.25F, 1.0F, List.of(), new BlocksAttacks.ItemDamageFunction(3.0F, 1.0F, 1.0F), Optional.empty(), Optional.empty(), Optional.empty())) };
            }
            _ => {}
        }
    }

    if item.kind == ItemKind::Food && (item.nutrition.is_some() || item.saturation.is_some()) {
        food_properties().register(&mut out);
        let mut builder = quote! { new FoodProperties.Builder() };
        if let Some(nutrition) = item.nutrition {
            quote_in! { builder => .nutrition($(nutrition)) };
        }
        if let Some(saturation) = item.saturation {
            let sat_str = format!("{}f", saturation);
            quote_in! { builder => .saturationModifier($(sat_str)) };
        }
        if item.always_edible {
            quote_in! { builder => .alwaysEdible() };
        }
        let food_props = builder;
        if item.effects.is_empty() {
            quote_in! { out => .food($(food_props).build()) };
        } else {
            import("net.minecraft.world.item.component", "Consumable").register(&mut out);
            import(
                "net.minecraft.world.item.consume_effects",
                "ApplyStatusEffectsConsumeEffect",
            )
            .register(&mut out);
            import("net.minecraft.world.effect", "MobEffectInstance").register(&mut out);
            import("net.minecraft.world.effect", "MobEffects").register(&mut out);
            import("java.util", "List").register(&mut out);
            import("net.minecraft.world.item", "ItemUseAnimation").register(&mut out);
            import("net.minecraft.sounds", "SoundEvents").register(&mut out);
            let mut consumable_builder = quote! { Consumable.builder() };
            quote_in! { consumable_builder => .consumeSeconds(1.6F) };
            quote_in! { consumable_builder => .animation(ItemUseAnimation.EAT) };
            quote_in! { consumable_builder => .sound(SoundEvents.GENERIC_EAT) };
            quote_in! { consumable_builder => .hasConsumeParticles(true) };
            if item.effects.len() == 1 {
                let effect = &item.effects[0];
                let effect_const = effect
                    .effect_type
                    .split(':')
                    .next_back()
                    .unwrap_or(&effect.effect_type)
                    .to_uppercase()
                    .replace('-', "_");
                quote_in! { consumable_builder => .onConsume(new ApplyStatusEffectsConsumeEffect(new $(mob_effect_instance())($(mob_effects()).$(effect_const), $(effect.duration), $(effect.amplifier)))) };
            } else {
                let mut effects_list = quote! { List.of() };
                for effect in &item.effects {
                    let effect_const = effect
                        .effect_type
                        .split(':')
                        .next_back()
                        .unwrap_or(&effect.effect_type)
                        .to_uppercase()
                        .replace('-', "_");
                    quote_in! { effects_list => $(mob_effect_instance())($(mob_effects()).$(effect_const), $(effect.duration), $(effect.amplifier)) };
                }
                quote_in! { consumable_builder => .onConsume(new ApplyStatusEffectsConsumeEffect($(effects_list))) };
            }
            quote_in! { consumable_builder => .build() };
            quote_in! { out => .food($(food_props).build(), $(consumable_builder)) };
        }
    }

    if let Some(dur) = item.durability {
        let snippet = format!("durability({})", dur);
        quote_in! { out => .$(snippet) }
    }

    out
}

fn item_factory(item_def: &Item, use_custom_class: bool) -> Tokens {
    if use_custom_class {
        let class_name = format!("{}Item", to_upper(&item_def.id));
        match item_def.kind {
            ItemKind::Axe | ItemKind::Shovel | ItemKind::Hoe => {
                if let Some(mat) = &item_def.material {
                    let material_const = mat.to_uppercase().replace('-', "_");
                    let speed = format!("{}f", item_def.attack_speed.unwrap_or(1.6));
                    let damage = format!("{}f", item_def.attack_damage.unwrap_or(1.0));
                    quote! { settings -> new $(class_name)($(tool_materials()).$(material_const), $(damage), $(speed), settings) }
                } else {
                    quote! { $(item())::new }
                }
            }
            ItemKind::Armor => {
                quote! { $(class_name)::new }
            }
            _ => quote! { $(class_name)::new },
        }
    } else {
        match item_def.kind {
            ItemKind::Axe | ItemKind::Shovel | ItemKind::Hoe => {
                if let Some(mat) = &item_def.material {
                    let material_const = mat.to_uppercase().replace('-', "_");
                    let speed = format!("{}f", item_def.attack_speed.unwrap_or(1.6));
                    let damage = format!("{}f", item_def.attack_damage.unwrap_or(1.0));
                    match item_def.kind {
                        ItemKind::Axe => {
                            quote! { settings -> new $(axe_item())($(tool_materials()).$(material_const), $(damage), $(speed), settings) }
                        }
                        ItemKind::Shovel => {
                            quote! { settings -> new $(shovel_item())($(tool_materials()).$(material_const), $(damage), $(speed), settings) }
                        }
                        ItemKind::Hoe => {
                            quote! { settings -> new $(hoe_item())($(tool_materials()).$(material_const), $(damage), $(speed), settings) }
                        }
                        _ => unreachable!(),
                    }
                } else {
                    quote! { $(item())::new }
                }
            }
            ItemKind::Armor => {
                quote! { $(item())::new }
            }
            ItemKind::Shield => {
                quote! { $(shield_item())::new }
            }
            ItemKind::Potion => {
                quote! { $(item())::new }
            }
            ItemKind::SpawnEgg => quote! { $(spawn_egg_item())::new },
            ItemKind::Fuel | ItemKind::Compostable => quote! { $(item())::new },
            _ => quote! { $(item())::new },
        }
    }
}

pub(crate) fn build_item_class(item_def: &Item, state: &ModState) -> Tokens {
    let class_name = format!("{}Item", to_upper(&item_def.id));
    let class_name = class_name.as_str();
    let mod_id = &state.mod_id;
    let item_id = &item_def.id;
    let tooltip_lines = &item_def.tooltip;
    let mut body = quote! {};

    for (idx, _line) in tooltip_lines.iter().enumerate() {
        let key = format!("itemTooltip.{}.{}.{}", mod_id, item_id, idx);
        quote_in! { body =>
            textConsumer.accept($(component()).translatable($(quoted(&key))).withStyle($(chat_formatting()).GOLD));$['\r']
        }
    }

    match item_def.kind {
        ItemKind::Axe | ItemKind::Shovel | ItemKind::Hoe => {
            let base_class = match item_def.kind {
                ItemKind::Axe => axe_item(),
                ItemKind::Shovel => shovel_item(),
                ItemKind::Hoe => hoe_item(),
                _ => unreachable!(),
            };
            quote! {
                public class $(class_name) extends $(base_class) {
                    public $(class_name)($(tool_materials()) material, float attackDamage, float attackSpeed, $(item()).Properties properties) {
                        super(material, attackDamage, attackSpeed, properties);
                    }

                    @Override
                    public void appendHoverText(
                        $(item_stack()) stack,
                        $(item()).TooltipContext context,
                        $(tooltip_display()) display,
                        $(consumer())<$(component())> textConsumer,
                        $(tooltip_flag()) type
                    ) {
                        $body
                    }
                }
            }
        }
        ItemKind::Armor => {
            quote! {
                public class $(class_name) extends $(item()) {
                    public $(class_name)($(item()).Properties properties) {
                        super(properties);
                    }

                    @Override
                    public void appendHoverText(
                        $(item_stack()) stack,
                        $(item()).TooltipContext context,
                        $(tooltip_display()) display,
                        $(consumer())<$(component())> textConsumer,
                        $(tooltip_flag()) type
                    ) {
                        $body
                    }
                }
            }
        }
        ItemKind::Shield => {
            quote! {
                public class $(class_name) extends $(shield_item()) {
                    public $(class_name)($(item()).Properties properties) {
                        super(properties);
                    }

                    @Override
                    public void appendHoverText(
                        $(item_stack()) stack,
                        $(item()).TooltipContext context,
                        $(tooltip_display()) display,
                        $(consumer())<$(component())> textConsumer,
                        $(tooltip_flag()) type
                    ) {
                        $body
                    }
                }
            }
        }
        _ => {
            quote! {
                public class $(class_name) extends $(item()) {
                    public $(class_name)($(item()).Properties properties) {
                        super(properties);
                    }

                    @Override
                    public void appendHoverText(
                        $(item_stack()) stack,
                        $(item()).TooltipContext context,
                        $(tooltip_display()) display,
                        $(consumer())<$(component())> textConsumer,
                        $(tooltip_flag()) type
                    ) {
                        $body
                    }
                }
            }
        }
    }
}

pub(crate) fn to_upper(id: &str) -> String {
    id.to_uppercase().replace('-', "_")
}

fn vanilla_entities<'a, T, F, F2>(
    entities: &'a [T],
    _tabs: &[CreativeTab],
    _id_fn: F,
    is_vanilla: F2,
) -> Vec<&'a T>
where
    F: Fn(&T) -> &str,
    F2: Fn(&T) -> bool,
{
    if _tabs.is_empty() {
        vec![]
    } else {
        entities.iter().filter(|e| is_vanilla(e)).collect()
    }
}

fn display_name(id: &str) -> String {
    id.replace('_', " ").to_title_case()
}

pub(crate) fn build_mod_mobs(state: &ModState) -> Tokens {
    if state.mobs.is_empty() {
        return quote! {
            public class ModMobs {
                public static void initialize() {
                }
            }
        };
    }

    let mob_entities = quote! {
        $(for mob in &state.mobs =>
            public static final EntityType<$(format!("{}Entity", to_pascal_case(&mob.id)))> $(to_upper(&mob.id)) = register(
                ModEntityTypeIds.$(to_upper(&mob.id)),
                $(entity_types()).Builder.<$(format!("{}Entity", to_pascal_case(&mob.id)))>of($(format!("{}Entity::new", to_pascal_case(&mob.id))), $(mob_category()).$(mob.spawn_category.as_deref().unwrap_or("MISC").to_uppercase()))
            );
        )
    };

    let register_method = quote! {
        private static <T extends $(entity())> $(entity_types())<T> register($(resource_key())<$(entity_types())<?>> key, $(entity_types()).Builder<T> builder) {
            return $(registry()).register($(built_in_registries()).ENTITY_TYPE, key, builder.build(key));
        }
    };

    let spawn_eggs = quote! {
        $(for mob in &state.mobs =>
            $(if let Some(egg_id) = &mob.spawn_egg_id =>
                public static final Item $(to_upper(egg_id)) = new $(spawn_egg_item())($(to_upper(&mob.id)), new Item.Properties());
            )
        )
    };

    quote! {
        public class ModMobs {
            $(mob_entities)

            $(spawn_eggs)

            public static void initialize() {
            }

            $(register_method)
        }
    }
}

pub(crate) fn build_mod_biomes(_state: &ModState) -> Tokens {
    quote! {
        public class ModBiomes {
            public static void initialize() {
            }
        }
    }
}

pub(crate) fn build_mod_dimensions(_state: &ModState) -> Tokens {
    quote! {
        public class ModDimensions {
            public static void initialize() {
            }
        }
    }
}

pub(crate) fn build_mod_structures(_state: &ModState) -> Tokens {
    quote! {
        public class ModStructures {
            public static void initialize() {
            }
        }
    }
}

pub(crate) fn build_mod_features(_state: &ModState) -> Tokens {
    quote! {
        public class ModFeatures {
            public static void initialize() {
            }
        }
    }
}

pub(crate) fn build_mod_loot_tables(_state: &ModState) -> Tokens {
    quote! {
        public class ModLootTables {
            public static void initialize() {
            }
        }
    }
}

pub(crate) fn build_mod_advancements(_state: &ModState) -> Tokens {
    quote! {
        public class ModAdvancements {
            public static void initialize() {
            }
        }
    }
}

pub(crate) fn build_mod_sound_events(_state: &ModState) -> Tokens {
    quote! {
        public class ModSoundEvents {
            public static void initialize() {
            }
        }
    }
}

pub(crate) fn build_mod_tags(_state: &ModState) -> Tokens {
    quote! {
        public class ModTags {
            public static void initialize() {
            }
        }
    }
}

fn to_pascal_case(s: &str) -> String {
    s.split('_')
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                None => String::new(),
            }
        })
        .collect()
}
