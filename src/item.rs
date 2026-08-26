use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Item {
    pub id: String,
}

impl Item {
    pub fn new(raw: &str) -> Result<Self> {
        let id = validate_id(raw)?;
        Ok(Self { id })
    }
}

fn validate_id(raw: &str) -> Result<String> {
    let id = raw.trim().to_lowercase();
    if id.is_empty() {
        anyhow::bail!("Item id cannot be empty.");
    }
    Ok(id)
}
