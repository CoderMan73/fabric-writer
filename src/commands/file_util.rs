use anyhow::Result;
use std::path::PathBuf;

pub const BEGIN_MARKER: &str = "# ============================================\n\
     # FABRIC-WRITER — MANAGED SECTION\n\
     # ============================================";
pub const END_MARKER: &str = "# ============================================\n\
     # END FABRIC-WRITER MANAGED\n\
     # ============================================";

// TODO: this code is kind of cringe and I should probably deprecate it
// and get rid of it lol
/// Replace or append a managed section in a text file using explicit markers.
///
/// If `begin` exists, everything between `begin` and `end` is replaced.
/// If `begin` does not exist, the full managed block is appended.
pub fn update_managed_section(path: &PathBuf, begin: &str, end: &str, block: &str) -> Result<()> {
    let existing_raw = std::fs::read_to_string(path).unwrap_or_default();
    let existing = existing_raw.replace("\r\n", "\n");

    let new_contents = if existing.contains(begin) {
        let before = existing.split(begin).next().unwrap_or_default();
        let after = existing.split(end).nth(1).unwrap_or_default();
        format!("{}{}{}", before, block, after)
    } else {
        format!("{}{}", existing, block)
    };

    std::fs::write(path, new_contents)?;
    Ok(())
}

/// Write inner content inside the default managed section.
///
/// If the managed section already exists, only its inner content is replaced.
/// If it does not exist, a new managed section is appended to the file.
pub fn write_managed_section(path: &PathBuf, inner: &str) -> Result<()> {
    let block = format!("\n{BEGIN_MARKER}\n{inner}\n{END_MARKER}\n");
    update_managed_section(path, BEGIN_MARKER, END_MARKER, &block)
}
