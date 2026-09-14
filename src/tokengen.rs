use crate::imports::*;
use crate::state::{Item, ItemKind, ModState, Recipe};
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
                    public static final $(item()) $(to_upper(&i.id)) = register(ModItemIds.$(to_upper(&i.id)), $(to_upper(&i.id))Item::new, $(item_properties(i)));$['\r']
                )
                $(if i.tooltip.is_empty() =>
                    public static final $(item()) $(to_upper(&i.id)) = register(ModItemIds.$(to_upper(&i.id)), $(item())::new, $(item_properties(i)));$['\r']
                )
            )

            public static void initialize() {
                $(if !&state.items.is_empty() =>
                    $(creative_mode_tab_events()).modifyOutputEvent($(creative_mode_tabs()).INGREDIENTS)
                        .register((creativeTab) ->
                        {
                            $(for i in &state.items =>
                                creativeTab.accept($(mod_items(state)).$(to_upper(&i.id)));$['\r']
                            )
                        });
                )
            }
        }
    }
}

pub(crate) fn build_mod_blocks(state: &ModState) -> Tokens {
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
                $(if !&state.blocks.is_empty() =>
                    $(creative_mode_tab_events()).modifyOutputEvent($(creative_mode_tabs()).INGREDIENTS)
                        .register((creativeTab) ->
                        {
                            $(for b in &state.blocks =>
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
                    itemModelGenerator.generateFlatItem($(&mod_items(state)).$(to_upper(&i.id)), $(model_templates()).FLAT_ITEM);$['\r']
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

fn item_properties(item: &Item) -> Tokens {
    let mut out: Tokens = quote! { new Item.Properties() };

    if item.kind == ItemKind::Tool
        && let Some(mat) = &item.material
    {
        let material_const = mat.to_uppercase().replace('-', "_");
        let damage = format!("{}f", item.attack_damage.unwrap_or(1.0));
        let speed = format!("{}f", item.attack_speed.unwrap_or(1.6));
        quote_in! { out => .sword($(tool_materials()).$(material_const), $(damage), $(speed)) }
    }

    if let Some(dur) = item.durability {
        let snippet = format!("durability({})", dur);
        quote_in! { out => .$(snippet) }
    }

    out
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
                $(function())<$(component())> textConsumer,
                $(tooltip_flag()) type
            ) {
                $body
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
