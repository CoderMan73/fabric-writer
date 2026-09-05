use anyhow::{Context, Result};
use genco::fmt::{self, IoWriter};
use genco::lang::java::{self, Java, Tokens};
use std::collections::HashSet;
use std::fs::{create_dir_all, read as fs_read, read_dir, remove_file, write as fs_write};
use std::path::{Path, PathBuf};

use crate::state::{Entity, ModState};
use crate::tokengen::{
    BuildFn, build_datagen_entrypoint, build_lang_provider, build_main_mod_class,
    build_mod_block_ids, build_mod_block_item_ids, build_mod_blocks, build_mod_item_ids,
    build_mod_items, build_model_provider,
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
        }
    }

    /// Marks every file that utilizes the entity as dirty.
    pub fn from_entity(entity: &Entity) -> Self {
        match entity {
            Entity::Item(_) => Self {
                mod_item_ids: true,
                mod_items: true,
                lang_provider: true,
                model_provider: true,
                datagen_entrypoint: true,
                ..Default::default()
            },
            Entity::Block(_) => Self {
                mod_blocks: true,
                mod_block_ids: true,
                mod_block_item_ids: true,
                lang_provider: true,
                model_provider: true,
                datagen_entrypoint: true,
                ..Default::default()
            },
            Entity::Recipe(_) => Self {
                lang_provider: true,
                model_provider: true,
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

    for (name, spec) in file_specs() {
        let path = resolve_path(
            name,
            &java_root,
            &client_root,
            &mod_class_name,
            &datagen_class_name,
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

    Ok(())
}

fn resolve_path(
    name: &str,
    java_root: &Path,
    client_root: &Path,
    mod_class_name: &str,
    datagen_class_name: &str,
) -> PathBuf {
    match name {
        "ModItemIds.java" => java_root.join(name),
        "ModItems.java" => java_root.join(name),
        "<ModName>.java" => java_root.join(mod_class_name),
        "LangProvider.java" => client_root.join(name),
        "<ModName>DataGenerator.java" => client_root.join(datagen_class_name),
        "ModelProvider.java" => client_root.join(name),
        "ModBlocks.java" => java_root.join(name),
        "ModBlockIds.java" => java_root.join(name),
        "ModBlockItemIds.java" => java_root.join(name),
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
pub fn write(path: &PathBuf, tokens: Tokens, package: &str) -> Result<()> {
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
