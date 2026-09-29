use anyhow::{Context, Result};
use genco::fmt::{self, IoWriter};
use genco::lang::java::{self, Java, Tokens};
use std::collections::{HashMap, HashSet};
use std::fs::{create_dir_all, read as fs_read, read_dir, remove_file, write as fs_write};
use std::path::{Path, PathBuf};

use crate::state::{BlockModelKind, Entity, ItemKind, ModState};
use crate::tokengen::{
    BuildFn, build_datagen_entrypoint, build_item_class, build_lang_provider, build_main_mod_class,
    build_mod_advancements, build_mod_biomes, build_mod_block_ids, build_mod_block_item_ids,
    build_mod_blocks, build_mod_creative_tabs, build_mod_dimensions, build_mod_features,
    build_mod_item_ids, build_mod_items, build_mod_loot_tables, build_mod_mobs,
    build_mod_sound_events, build_mod_structures, build_mod_tags, build_model_provider,
    build_recipe_provider, build_worldgen_provider, to_upper,
};

const PLACEHOLDER_ITEM: &[u8] = include_bytes!("../assets/placeholder_item.png");
const PLACEHOLDER_BLOCK: &[u8] = include_bytes!("../assets/placeholder_block.png");

/// Per-file dirty flags for one [`regenerate_all`] pass.
///
/// Each field corresponds to a generated Java file.
#[derive(Debug, Clone, Copy, Default)]
pub struct DirtyFlags {
    /// ModItemIds.java
    pub mod_item_ids: bool,

    /// ModItems.java
    pub mod_items: bool,

    /// Main mod class (`<ModName>.java`).
    pub mod_class: bool,

    /// LangProvider.java
    pub lang_provider: bool,

    /// `<ModName>DataGenerator.java`
    pub datagen_entrypoint: bool,

    /// ModelProvider.java
    pub model_provider: bool,

    /// ModBlocks.java
    pub mod_blocks: bool,

    /// ModBlockIds.java
    pub mod_block_ids: bool,

    /// ModBlockItemIds.java
    pub mod_block_item_ids: bool,

    /// `<ModName>RecipeProvider.java`
    pub recipe_provider: bool,

    /// `ModCreativeTabs.java`
    pub creative_tabs: bool,

    /// `ModMobs.java`
    pub mod_mobs: bool,

    /// `ModBiomes.java`
    pub mod_biomes: bool,

    /// `ModDimensions.java`
    pub mod_dimensions: bool,

    /// `ModStructures.java`
    pub mod_structures: bool,

    /// `ModFeatures.java`
    pub mod_features: bool,

    /// `ModLootTables.java`
    pub mod_loot_tables: bool,

    /// `ModAdvancements.java`
    pub mod_advancements: bool,

    /// `ModSoundEvents.java`
    pub mod_sound_events: bool,

    /// `ModTags.java`
    pub mod_tags: bool,

    /// `<ModName>WorldgenProvider.java`
    pub worldgen_provider: bool,
}

impl DirtyFlags {
    /// Mark every file dirty.
    pub const fn all() -> Self {
        Self {
            mod_item_ids: true,
            mod_items: true,
            mod_class: true,
            lang_provider: true,
            datagen_entrypoint: true,
            model_provider: true,
            mod_blocks: true,
            mod_block_ids: true,
            mod_block_item_ids: true,
            recipe_provider: true,
            creative_tabs: true,
            mod_mobs: true,
            mod_biomes: true,
            mod_dimensions: true,
            mod_structures: true,
            mod_features: true,
            mod_loot_tables: true,
            mod_advancements: true,
            mod_sound_events: true,
            mod_tags: true,
            worldgen_provider: true,
        }
    }

    /// Marks every file that utilizes the entity as dirty.
    pub fn from_entity(entity: &Entity) -> Self {
        match entity {
            Entity::Item(_) => Self {
                mod_item_ids: true,
                mod_items: true,
                mod_class: true,
                lang_provider: true,
                model_provider: true,
                datagen_entrypoint: true,
                ..Default::default()
            },
            Entity::Block(_) => Self {
                mod_blocks: true,
                mod_block_ids: true,
                mod_block_item_ids: true,
                mod_class: true,
                lang_provider: true,
                model_provider: true,
                datagen_entrypoint: true,
                ..Default::default()
            },
            Entity::Recipe(_) => Self {
                recipe_provider: true,
                datagen_entrypoint: true,
                ..Default::default()
            },
            Entity::CreativeTab(_) => Self {
                creative_tabs: true,
                mod_class: true,
                ..Default::default()
            },
            Entity::Mob(_) => Self {
                mod_mobs: true,
                mod_class: true,
                ..Default::default()
            },
            Entity::Biome(_) => Self {
                mod_biomes: true,
                mod_class: true,
                datagen_entrypoint: true,
                worldgen_provider: true,
                ..Default::default()
            },
            Entity::Dimension(_) => Self {
                mod_dimensions: true,
                mod_class: true,
                datagen_entrypoint: true,
                worldgen_provider: true,
                ..Default::default()
            },
            Entity::DimensionType(_) => Self {
                mod_dimensions: true,
                mod_class: true,
                datagen_entrypoint: true,
                worldgen_provider: true,
                ..Default::default()
            },
            Entity::Structure(_) => Self {
                mod_structures: true,
                mod_class: true,
                datagen_entrypoint: true,
                worldgen_provider: true,
                ..Default::default()
            },
            Entity::Feature(_) => Self {
                mod_features: true,
                mod_class: true,
                datagen_entrypoint: true,
                worldgen_provider: true,
                ..Default::default()
            },
            Entity::LootTable(_) => Self {
                mod_loot_tables: true,
                mod_class: true,
                datagen_entrypoint: true,
                ..Default::default()
            },
            Entity::Advancement(_) => Self {
                mod_advancements: true,
                mod_class: true,
                datagen_entrypoint: true,
                ..Default::default()
            },
            Entity::SoundEvent(_) => Self {
                mod_sound_events: true,
                mod_class: true,
                datagen_entrypoint: true,
                ..Default::default()
            },
            Entity::Tag(_) => Self {
                mod_tags: true,
                mod_class: true,
                datagen_entrypoint: true,
                ..Default::default()
            },
        }
    }

    fn is_set(&self, field: &str) -> bool {
        match field {
            "mod_item_ids" => self.mod_item_ids,
            "mod_items" => self.mod_items,
            "mod_class" => self.mod_class,
            "lang_provider" => self.lang_provider,
            "datagen_entrypoint" => self.datagen_entrypoint,
            "model_provider" => self.model_provider,
            "mod_blocks" => self.mod_blocks,
            "mod_block_ids" => self.mod_block_ids,
            "mod_block_item_ids" => self.mod_block_item_ids,
            "recipe_provider" => self.recipe_provider,
            "creative_tabs" => self.creative_tabs,
            "mod_mobs" => self.mod_mobs,
            "mod_biomes" => self.mod_biomes,
            "mod_dimensions" => self.mod_dimensions,
            "mod_structures" => self.mod_structures,
            "mod_features" => self.mod_features,
            "mod_loot_tables" => self.mod_loot_tables,
            "mod_advancements" => self.mod_advancements,
            "mod_sound_events" => self.mod_sound_events,
            "mod_tags" => self.mod_tags,
            "worldgen_provider" => self.worldgen_provider,
            _ => false,
        }
    }
}

/// A single generated file's path, build function, package, and an
/// existence predicate that decides whether the file should exist
/// given the current state. Files whose predicate returns `false` are pruned
/// (deleted if present) rather than regenerated.
struct FileSpec {
    field: &'static str,
    build: BuildFn,
    should_exist: fn(&ModState) -> bool,
}

fn items_exist(state: &ModState) -> bool {
    !state.items.is_empty()
}

fn blocks_exist(state: &ModState) -> bool {
    !state.blocks.is_empty()
}

fn providers_exist(state: &ModState) -> bool {
    items_exist(state) || blocks_exist(state)
}

fn recipes_exist(state: &ModState) -> bool {
    !state.recipes.is_empty()
}

fn worldgen_exists(state: &ModState) -> bool {
    !state.biomes.is_empty()
        || !state.features.is_empty()
        || !state.structures.is_empty()
        || !state.dimensions.is_empty()
        || !state.dimension_types.is_empty()
}

fn file_specs() -> &'static [(&'static str, FileSpec)] {
    &[
        (
            "ModItemIds.java",
            FileSpec {
                field: "mod_item_ids",
                build: build_mod_item_ids,
                should_exist: items_exist,
            },
        ),
        (
            "ModItems.java",
            FileSpec {
                field: "mod_items",
                build: build_mod_items,
                should_exist: items_exist,
            },
        ),
        (
            "<ModName>.java",
            FileSpec {
                field: "mod_class",
                build: build_main_mod_class,
                should_exist: |_| true,
            },
        ),
        (
            "LangProvider.java",
            FileSpec {
                field: "lang_provider",
                build: build_lang_provider,
                should_exist: providers_exist,
            },
        ),
        (
            "<ModName>DataGenerator.java",
            FileSpec {
                field: "datagen_entrypoint",
                build: build_datagen_entrypoint,
                should_exist: providers_exist,
            },
        ),
        (
            "ModelProvider.java",
            FileSpec {
                field: "model_provider",
                build: build_model_provider,
                should_exist: providers_exist,
            },
        ),
        (
            "<ModName>RecipeProvider.java",
            FileSpec {
                field: "recipe_provider",
                build: build_recipe_provider,
                should_exist: recipes_exist,
            },
        ),
        (
            "ModBlocks.java",
            FileSpec {
                field: "mod_blocks",
                build: build_mod_blocks,
                should_exist: blocks_exist,
            },
        ),
        (
            "ModBlockIds.java",
            FileSpec {
                field: "mod_block_ids",
                build: build_mod_block_ids,
                should_exist: blocks_exist,
            },
        ),
        (
            "ModBlockItemIds.java",
            FileSpec {
                field: "mod_block_item_ids",
                build: build_mod_block_item_ids,
                should_exist: blocks_exist,
            },
        ),
        (
            "ModCreativeTabs.java",
            FileSpec {
                field: "creative_tabs",
                build: build_mod_creative_tabs,
                should_exist: |state| !state.creative_tabs.is_empty(),
            },
        ),
        (
            "ModMobs.java",
            FileSpec {
                field: "mod_mobs",
                build: build_mod_mobs,
                should_exist: |state| !state.mobs.is_empty(),
            },
        ),
        (
            "ModBiomes.java",
            FileSpec {
                field: "mod_biomes",
                build: build_mod_biomes,
                should_exist: |state| !state.biomes.is_empty(),
            },
        ),
        (
            "ModDimensions.java",
            FileSpec {
                field: "mod_dimensions",
                build: build_mod_dimensions,
                should_exist: |state| !state.dimensions.is_empty(),
            },
        ),
        (
            "ModStructures.java",
            FileSpec {
                field: "mod_structures",
                build: build_mod_structures,
                should_exist: |state| !state.structures.is_empty(),
            },
        ),
        (
            "ModFeatures.java",
            FileSpec {
                field: "mod_features",
                build: build_mod_features,
                should_exist: |state| !state.features.is_empty(),
            },
        ),
        (
            "ModLootTables.java",
            FileSpec {
                field: "mod_loot_tables",
                build: build_mod_loot_tables,
                should_exist: |state| !state.loot_tables.is_empty(),
            },
        ),
        (
            "ModAdvancements.java",
            FileSpec {
                field: "mod_advancements",
                build: build_mod_advancements,
                should_exist: |state| !state.advancements.is_empty(),
            },
        ),
        (
            "ModSoundEvents.java",
            FileSpec {
                field: "mod_sound_events",
                build: build_mod_sound_events,
                should_exist: |state| !state.sound_events.is_empty(),
            },
        ),
        (
            "ModTags.java",
            FileSpec {
                field: "mod_tags",
                build: build_mod_tags,
                should_exist: |state| !state.tags.is_empty(),
            },
        ),
        (
            "<ModName>WorldgenProvider.java",
            FileSpec {
                field: "worldgen_provider",
                build: build_worldgen_provider,
                should_exist: worldgen_exists,
            },
        ),
    ]
}

/// Regenerates Java sources whose dirty flags are set, pruning or skipping
/// files as appropriate for the current state.
pub fn regenerate_all(state: &ModState, dirty: DirtyFlags, verbose: bool) -> Result<()> {
    let java_root = PathBuf::from("src/main/java").join(state.package_name.replace('.', "/"));
    let client_root = PathBuf::from("src/client/java")
        .join(state.package_name.replace('.', "/"))
        .join("client");
    create_dir_all(&java_root).context("Failed to create Java source directory")?;
    create_dir_all(&client_root).context("Failed to create client Java source directory")?;

    let package = state.package_name.as_str();
    let client_package = format!("{}.client", package);

    let mod_class_name = format!("{}.java", state.mod_name);
    let datagen_class_name = format!("{}DataGenerator.java", state.mod_name);
    let recipe_provider_class_name = format!("{}RecipeProvider.java", state.mod_name);
    let worldgen_provider_class_name = format!("{}WorldgenProvider.java", state.mod_name);

    for (name, spec) in file_specs() {
        let path = resolve_path(
            name,
            &java_root,
            &client_root,
            &mod_class_name,
            &datagen_class_name,
            &recipe_provider_class_name,
            &worldgen_provider_class_name,
        );
        let dirty_bit = dirty.is_set(spec.field);

        if !(spec.should_exist)(state) {
            if path.exists() {
                remove_file(&path).with_context(|| {
                    format!("Failed to prune {} (collection is empty)", path.display())
                })?;
                if verbose {
                    vlog("pruned", &path);
                }
            } else if verbose {
                vlog("skipped (empty)", &path);
            }
            continue;
        }

        if dirty_bit {
            let pkg = if path.starts_with(&client_root) {
                &client_package
            } else {
                package
            };
            write(&path, (spec.build)(state), pkg)?;
            if verbose {
                vlog("wrote", &path);
            }
        } else if verbose {
            vlog("skipped", &path);
        }
    }

    copy_textures(state, verbose)?;
    generate_model_resources(state, verbose)?;
    generate_loot_table_resources(state, verbose)?;
    generate_biome_resources(state, verbose)?;
    generate_configured_feature_resources(state, verbose)?;
    generate_placed_feature_resources(state, verbose)?;
    generate_dimension_type_resources(state, verbose)?;
    generate_dimension_resources(state, verbose)?;
    generate_structure_set_resources(state, verbose)?;
    generate_advancement_resources(state, verbose)?;
    generate_tag_resources(state, verbose)?;
    generate_sound_resources(state, verbose)?;

    for i in &state.items {
        let class_name = format!("{}Item.java", to_upper(&i.id));
        let path = java_root.join(&class_name);
        if i.tooltip.is_empty() {
            if path.exists() {
                remove_file(&path)
                    .with_context(|| format!("Failed to prune {}", path.display()))?;
                if verbose {
                    vlog("pruned", &path);
                }
            }
        } else {
            let pkg = package;
            write(&path, build_item_class(i, state), pkg)?;
            if verbose {
                vlog("wrote", &path);
            }
        }
    }

    prune_orphaned_java(&java_root, &client_root, state, verbose)?;

    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn resolve_path(
    name: &str,
    java_root: &Path,
    client_root: &Path,
    mod_class_name: &str,
    datagen_class_name: &str,
    recipe_provider_class_name: &str,
    worldgen_provider_class_name: &str,
) -> PathBuf {
    match name {
        "ModItemIds.java" => java_root.join(name),
        "ModItems.java" => java_root.join(name),
        "<ModName>.java" => java_root.join(mod_class_name),
        "LangProvider.java" => client_root.join(name),
        "<ModName>DataGenerator.java" => client_root.join(datagen_class_name),
        "ModelProvider.java" => client_root.join(name),
        "<ModName>RecipeProvider.java" => client_root.join(recipe_provider_class_name),
        "<ModName>WorldgenProvider.java" => client_root.join(worldgen_provider_class_name),
        "ModBlocks.java" => java_root.join(name),
        "ModBlockIds.java" => java_root.join(name),
        "ModBlockItemIds.java" => java_root.join(name),
        "ModCreativeTabs.java" => java_root.join(name),
        "ModMobs.java" => java_root.join(name),
        "ModBiomes.java" => java_root.join(name),
        "ModDimensions.java" => java_root.join(name),
        "ModStructures.java" => java_root.join(name),
        "ModFeatures.java" => java_root.join(name),
        "ModLootTables.java" => java_root.join(name),
        "ModAdvancements.java" => java_root.join(name),
        "ModSoundEvents.java" => java_root.join(name),
        "ModTags.java" => java_root.join(name),
        _ => unreachable!("unknown file spec name: {}", name),
    }
}

fn verbose_path(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/").replace("//", "/")
}

fn vlog(action: &str, path: &Path) {
    println!("{action:<8} {}", verbose_path(path));
}

/// Formats and writes Java `Tokens` to disk at `path` with the given package declaration.
///
/// Only writes when the new content differs from what is already on disk.
pub fn write(path: &Path, tokens: Tokens, package: &str) -> Result<()> {
    if let Some(parent) = path.parent() {
        create_dir_all(parent).context("Failed to create Java output directory")?;
    }
    let config = java::Config::default().with_package(package);
    let fmt_config = fmt::Config::from_lang::<Java>();
    let mut buf = Vec::new();
    let mut writer = IoWriter::new(&mut buf);
    tokens.format_file(&mut writer.as_formatter(&fmt_config), &config)?;

    if path.exists()
        && let Ok(existing) = fs_read(path)
        && existing == buf
    {
        return Ok(());
    }

    fs_write(path, buf).context("Failed to write Java file")?;
    Ok(())
}

fn copy_textures_for<'a, I>(
    dir: &Path,
    ids: I,
    placeholder: &[u8],
    kind: &str,
    verbose: bool,
) -> Result<()>
where
    I: IntoIterator<Item = &'a String>,
{
    create_dir_all(dir).with_context(|| format!("Failed to create {} textures directory", kind))?;

    let expected: HashSet<String> = ids.into_iter().map(|id| format!("{}.png", id)).collect();

    if dir.exists() {
        for entry in
            read_dir(dir).with_context(|| format!("Failed to read {} textures directory", kind))?
        {
            let entry = entry.with_context(|| format!("Failed to read {} dir entry", kind))?;
            let name = entry.file_name().to_string_lossy().into_owned();
            if name.ends_with(".png") && !expected.contains(&name) {
                remove_file(entry.path()).with_context(|| {
                    format!("Failed to prune orphaned {} texture '{}'", kind, name)
                })?;
                if verbose {
                    vlog("pruned", &entry.path());
                }
            }
        }
    }

    for id in &expected {
        let dest = dir.join(id);
        if !dest.exists() {
            std::fs::write(&dest, placeholder).with_context(|| {
                format!(
                    "Failed to write placeholder texture for {} '{}'",
                    kind,
                    id.trim_end_matches(".png")
                )
            })?;
            if verbose {
                vlog("wrote", &dest);
            }
        } else if verbose {
            vlog("skipped", &dest);
        }
    }

    Ok(())
}

fn copy_textures(state: &ModState, verbose: bool) -> Result<()> {
    let textures_root = PathBuf::from("src/main/resources/assets")
        .join(&state.mod_id)
        .join("textures");
    copy_textures_for(
        &textures_root.join("item"),
        state.items.iter().map(|i| &i.id),
        PLACEHOLDER_ITEM,
        "item",
        verbose,
    )?;
    copy_textures_for(
        &textures_root.join("block"),
        state.blocks.iter().map(|b| &b.id),
        PLACEHOLDER_BLOCK,
        "block",
        verbose,
    )?;
    Ok(())
}

fn generate_model_resources(state: &ModState, _verbose: bool) -> Result<()> {
    let assets_root = PathBuf::from("src/main/resources/assets").join(&state.mod_id);
    let blockstates_root = assets_root.join("blockstates");
    let block_models_root = assets_root.join("models").join("block");
    let item_models_root = assets_root.join("models").join("item");
    create_dir_all(&blockstates_root)?;
    create_dir_all(&block_models_root)?;
    create_dir_all(&item_models_root)?;

    for block in &state.blocks {
        let id = &block.id;
        let mod_id = &state.mod_id;
        let blockstate_path = blockstates_root.join(format!("{}.json", id));
        let block_model_path = block_models_root.join(format!("{}.json", id));

        match block.model_kind {
            BlockModelKind::CubeAll | BlockModelKind::Cube => {
                std::fs::write(
                    blockstate_path,
                    format!(
                        r#"{{"variants":{{"":{{"model":"{}:block/{}"}}}}}}"#,
                        mod_id, id
                    ),
                )?;
                std::fs::write(
                    block_model_path,
                    format!(
                        r#"{{"parent":"minecraft:block/cube_all","textures":{{"all":"{}:block/{}"}}}}"#,
                        mod_id, id
                    ),
                )?;
            }
            BlockModelKind::CubeColumn => {
                std::fs::write(
                    blockstate_path,
                    format!(
                        r#"{{"variants":{{"":{{"model":"{}:block/{}"}}}}}}"#,
                        mod_id, id
                    ),
                )?;
                std::fs::write(
                    block_model_path,
                    format!(
                        r#"{{"parent":"minecraft:block/cube_column","textures":{{"end":"{}:block/{}_top","side":"{}:block/{}_side"}}}}"#,
                        mod_id, id, mod_id, id
                    ),
                )?;
            }
            BlockModelKind::CubeBottomTop | BlockModelKind::Orientable => {
                let texture_name = id;
                std::fs::write(
                    blockstate_path,
                    format!(
                        r#"{{"variants":{{"":{{"model":"{}:block/{}"}}}}}}"#,
                        mod_id, id
                    ),
                )?;
                std::fs::write(
                    block_model_path,
                    format!(
                        r#"{{"parent":"minecraft:block/cube_bottom_top","textures":{{"bottom":"{}:block/{}_bottom","top":"{}:block/{}_top","side":"{}:block/{}_side"}}}}"#,
                        mod_id, texture_name, mod_id, texture_name, mod_id, texture_name
                    ),
                )?;
            }
            BlockModelKind::Slab => {
                std::fs::write(
                    blockstate_path,
                    format!(
                        r#"{{"variants":{{"type=bottom":{{"model":"{}:block/{}_slab_bottom"}},"type=top":{{"model":"{}:block/{}_slab_top"}},"type=double":{{"model":"{}:block/{}_slab_double"}}}}}}"#,
                        mod_id, id, mod_id, id, mod_id, id
                    ),
                )?;
                std::fs::write(
                    block_models_root.join(format!("{}_slab_bottom.json", id)),
                    format!(
                        r#"{{"parent":"minecraft:block/slab_bottom","textures":{{"bottom":"{}:block/{}","top":"{}:block/{}","side":"{}:block/{}"}}}}"#,
                        mod_id, id, mod_id, id, mod_id, id
                    ),
                )?;
                std::fs::write(
                    block_models_root.join(format!("{}_slab_top.json", id)),
                    format!(
                        r#"{{"parent":"minecraft:block/slab_top","textures":{{"bottom":"{}:block/{}","top":"{}:block/{}","side":"{}:block/{}"}}}}"#,
                        mod_id, id, mod_id, id, mod_id, id
                    ),
                )?;
                std::fs::write(
                    block_models_root.join(format!("{}_slab_double.json", id)),
                    format!(
                        r#"{{"parent":"minecraft:block/cube_all","textures":{{"all":"{}:block/{}"}}}}"#,
                        mod_id, id
                    ),
                )?;
            }
            BlockModelKind::Stairs => {
                std::fs::write(
                    blockstate_path,
                    format!(
                        r#"{{"variants":{{"facing=east,half=bottom,shape=straight":{{"model":"{}:block/{}_stairs"}},"facing=west,half=bottom,shape=straight":{{"model":"{}:block/{}_stairs","y":90,"uvlock":true}},"facing=south,half=bottom,shape=straight":{{"model":"{}:block/{}_stairs","y":180,"uvlock":true}},"facing=north,half=bottom,shape=straight":{{"model":"{}:block/{}_stairs","y":270,"uvlock":true}},"facing=east,half=bottom,shape=outer_right":{{"model":"{}:block/{}_stairs_outer_right"}},"facing=west,half=bottom,shape=outer_right":{{"model":"{}:block/{}_stairs_outer_right","y":90,"uvlock":true}},"facing=south,half=bottom,shape=outer_right":{{"model":"{}:block/{}_stairs_outer_right","y":180,"uvlock":true}},"facing=north,half=bottom,shape=outer_right":{{"model":"{}:block/{}_stairs_outer_right","y":270,"uvlock":true}},"facing=east,half=bottom,shape=outer_left":{{"model":"{}:block/{}_stairs_outer_left"}},"facing=west,half=bottom,shape=outer_left":{{"model":"{}:block/{}_stairs_outer_left","y":90,"uvlock":true}},"facing=south,half=bottom,shape=outer_left":{{"model":"{}:block/{}_stairs_outer_left","y":180,"uvlock":true}},"facing=north,half=bottom,shape=outer_left":{{"model":"{}:block/{}_stairs_outer_left","y":270,"uvlock":true}},"facing=east,half=bottom,shape=inner_right":{{"model":"{}:block/{}_stairs_inner_right"}},"facing=west,half=bottom,shape=inner_right":{{"model":"{}:block/{}_stairs_inner_right","y":90,"uvlock":true}},"facing=south,half=bottom,shape=inner_right":{{"model":"{}:block/{}_stairs_inner_right","y":180,"uvlock":true}},"facing=north,half=bottom,shape=inner_right":{{"model":"{}:block/{}_stairs_inner_right","y":270,"uvlock":true}},"facing=east,half=bottom,shape=inner_left":{{"model":"{}:block/{}_stairs_inner_left"}},"facing=west,half=bottom,shape=inner_left":{{"model":"{}:block/{}_stairs_inner_left","y":90,"uvlock":true}},"facing=south,half=bottom,shape=inner_left":{{"model":"{}:block/{}_stairs_inner_left","y":180,"uvlock":true}},"facing=north,half=bottom,shape=inner_left":{{"model":"{}:block/{}_stairs_inner_left","y":270,"uvlock":true}},"half=top":{{"model":"{}:block/{}_stairs","x":180,"uvlock":true}}}}"#,
                        mod_id,
                        id,
                        mod_id,
                        id,
                        mod_id,
                        id,
                        mod_id,
                        id,
                        mod_id,
                        id,
                        mod_id,
                        id,
                        mod_id,
                        id,
                        mod_id,
                        id,
                        mod_id,
                        id,
                        mod_id,
                        id,
                        mod_id,
                        id,
                        mod_id,
                        id,
                        mod_id,
                        id,
                        mod_id,
                        id,
                        mod_id,
                        id,
                        mod_id,
                        id,
                        mod_id,
                        id,
                        mod_id,
                        id,
                        mod_id,
                        id,
                        mod_id,
                        id,
                        mod_id,
                        id
                    ),
                )?;
                let tex = id;
                for suffix in &[
                    "",
                    "_inner_right",
                    "_inner_left",
                    "_outer_right",
                    "_outer_left",
                ] {
                    let parent = if suffix.is_empty() {
                        "minecraft:block/stairs".to_string()
                    } else {
                        format!("minecraft:block/stairs{}", suffix)
                    };
                    std::fs::write(
                        block_models_root.join(format!("{}{}.json", id, suffix)),
                        format!(
                            r#"{{"parent":"{}","textures":{{"bottom":"{}:block/{}","top":"{}:block/{}","side":"{}:block/{}"}}}}"#,
                            parent, mod_id, tex, mod_id, tex, mod_id, tex
                        ),
                    )?;
                }
            }
            BlockModelKind::Chain => {
                std::fs::write(
                    blockstate_path,
                    format!(
                        r#"{{"variants":{{"axis=x":{{"model":"{}:block/{}","x":90,"y":90}},"axis=y":{{"model":"{}:block/{}"}},"axis=z":{{"model":"{}:block/{}","x":90}}}}}}"#,
                        mod_id, id, mod_id, id, mod_id, id
                    ),
                )?;
                std::fs::write(
                    block_model_path,
                    format!(
                        r#"{{"parent":"minecraft:block/template_chain","textures":{{"texture":"{}:block/{}"}}}}"#,
                        mod_id, id
                    ),
                )?;
            }
            BlockModelKind::Fence => {
                std::fs::write(
                    blockstate_path,
                    format!(
                        r#"{{"variants":{{"":{{"model":"{}:block/{}"}}}}}}"#,
                        mod_id, id
                    ),
                )?;
                std::fs::write(
                    block_model_path,
                    format!(
                        r#"{{"parent":"minecraft:block/fence","textures":{{"texture":"{}:block/{}","post":"{}:block/{}"}}}}"#,
                        mod_id, id, mod_id, id
                    ),
                )?;
            }
            BlockModelKind::Wall => {
                std::fs::write(
                    blockstate_path,
                    format!(
                        r#"{{"variants":{{"":{{"model":"{}:block/{}"}}}}}}"#,
                        mod_id, id
                    ),
                )?;
                std::fs::write(
                    block_model_path,
                    format!(
                        r#"{{"parent":"minecraft:block/wall","textures":{{"texture":"{}:block/{}"}}}}"#,
                        mod_id, id
                    ),
                )?;
            }
            BlockModelKind::Cross | BlockModelKind::TintedCross => {
                std::fs::write(
                    blockstate_path,
                    format!(
                        r#"{{"variants":{{"":{{"model":"{}:block/{}"}}}}}}"#,
                        mod_id, id
                    ),
                )?;
                std::fs::write(
                    block_model_path,
                    format!(
                        r#"{{"parent":"minecraft:block/cross","textures":{{"cross":"{}:block/{}"}}}}"#,
                        mod_id, id
                    ),
                )?;
            }
            BlockModelKind::Crop => {
                std::fs::write(
                    blockstate_path,
                    format!(
                        r#"{{"variants":{{"age=0":{{"model":"{}:block/{}_stage0"}},"age=1":{{"model":"{}:block/{}_stage1"}},"age=2":{{"model":"{}:block/{}_stage2"}},"age=3":{{"model":"{}:block/{}_stage3"}}}}}}"#,
                        mod_id, id, mod_id, id, mod_id, id, mod_id, id
                    ),
                )?;
                for stage in 0..4 {
                    std::fs::write(
                        block_models_root.join(format!("{}_stage{}.json", id, stage)),
                        format!(
                            r#"{{"parent":"minecraft:block/crop","textures":{{"crop":"{}:block/{}_stage{}"}}}}"#,
                            mod_id, id, stage
                        ),
                    )?;
                }
            }
            BlockModelKind::SingleFace => {
                std::fs::write(
                    blockstate_path,
                    format!(
                        r#"{{"variants":{{"":{{"model":"{}:block/{}"}}}}}}"#,
                        mod_id, id
                    ),
                )?;
                std::fs::write(
                    block_model_path,
                    format!(
                        r#"{{"parent":"minecraft:block/template_single_face","textures":{{"texture":"{}:block/{}"}}}}"#,
                        mod_id, id
                    ),
                )?;
            }
            BlockModelKind::Ore => {
                std::fs::write(
                    blockstate_path,
                    format!(
                        r#"{{"variants":{{"":{{"model":"{}:block/{}"}}}}}}"#,
                        mod_id, id
                    ),
                )?;
                std::fs::write(
                    block_model_path,
                    format!(
                        r#"{{"parent":"minecraft:block/ore","textures":{{"all":"{}:block/{}"}}}}"#,
                        mod_id, id
                    ),
                )?;
            }
            BlockModelKind::Carpet => {
                std::fs::write(
                    blockstate_path,
                    format!(
                        r#"{{"variants":{{"":{{"model":"{}:block/{}"}}}}}}"#,
                        mod_id, id
                    ),
                )?;
                std::fs::write(
                    block_model_path,
                    format!(
                        r#"{{"parent":"minecraft:block/carpet","textures":{{"wool":"{}:block/{}"}}}}"#,
                        mod_id, id
                    ),
                )?;
            }
            BlockModelKind::Ladder => {
                std::fs::write(
                    blockstate_path,
                    format!(
                        r#"{{"variants":{{"":{{"model":"{}:block/{}"}}}}}}"#,
                        mod_id, id
                    ),
                )?;
                std::fs::write(
                    block_model_path,
                    format!(
                        r#"{{"parent":"minecraft:block/ladder","textures":{{"texture":"{}:block/{}"}}}}"#,
                        mod_id, id
                    ),
                )?;
            }
            BlockModelKind::Lever => {
                std::fs::write(
                    blockstate_path,
                    format!(
                        r#"{{"variants":{{"":{{"model":"{}:block/{}"}}}}}}"#,
                        mod_id, id
                    ),
                )?;
                std::fs::write(
                    block_model_path,
                    format!(
                        r#"{{"parent":"minecraft:block/lever","textures":{{"texture":"{}:block/{}"}}}}"#,
                        mod_id, id
                    ),
                )?;
            }
            BlockModelKind::Button => {
                std::fs::write(
                    blockstate_path,
                    format!(
                        r#"{{"variants":{{"":{{"model":"{}:block/{}"}}}}}}"#,
                        mod_id, id
                    ),
                )?;
                std::fs::write(
                    block_model_path,
                    format!(
                        r#"{{"parent":"minecraft:block/button","textures":{{"texture":"{}:block/{}"}}}}"#,
                        mod_id, id
                    ),
                )?;
            }
            BlockModelKind::PressurePlate => {
                std::fs::write(
                    blockstate_path,
                    format!(
                        r#"{{"variants":{{"":{{"model":"{}:block/{}"}}}}}}"#,
                        mod_id, id
                    ),
                )?;
                std::fs::write(
                    block_model_path,
                    format!(
                        r#"{{"parent":"minecraft:block/pressure_plate","textures":{{"texture":"{}:block/{}"}}}}"#,
                        mod_id, id
                    ),
                )?;
            }
            BlockModelKind::Door => {
                std::fs::write(
                    blockstate_path,
                    format!(
                        r#"{{"variants":{{"facing=east,half=lower,hinge=left":{{"model":"{}:block/{}_bottom"}},"facing=east,half=upper,hinge=left":{{"model":"{}:block/{}_top"}}}}}}"#,
                        mod_id, id, mod_id, id
                    ),
                )?;
                std::fs::write(
                    block_models_root.join(format!("{}_bottom.json", id)),
                    format!(
                        r#"{{"parent":"minecraft:block/door_bottom","textures":{{"door":"{}:block/{}"}}}}"#,
                        mod_id, id
                    ),
                )?;
                std::fs::write(
                    block_models_root.join(format!("{}_top.json", id)),
                    format!(
                        r#"{{"parent":"minecraft:block/door_top","textures":{{"door":"{}:block/{}"}}}}"#,
                        mod_id, id
                    ),
                )?;
            }
            BlockModelKind::Trapdoor => {
                std::fs::write(
                    blockstate_path,
                    format!(
                        r#"{{"variants":{{"":{{"model":"{}:block/{}"}}}}}}"#,
                        mod_id, id
                    ),
                )?;
                std::fs::write(
                    block_model_path,
                    format!(
                        r#"{{"parent":"minecraft:block/trapdoor","textures":{{"texture":"{}:block/{}"}}}}"#,
                        mod_id, id
                    ),
                )?;
            }
            BlockModelKind::FenceGate => {
                std::fs::write(
                    blockstate_path,
                    format!(
                        r#"{{"variants":{{"":{{"model":"{}:block/{}"}}}}}}"#,
                        mod_id, id
                    ),
                )?;
                std::fs::write(
                    block_model_path,
                    format!(
                        r#"{{"parent":"minecraft:block/fence_gate","textures":{{"texture":"{}:block/{}"}}}}"#,
                        mod_id, id
                    ),
                )?;
            }
            BlockModelKind::Lantern => {
                std::fs::write(
                    blockstate_path,
                    format!(
                        r#"{{"variants":{{"hanging=false":{{"model":"{}:block/{}"}},"hanging=true":{{"model":"{}:block/{}_hanging"}}}}}}"#,
                        mod_id, id, mod_id, id
                    ),
                )?;
                std::fs::write(
                    block_model_path,
                    format!(
                        r#"{{"parent":"minecraft:block/template_lantern","textures":{{"lantern":"{}:block/{}"}}}}"#,
                        mod_id, id
                    ),
                )?;
                std::fs::write(
                    block_models_root.join(format!("{}_hanging.json", id)),
                    format!(
                        r#"{{"parent":"minecraft:block/template_hanging_lantern","textures":{{"lantern":"{}:block/{}"}}}}"#,
                        mod_id, id
                    ),
                )?;
            }
        }

        let item_model_path = item_models_root.join(format!("{}.json", id));
        std::fs::write(
            item_model_path,
            format!(r#"{{"parent":"{}:block/{}"}}"#, mod_id, id),
        )?;
    }

    let block_item_ids: Vec<_> = state.blocks.iter().map(|b| b.id.as_str()).collect();

    for item in &state.items {
        let id = &item.id;
        let mod_id = &state.mod_id;
        let item_model_path = item_models_root.join(format!("{}.json", id));
        if !block_item_ids.contains(&id.as_str()) {
            if item.kind == ItemKind::SpawnEgg {
                std::fs::write(
                    item_model_path,
                    format!(
                        r#"{{"parent":"minecraft:item/template_spawn_egg","layers":[{{"texture":"{}:item/{}"}}]}}"#,
                        mod_id, id
                    ),
                )?;
            } else {
                std::fs::write(
                    item_model_path,
                    format!(
                        r#"{{"parent":"minecraft:item/generated","textures":{{"layer0":"{}:item/{}"}}}}"#,
                        mod_id, id
                    ),
                )?;
            }
        }
    }

    Ok(())
}

fn generate_loot_table_resources(state: &ModState, _verbose: bool) -> Result<()> {
    let data_root = PathBuf::from("src/main/resources/data")
        .join(&state.mod_id)
        .join("loot_tables");
    create_dir_all(&data_root)?;

    for table in &state.loot_tables {
        let id = &table.id;
        let loot_type = &table.loot_type;
        let dir_name = match loot_type.as_str() {
            "chest" => "chests",
            "entity" => "entities",
            "block" => "blocks",
            _ => loot_type.as_str(),
        };
        let rolls = table.rolls.unwrap_or(1);
        let pool_dir = data_root.join(dir_name);
        create_dir_all(&pool_dir)?;
        let path = pool_dir.join(format!("{}.json", id));

        let entries: Vec<String> = table
            .entries
            .iter()
            .map(|e| {
                let name = if e.item.contains(':') {
                    e.item.clone()
                } else {
                    format!("{}:{}", state.mod_id, e.item)
                };
                let mut entry = format!(r#"{{"type":"minecraft:item","name":"{}""#, name);
                if let Some(w) = e.weight {
                    entry.push_str(&format!(r#","weight":{}"#, w));
                }
                if e.min_count != e.max_count || e.min_count != 1 {
                    entry.push_str(&format!(
                        r#","functions":[{{"function":"minecraft:set_count","count":{{"min":{},"max":{}}}}}]"#,
                        e.min_count, e.max_count
                    ));
                } else if e.min_count != 1 {
                    entry.push_str(&format!(
                        r#","functions":[{{"function":"minecraft:set_count","count":{}}}]"#,
                        e.min_count
                    ));
                }
                entry.push('}');
                entry
            })
            .collect();

        let entries_str = entries.join(",");
        let json = format!(
            r#"{{"pools":[{{"rolls":{},"entries":[{}]}}]}}"#,
            rolls, entries_str
        );
        std::fs::write(path, json)?;
    }

    Ok(())
}

fn generate_advancement_resources(state: &ModState, _verbose: bool) -> Result<()> {
    let data_root = PathBuf::from("src/main/resources/data")
        .join(&state.mod_id)
        .join("advancement");
    create_dir_all(&data_root)?;

    for adv in &state.advancements {
        let id = &adv.id;
        let path = data_root.join(format!("{}.json", id));

        let mut parts: Vec<String> = Vec::new();

        if let Some(parent) = &adv.parent {
            parts.push(format!(r#""parent":"{}""#, parent));
        }

        let mut display_parts: Vec<String> = Vec::new();
        display_parts.push(format!(r#""title":"{}""#, adv.title));
        display_parts.push(format!(r#""description":"{}""#, adv.description));
        display_parts.push(format!(r#""icon":{{"item":"{}"}}"#, adv.icon));

        if let Some(bg) = &adv.background {
            display_parts.push(format!(r#""background":"{}""#, bg));
        }

        let frame = adv.frame.as_deref().unwrap_or("task");
        display_parts.push(format!(r#""frame":"{}""#, frame));

        let show_toast = adv.show_toast.unwrap_or(true);
        display_parts.push(format!(r#""show_toast":{}"#, show_toast));

        let announce = adv.announce_to_chat.unwrap_or(true);
        display_parts.push(format!(r#""announce_to_chat":{}"#, announce));

        if let Some(hidden) = adv.hidden {
            display_parts.push(format!(r#""hidden":{}"#, hidden));
        }

        parts.push(format!(r#""display":{{{}}}"#, display_parts.join(",")));

        let mut criteria_str: Vec<String> = Vec::new();
        for criterion in &adv.criteria {
            let mut cond_parts: Vec<String> = Vec::new();
            for (k, v) in &criterion.conditions {
                cond_parts.push(format!(r#""{}":"{}""#, k, v));
            }
            let criterion_json = {
                let mut s = String::from("{");
                s.push_str(&format!("\"trigger\":\"{}\"", criterion.trigger));
                if !cond_parts.is_empty() {
                    s.push_str(",\"conditions\":{");
                    s.push_str(&cond_parts.join(","));
                    s.push('}');
                }
                s.push('}');
                s
            };

            criteria_str.push(format!(r#""{}":{}"#, criterion.id, criterion_json));
        }

        parts.push(format!(r#""criteria":{{{}}}"#, criteria_str.join(",")));

        if let Some(rewards) = &adv.rewards {
            let mut reward_parts: Vec<String> = Vec::new();
            if let Some(exp) = rewards.experience {
                reward_parts.push(format!(r#""experience":{}"#, exp));
            }
            if !rewards.loot.is_empty() {
                let loot_str: Vec<String> =
                    rewards.loot.iter().map(|l| format!(r#""{}""#, l)).collect();
                reward_parts.push(format!(r#""loot":[{}]"#, loot_str.join(",")));
            }
            if !rewards.item.is_empty() {
                let item_str: Vec<String> =
                    rewards.item.iter().map(|i| format!(r#""{}""#, i)).collect();
                reward_parts.push(format!(r#""item":[{}]"#, item_str.join(",")));
            }
            if !rewards.recipes.is_empty() {
                let recipe_str: Vec<String> = rewards
                    .recipes
                    .iter()
                    .map(|r| format!(r#""{}""#, r))
                    .collect();
                reward_parts.push(format!(r#""recipes":[{}]"#, recipe_str.join(",")));
            }
            if !reward_parts.is_empty() {
                parts.push(format!(r#""rewards":{{{}}}"#, reward_parts.join(",")));
            }
        }

        let json = format!(r#"{{{}}}"#, parts.join(","));
        std::fs::write(path, json)?;
    }

    Ok(())
}

fn generate_tag_resources(state: &ModState, _verbose: bool) -> Result<()> {
    let data_root = PathBuf::from("src/main/resources/data")
        .join(&state.mod_id)
        .join("tags");
    create_dir_all(&data_root)?;

    for tag in &state.tags {
        let dir_name = match tag.tag_type.as_str() {
            "block" => "blocks",
            "item" => "items",
            "entity" => "entity_types",
            "fluid" => "fluids",
            other => other,
        };
        let tag_dir = data_root.join(dir_name);
        create_dir_all(&tag_dir)?;
        let path = tag_dir.join(format!("{}.json", tag.id));

        let values: Vec<String> = tag
            .values
            .iter()
            .map(|v| {
                if v.contains(':') {
                    format!(r#""{}""#, v)
                } else {
                    format!(r#""{}:{}""#, state.mod_id, v)
                }
            })
            .collect();

        let replace = tag.replace.unwrap_or(false);
        let json = format!(
            r#"{{"replace":{},"values":[{}]}}"#,
            replace,
            values.join(",")
        );
        std::fs::write(path, json)?;
    }

    Ok(())
}

fn generate_sound_resources(state: &ModState, _verbose: bool) -> Result<()> {
    if state.sound_events.is_empty() {
        return Ok(());
    }

    let assets_root = PathBuf::from("src/main/resources/assets").join(&state.mod_id);
    create_dir_all(&assets_root)?;

    let path = assets_root.join("sounds.json");

    let mut entries: Vec<String> = Vec::new();
    for sound in &state.sound_events {
        let full_path = if sound.sound_path.contains(':') {
            sound.sound_path.clone()
        } else {
            format!("{}:{}", state.mod_id, sound.sound_path)
        };
        let subtitle = format!("sound.{}.{}", state.mod_id, sound.id);
        entries.push(format!(
            r#""{}":{{"subtitle":"{}","sounds":["{}"]}}"#,
            sound.id, subtitle, full_path
        ));
    }

    let json = format!(r#"{{{}}}"#, entries.join(","));
    std::fs::write(path, json)?;

    Ok(())
}

fn generate_biome_resources(state: &ModState, _verbose: bool) -> Result<()> {
    let data_root = PathBuf::from("src/main/resources/data")
        .join(&state.mod_id)
        .join("worldgen")
        .join("biome");
    create_dir_all(&data_root)?;

    for biome in &state.biomes {
        let id = &biome.id;
        let mut effects = String::new();
        let mut attributes = String::new();
        let mut has_effects = false;
        let mut has_attributes = false;

        if let Some(water_color) = biome.water_color {
            effects.push_str(&format!(r##""water_color":"#{:06X}""##, water_color as u32));
            has_effects = true;
        }
        if let Some(sky_color) = biome.sky_color {
            if has_attributes {
                attributes.push(',');
            }
            attributes.push_str(&format!(
                r##""minecraft:visual/sky_color":"#{:06X}""##,
                sky_color as u32
            ));
            has_attributes = true;
        }
        if let Some(water_fog_color) = biome.water_fog_color {
            if has_attributes {
                attributes.push(',');
            }
            attributes.push_str(&format!(
                r##""minecraft:visual/water_fog_color":"#{:06X}""##,
                water_fog_color as u32
            ));
            has_attributes = true;
        }
        if let Some(fog_color) = biome.fog_color {
            if has_attributes {
                attributes.push(',');
            }
            attributes.push_str(&format!(
                r##""minecraft:visual/fog_color":"#{:06X}""##,
                fog_color as u32
            ));
            has_attributes = true;
        }

        let effects_obj = if has_effects {
            format!("\"effects\":{{{}}},", effects)
        } else {
            String::new()
        };

        let attributes_obj = if has_attributes {
            format!("\"attributes\":{{{}}},", attributes)
        } else {
            String::new()
        };

        let mut spawners: HashMap<&str, Vec<String>> = HashMap::new();
        for spawn in &biome.mob_spawns {
            let category = spawn.category.as_deref().unwrap_or("monster");
            spawners.entry(category).or_default().push(format!(
                r#"{{"type":"{}","weight":{},"minCount":{},"maxCount":{}}}"#,
                spawn.entity_type, spawn.weight, spawn.min_count, spawn.max_count
            ));
        }

        let mut spawners_str = String::new();
        for (category, entries) in spawners {
            if !spawners_str.is_empty() {
                spawners_str.push(',');
            }
            spawners_str.push_str(&format!(r#""{}":[{}]"#, category, entries.join(",")));
        }

        let temperature = biome
            .temperature
            .map(|t| format!(r#""temperature":{},"#, t))
            .unwrap_or_default();
        let downfall = biome
            .downfall
            .map(|d| format!(r#""downfall":{},"#, d))
            .unwrap_or_default();
        let has_precipitation = biome
            .has_precipitation
            .map(|p| format!(r#""has_precipitation":{},"#, p))
            .unwrap_or_default();

        let features = if biome.features.is_empty() && state.features.is_empty() {
            String::from("[]")
        } else {
            let mut feature_refs: Vec<String> = biome
                .features
                .iter()
                .map(|f| format!(r#""{}""#, f))
                .collect();
            for feature in &state.features {
                feature_refs.push(format!(r#""{}:{}""#, state.mod_id, feature.id));
            }
            if feature_refs.is_empty() {
                String::from("[]")
            } else {
                format!(r#"[{}]"#, feature_refs.join(","))
            }
        };

        let structures = if biome.structures.is_empty() {
            String::from("[]")
        } else {
            format!(
                r#""structures":[{}],"#,
                biome
                    .structures
                    .iter()
                    .map(|s| format!(r#""{}""#, s))
                    .collect::<Vec<_>>()
                    .join(",")
            )
        };

        let json = format!(
            r#"{{{}{}{}{}{}"spawners":{{{},"spawn_costs":{{}},"features":{},"structures":{}}}"#,
            attributes_obj,
            effects_obj,
            temperature,
            downfall,
            has_precipitation,
            spawners_str,
            features,
            structures
        );

        std::fs::write(data_root.join(format!("{}.json", id)), json)?;
    }

    Ok(())
}

fn generate_configured_feature_resources(state: &ModState, _verbose: bool) -> Result<()> {
    let data_root = PathBuf::from("src/main/resources/data")
        .join(&state.mod_id)
        .join("worldgen")
        .join("configured_feature");
    create_dir_all(&data_root)?;

    for feature in &state.features {
        let id = &feature.id;
        let mut config = String::new();

        match feature.feature_type.as_str() {
            "ore" => {
                config.push_str(&format!(
                    r#""type":"minecraft:ore","target":{{"target":"{}","state":{{"Name":"{}"}}}},"size":{{"min_inclusive":{{"below":64}},"max_inclusive":{{"below":0}}}}"#,
                    feature.block.as_deref().unwrap_or("minecraft:stone"),
                    feature.block.as_deref().unwrap_or("minecraft:stone")
                ));
            }
            "blob" => {
                config.push_str(&format!(
                    r#""type":"minecraft:random_blob","state":{{"Name":"{}"}},"radius":{{"type":"minecraft:uniform","value":{{"min_inclusive":{},"max_inclusive":{}}}}}"#,
                    feature.block.as_deref().unwrap_or("minecraft:stone"),
                    feature.radius.unwrap_or(2),
                    feature.radius.unwrap_or(4)
                ));
            }
            "delta" => {
                config.push_str(&format!(
                    r#""type":"minecraft:delta_feature","state":{{"Name":"{}"}},"size":{}"#,
                    feature.block.as_deref().unwrap_or("minecraft:netherrack"),
                    feature.radius.unwrap_or(3)
                ));
            }
            _ => {
                config.push_str(&format!(
                    r#""type":"{}","block":"{}""#,
                    feature.feature_type,
                    feature.block.as_deref().unwrap_or("minecraft:stone")
                ));
            }
        }

        let json = format!(r#"{{{}}}"#, config);
        std::fs::write(data_root.join(format!("{}.json", id)), json)?;
    }

    Ok(())
}

fn generate_placed_feature_resources(state: &ModState, _verbose: bool) -> Result<()> {
    let data_root = PathBuf::from("src/main/resources/data")
        .join(&state.mod_id)
        .join("worldgen")
        .join("placed_feature");
    create_dir_all(&data_root)?;

    for feature in &state.features {
        let id = &feature.id;
        let mut placement = String::new();

        match feature.feature_type.as_str() {
            "ore" => {
                placement.push_str(r#""type":"minecraft:count","count":{{"type":"minecraft:uniform","value":{{"min_inclusive":1,"max_inclusive":8}}}}"#);
                placement.push(',');
                placement.push_str(r#""type":"minecraft:in_square""#);
                placement.push(',');
                placement.push_str(r#""type":"minecraft:height_range","height":{{"type":"minecraft:uniform","value":{{"min_inclusive":{"below":64},"max_inclusive":{"below":0}}}}}"#);
            }
            "blob" => {
                placement.push_str(r#""type":"minecraft:in_square""#);
                placement.push(',');
                placement.push_str(r#""type":"minecraft:height_range","height":{{"type":"minecraft:uniform","value":{{"min_inclusive":{"below":64},"max_inclusive":{"below":0}}}}}"#);
            }
            "delta" => {
                placement.push_str(r#""type":"minecraft:in_square""#);
                placement.push(',');
                placement.push_str(r#""type":"minecraft:height_range","height":{{"type":"minecraft:uniform","value":{{"min_inclusive":{"below":64},"max_inclusive":{"below":0}}}}}"#);
            }
            _ => {
                placement.push_str(r#""type":"minecraft:in_square""#);
            }
        }

        let json = format!(
            "{{\"feature\":\"{}:{}\",\"placement\":[{{{}}}]}}",
            state.mod_id, id, placement
        );
        std::fs::write(data_root.join(format!("{}.json", id)), json)?;
    }

    Ok(())
}

fn generate_dimension_type_resources(state: &ModState, _verbose: bool) -> Result<()> {
    let data_root = PathBuf::from("src/main/resources/data")
        .join(&state.mod_id)
        .join("dimension_type");
    create_dir_all(&data_root)?;

    for dim_type in &state.dimension_types {
        let id = &dim_type.id;
        let mut properties = Vec::new();

        if let Some(ultrawarm) = dim_type.ultrawarm {
            properties.push(format!(r#""ultrawarm":{}"#, ultrawarm));
        }
        if let Some(natural) = dim_type.natural {
            properties.push(format!(r#""natural":{}"#, natural));
        }
        if let Some(piglin_safe) = dim_type.piglin_safe {
            properties.push(format!(r#""piglin_safe":{}"#, piglin_safe));
        }
        if let Some(respawn_anchor_works) = dim_type.respawn_anchor_works {
            properties.push(format!(
                r#""respawn_anchor_works":{}"#,
                respawn_anchor_works
            ));
        }
        if let Some(bed_works) = dim_type.bed_works {
            properties.push(format!(r#""bed_works":{}"#, bed_works));
        }
        if let Some(has_raids) = dim_type.has_raids {
            properties.push(format!(r#""has_raids":{}"#, has_raids));
        }
        if let Some(has_skylight) = dim_type.has_skylight {
            properties.push(format!(r#""has_skylight":{}"#, has_skylight));
        }
        if let Some(has_ceiling) = dim_type.has_ceiling {
            properties.push(format!(r#""has_ceiling":{}"#, has_ceiling));
        }
        if let Some(coordinate_scale) = dim_type.coordinate_scale {
            properties.push(format!(r#""coordinate_scale":{}"#, coordinate_scale));
        }
        if let Some(logical_height) = dim_type.logical_height {
            properties.push(format!(r#""logical_height":{}"#, logical_height));
        }
        if let Some(min_y) = dim_type.min_y {
            properties.push(format!(r#""min_y":{}"#, min_y));
        }
        if let Some(height) = dim_type.height {
            properties.push(format!(r#""height":{}"#, height));
        }
        if let Some(monster_spawn_light_level) = dim_type.monster_spawn_light_level {
            properties.push(format!(
                r#""monster_spawn_light_level":{}"#,
                monster_spawn_light_level
            ));
        }
        if let Some(monster_spawn_block_light_limit) = dim_type.monster_spawn_block_light_limit {
            properties.push(format!(
                r#""monster_spawn_block_light_limit":{}"#,
                monster_spawn_block_light_limit
            ));
        }
        if let Some(ambient_light) = dim_type.ambient_light {
            properties.push(format!(r#""ambient_light":{}"#, ambient_light));
        }

        let json = format!(r#"{{{}}}"#, properties.join(","));
        std::fs::write(data_root.join(format!("{}.json", id)), json)?;
    }

    Ok(())
}

fn generate_dimension_resources(state: &ModState, _verbose: bool) -> Result<()> {
    let data_root = PathBuf::from("src/main/resources/data")
        .join(&state.mod_id)
        .join("dimension");
    create_dir_all(&data_root)?;

    for dimension in &state.dimensions {
        let id = &dimension.id;
        let json = format!(r#"{{"type":"{}"}}"#, dimension.dimension_type);
        std::fs::write(data_root.join(format!("{}.json", id)), json)?;
    }

    Ok(())
}

fn generate_structure_set_resources(state: &ModState, _verbose: bool) -> Result<()> {
    let data_root = PathBuf::from("src/main/resources/data")
        .join(&state.mod_id)
        .join("worldgen")
        .join("structure_set");
    create_dir_all(&data_root)?;

    for structure in &state.structures {
        let id = &structure.id;
        let spacing = structure.spacing.unwrap_or(32);
        let separation = structure.separation.unwrap_or(8);
        let salt = structure.salt.unwrap_or(123456789);
        let json = format!(
            r#"{{"placement":{{"type":"minecraft:random_spread","spacing":{},"separation":{},"salt":{}}},"structures":["{}"]}}"#,
            spacing, separation, salt, structure.structure_type
        );
        std::fs::write(data_root.join(format!("{}.json", id)), json)?;
    }

    Ok(())
}

fn prune_orphaned_java(
    java_root: &Path,
    client_root: &Path,
    state: &ModState,
    verbose: bool,
) -> Result<()> {
    let mut expected: HashSet<PathBuf> = HashSet::new();

    for (name, spec) in file_specs() {
        if (spec.should_exist)(state) {
            let mod_class_name = format!("{}.java", state.mod_name);
            let datagen_class_name = format!("{}DataGenerator.java", state.mod_name);
            let recipe_provider_class_name = format!("{}RecipeProvider.java", state.mod_name);
            let worldgen_provider_class_name = format!("{}WorldgenProvider.java", state.mod_name);
            let path = resolve_path(
                name,
                java_root,
                client_root,
                &mod_class_name,
                &datagen_class_name,
                &recipe_provider_class_name,
                &worldgen_provider_class_name,
            );
            expected.insert(path);
        }
    }

    for i in &state.items {
        if !i.tooltip.is_empty() {
            expected.insert(java_root.join(format!("{}Item.java", to_upper(&i.id))));
        }
    }

    let known_prefixes: [&str; 4] = [
        "ArmorProvider",
        "ShieldProvider",
        "FuelProvider",
        "CompostableProvider",
    ];

    for dir in [java_root, client_root] {
        if !dir.exists() {
            continue;
        }
        for entry in read_dir(dir).with_context(|| format!("Failed to read {}", dir.display()))? {
            let entry =
                entry.with_context(|| format!("Failed to read dir entry in {}", dir.display()))?;
            let path = entry.path();
            if !path.extension().map(|e| e == "java").unwrap_or(false) {
                continue;
            }
            if expected.contains(&path) {
                continue;
            }
            let file_name = path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
            if known_prefixes
                .iter()
                .any(|suffix| file_name.ends_with(suffix))
            {
                remove_file(&path)
                    .with_context(|| format!("Failed to prune orphaned {}", path.display()))?;
                if verbose {
                    vlog("pruned", &path);
                }
            }
        }
    }

    Ok(())
}
