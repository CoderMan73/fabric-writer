use crate::imports::*;
use crate::state::{Item, ItemKind, ModState};
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
                public static final $(item()) $(to_upper(&i.id)) = register(ModItemIds.$(to_upper(&i.id)), $(item())::new, $(item_properties(i)));$['\r']
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
                ModItems.initialize();
                ModBlocks.initialize();
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

                pack.addProvider(LangProvider::new);
                pack.addProvider(ModelProvider::new);
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

fn item_properties(item: &Item) -> Tokens {
    let mut out: Tokens = quote! { new Item.Properties() };

    if item.kind == ItemKind::Tool
        && let Some(mat) = &item.material
    {
        let snippet = format!(
            "sword({}, {}, {})",
            mat,
            item.attack_damage.unwrap_or(1.0),
            item.attack_speed.unwrap_or(1.6)
        );
        quote_in! { out => .$(snippet) }
    }

    if let Some(dur) = item.durability {
        let snippet = format!("durability({})", dur);
        quote_in! { out => .$(snippet) }
    }

    out
}

fn to_upper(id: &str) -> String {
    id.to_uppercase().replace('-', "_")
}

fn display_name(id: &str) -> String {
    id.replace('_', " ").to_title_case()
}
