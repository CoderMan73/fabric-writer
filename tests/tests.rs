//! Integration tests for fabric-writer.
#![allow(missing_docs)]

mod common;

use anyhow::Result;
use common::TestEnv;
use serial_test::serial;
use std::fs::read;
use std::path::PathBuf;

#[test]
#[ignore]
#[serial]
fn cache_copies_correctly() -> Result<()> {
    let env = TestEnv::new()?;

    // Verify key files from the cache were copied identically to the temp project.
    let cache = PathBuf::from(".testing-cache/26.2/TestMod");

    for relative in ["gradle.properties", ".fw/fabric-writer.yml"] {
        let cache_file = cache.join(relative);
        let temp_file = env.project_dir.join(relative);

        assert!(cache_file.exists(), "Cache file missing: {}", relative);
        assert!(temp_file.exists(), "Temp file missing: {}", relative);

        let cache_contents = read(&cache_file)?;
        let temp_contents = read(&temp_file)?;
        assert_eq!(
            cache_contents, temp_contents,
            "File contents differ: {}",
            relative
        );
    }

    Ok(())
}
