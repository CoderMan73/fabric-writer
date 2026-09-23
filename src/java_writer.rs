use anyhow::{Context, Result};
use genco::fmt::{self, IoWriter};
use genco::lang::java::{self, Java, Tokens};
use std::collections::HashSet;
use std::fs::{create_dir_all, read as fs_read, read_dir, remove_file, write as fs_write};
use std::path::{Path, PathBuf};

use crate::state::{BlockModelKind, Entity, ModState};
use crate::tokengen::{
    BuildFn, build_datagen_entrypoint, build_item_class, build_lang_provider, build_main_mod_class,
    build_mod_advancements, build_mod_biomes, build_mod_block_ids, build_mod_block_item_ids,
    build_mod_blocks, build_mod_creative_tabs, build_mod_dimensions, build_mod_features,
    build_mod_item_ids, build_mod_items, build_mod_loot_tables, build_mod_mobs,
    build_mod_sound_events, build_mod_structures, build_mod_tags, build_model_provider,
    build_recipe_provider, to_upper,
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
                ..Default::default()
            },
            Entity::Dimension(_) => Self {
                mod_dimensions: true,
                mod_class: true,
                datagen_entrypoint: true,
                ..Default::default()
            },
            Entity::Structure(_) => Self {
                mod_structures: true,
                mod_class: true,
                datagen_entrypoint: true,
                ..Default::default()
            },
            Entity::Feature(_) => Self {
                mod_features: true,
                mod_class: true,
                datagen_entrypoint: true,
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

    for (name, spec) in file_specs() {
        let path = resolve_path(
            name,
            &java_root,
            &client_root,
            &mod_class_name,
            &datagen_class_name,
            &recipe_provider_class_name,
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
) -> PathBuf {
    match name {
        "ModItemIds.java" => java_root.join(name),
        "ModItems.java" => java_root.join(name),
        "<ModName>.java" => java_root.join(mod_class_name),
        "LangProvider.java" => client_root.join(name),
        "<ModName>DataGenerator.java" => client_root.join(datagen_class_name),
        "ModelProvider.java" => client_root.join(name),
        "<ModName>RecipeProvider.java" => client_root.join(recipe_provider_class_name),
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
            BlockModelKind::CubeBottomTop
            | BlockModelKind::CubeColumn
            | BlockModelKind::Orientable => {
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
                        r#"{{"parent":"minecraft:block/cube_bottom_top","textures":{{"bottom":"{}:block/{}_bottom","top":"{}:block/{}_top","side":"{}:block/{}"}}}}"#,
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
            _ => {
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
        }

        let item_model_path = item_models_root.join(format!("{}.json", id));
        if !item_model_path.exists() {
            std::fs::write(
                item_model_path,
                format!(
                    r#"{{"parent":"minecraft:item/generated","textures":{{"layer0":"{}:item/{}"}}}}"#,
                    mod_id, id
                ),
            )?;
        }
    }

    for item in &state.items {
        let id = &item.id;
        let mod_id = &state.mod_id;
        let item_model_path = item_models_root.join(format!("{}.json", id));
        if !item_model_path.exists() {
            std::fs::write(
                item_model_path,
                format!(
                    r#"{{"parent":"minecraft:item/generated","textures":{{"layer0":"{}:item/{}"}}}}"#,
                    mod_id, id
                ),
            )?;
        }
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
            let path = resolve_path(
                name,
                java_root,
                client_root,
                &mod_class_name,
                &datagen_class_name,
                &recipe_provider_class_name,
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
