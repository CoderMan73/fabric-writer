use anyhow::{Context, Result};
use std::env::var;
use std::fs::{copy, create_dir_all, read_dir};
use std::path::{Path, PathBuf};
use tempfile::{TempDir, tempdir};

use fabric_writer::commands::init::{InitArgs, run as run_init};

const CACHE_ROOT: &str = ".testing-cache";
const TEST_MOD_NAME: &str = "TestMod";
const TEST_VERSION: &str = "26.2";
// TODO: and this too probably: https://rustprojectprimer.com/measure/coverage.html
// TODO: should probably implement this: https://rustprojectprimer.com/testing/mutations.html

pub struct TestEnv {
    pub project_dir: PathBuf,
    _temp: TempDir,
}

impl TestEnv {
    pub fn new() -> Result<Self> {
        let _ = dotenvy::dotenv();
        let cache_dir = PathBuf::from(CACHE_ROOT).join(TEST_VERSION);
        if !cache_dir.join("TestMod").exists() {
            create_dir_all(&cache_dir).context("Failed to create test cache dir")?;

            let java_path = var("FABRIC_WRITER_TEST_JAVA")
                .expect("Set FABRIC_WRITER_TEST_JAVA to your Java path for tests");

            run_init(InitArgs {
                name: TEST_MOD_NAME.into(),
                version: TEST_VERSION.into(),
                options: vec!["datagen".into(), "splitSources".into()],
                dir: Some(cache_dir.display().to_string()),
                java_path,
                dangerous: true,
            })?;
        }

        let temp = tempdir().context("Failed to create temp dir")?;
        let source = cache_dir.join(TEST_MOD_NAME);
        let dest = temp.path().join(TEST_MOD_NAME);
        copy_dir_all(&source, &dest)?;

        Ok(Self {
            project_dir: dest,
            _temp: temp,
        })
    }
}

fn copy_dir_all(src: &Path, dst: &Path) -> Result<()> {
    create_dir_all(dst)?;
    for entry in read_dir(src)? {
        let entry = entry?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());
        if src_path.is_dir() {
            copy_dir_all(&src_path, &dst_path)?;
        } else {
            copy(&src_path, &dst_path)?;
        }
    }
    Ok(())
}
