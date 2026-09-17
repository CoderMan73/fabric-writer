use crate::imports::*;
use crate::state::{Block, Item, ItemKind, ModState, Recipe};
use genco::lang::java::Tokens;
use genco::prelude::*;
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
    let vanilla_items: Vec<&Item> = if !state.creative_tabs.is_empty() {
        state
            .items
            .iter()
            .filter(|i| i.creative_tab.as_deref() == Some("ingredients"))
            .collect()
    } else {
        Vec::new()
    };

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
    let vanilla_blocks: Vec<&Block> = if !state.creative_tabs.is_empty() {
        state
            .blocks
            .iter()
            .filter(|b| b.creative_tab.as_deref() == Some("ingredients"))
            .collect()
    } else {
        Vec::new()
    };

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
            $(for b in &state.blocks =>
                public static final $(block()) $(to_upper(&b.id)) = register(
                    ModBlockItemIds.$(to_upper(&b.id)),
                    $(block())::new,
                    $(block_behaviour()).Properties.ofFullCopy($(blocks()).DIRT)
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
                $(if state.items.iter().any(|i| i.kind == ItemKind::Fuel) =>
                    pack.addProvider($(format!("{}FuelProvider", state.mod_name))::new);
                )
                $(if state.items.iter().any(|i| i.kind == ItemKind::Compostable) =>
                    pack.addProvider($(format!("{}CompostableProvider", state.mod_name))::new);
                )
                $(if state.items.iter().any(|i| i.kind == ItemKind::Armor) =>
                    pack.addProvider($(format!("{}ArmorProvider", state.mod_name))::new);
                )
                $(if state.items.iter().any(|i| i.kind == ItemKind::Shield) =>
                    pack.addProvider($(format!("{}ShieldProvider", state.mod_name))::new);
                )
            }
        }
    }
}

pub(crate) fn build_model_provider(state: &ModState) -> Tokens {
    quote! {
        public class ModelProvider extends $(fabric_model_provider()) {
            protected ModelProvider($(fabric_pack_output()) output) {
                super(output);
            }

            @Override
            public void generateBlockStateModels($(block_model_generators()) blockStateModelGenerator) {
                $(for b in &state.blocks =>
                    blockStateModelGenerator.createTrivialCube($(&mod_blocks(state)).$(to_upper(&b.id)));$['\r']
                )
            }

            @Override
            public void generateItemModels($(item_model_generators()) itemModelGenerator) {
                $(for i in &state.items =>
                    $(if i.kind == ItemKind::Armor =>
                        itemModelGenerator.generateArmorModel($(&mod_items(state)).$(to_upper(&i.id)));$['\r']
                    )
                    $(if i.kind == ItemKind::Shield =>
                        itemModelGenerator.generateShieldModel($(&mod_items(state)).$(to_upper(&i.id)));$['\r']
                    )
                    $(if !(i.kind == ItemKind::Armor || i.kind == ItemKind::Shield) =>
                        itemModelGenerator.generateFlatItem($(&mod_items(state)).$(to_upper(&i.id)), $(model_templates()).FLAT_ITEM);$['\r']
                    )
                )
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

pub(crate) fn build_fuel_provider(state: &ModState) -> Tokens {
    let has_fuel = state
        .items
        .iter()
        .any(|i| i.kind == ItemKind::Fuel && i.burn_time.is_some());
    if !has_fuel {
        return quote! {};
    }
    quote! {
        public class $(format!("{}FuelProvider", state.mod_name)) extends $(fuel_provider()) {
            public $(format!("{}FuelProvider", state.mod_name))($(fabric_pack_output()) output) {
                super(output);
            }

            @Override
            public void generate() {
                $(for i in &state.items =>
                    $(if i.kind == ItemKind::Fuel =>
                        $(if let Some(burn_time) = i.burn_time =>
                            $(fuel_registry()).register($(mod_items(state)).$(to_upper(&i.id)), $(burn_time));$['\r']
                        )
                    )
                )
            }
        }
    }
}

pub(crate) fn build_compostable_provider(state: &ModState) -> Tokens {
    let has_compostable = state
        .items
        .iter()
        .any(|i| i.kind == ItemKind::Compostable && i.compost_chance.is_some());
    if !has_compostable {
        return quote! {};
    }
    quote! {
        public class $(format!("{}CompostableProvider", state.mod_name)) extends $(compostable_provider()) {
            public $(format!("{}CompostableProvider", state.mod_name))($(fabric_pack_output()) output) {
                super(output);
            }

            @Override
            public void generate() {
                $(for i in &state.items =>
                    $(if i.kind == ItemKind::Compostable =>
                        $(if let Some(chance) = i.compost_chance =>
                            $(compostable_registry()).register($(mod_items(state)).$(to_upper(&i.id)), $(format!("{}f", chance)));$['\r']
                        )
                    )
                )
            }
        }
    }
}

pub(crate) fn build_armor_provider(state: &ModState) -> Tokens {
    let has_armor = state.items.iter().any(|i| i.kind == ItemKind::Armor);
    if !has_armor {
        return quote! {};
    }
    quote! {
        public class $(format!("{}ArmorProvider", state.mod_name)) extends $(fabric_equipment_model_provider()) {
            public $(format!("{}ArmorProvider", state.mod_name))($(fabric_pack_output()) output) {
                super(output);
            }

            @Override
            public void generateEquipmentModels($(equipment_model_generator()) generator) {
                $(for i in &state.items =>
                    $(if i.kind == ItemKind::Armor =>
                        generator.generateArmor($(mod_items(state)).$(to_upper(&i.id)));$['\r']
                    )
                )
            }
        }
    }
}

pub(crate) fn build_shield_provider(state: &ModState) -> Tokens {
    let has_shield = state.items.iter().any(|i| i.kind == ItemKind::Shield);
    if !has_shield {
        return quote! {};
    }
    quote! {
        public class $(format!("{}ShieldProvider", state.mod_name)) extends $(fabric_equipment_model_provider()) {
            public $(format!("{}ShieldProvider", state.mod_name))($(fabric_pack_output()) output) {
                super(output);
            }

            @Override
            public void generateEquipmentModels($(equipment_model_generator()) generator) {
                $(for i in &state.items =>
                    $(if i.kind == ItemKind::Shield =>
                        generator.generateShield($(mod_items(state)).$(to_upper(&i.id)));$['\r']
                    )
                )
            }
        }
    }
}

fn recipe_call(recipe: &Recipe, state: &ModState) -> Tokens {
    let result_ref = result_item(recipe, state);
    let count = recipe.count;
    let recipe_id = strip_namespace(&recipe.id);
    let full_id = format!("{}:{}", state.mod_id, recipe_id);
    match recipe.kind.as_str() {
        "crafting_shaped" => {
            let pattern_lines = &recipe.pattern;
            let defines = &recipe.ingredients;
            let result_ref2 = result_ref.clone();
            let result_ref3 = result_ref.clone();
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
            let result_ref2 = result_ref.clone();
            let result_ref3 = result_ref.clone();
            quote! {
                shapeless($(recipe_category()).MISC, $(result_ref), $(count))
                    $(for (_, v) in ingredients => .requires($(ingredient_ref(v, state))))
                    .unlockedBy(getHasName($(result_ref2)), has($(result_ref3)))
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
            ItemKind::Food => {
                if item.nutrition.is_some() || item.saturation.is_some() {
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
                    quote_in! { out => .food($(builder).build()) };
                }
            }
            ItemKind::SpawnEgg => {
                if let Some(entity_type_str) = &item.entity_type {
                    let vanilla_prefix = "minecraft:";
                    let entity_const = if entity_type_str.starts_with(vanilla_prefix) {
                        entity_type_str[vanilla_prefix.len()..].to_uppercase().replace('-', "_")
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
            ItemKind::Shield => {}
            ItemKind::Potion => {
                if !item.effects.is_empty() {
                    let mut component_args = quote! {};
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
                        quote_in! { component_args =>
                            .component($(data_components()).POTION_CONTENTS, new $(potion_contents())($(list()).of(new $(mob_effect_instance())($(mob_effects()).$(effect_const), $(dur), $(amp), 1.0f))))
                        }
                    }
                    quote_in! { out => $component_args }
                }
            }
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

fn display_name(id: &str) -> String {
    id.replace('_', " ").to_title_case()
}
