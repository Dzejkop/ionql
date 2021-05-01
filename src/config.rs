use std::fs::File;
use std::path::Path;

use anyhow::Context;
use serde::{Deserialize, Serialize};

use crate::row_mapping::Mapping;

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Config {
    pub mappings: Mapping,
}

impl Config {
    pub fn load_or_create_default(
        path: impl AsRef<Path>,
    ) -> anyhow::Result<Self> {
        let path = path.as_ref();

        if path.exists() {
            Self::load(path)
        } else {
            Self::create_default(path)
        }
    }

    fn create_default(path: impl AsRef<Path>) -> anyhow::Result<Self> {
        let path = path.as_ref();

        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).with_context(|| {
                format!(
                    "Failed to create parent directories for {}",
                    path.display()
                )
            })?;
        }

        let file = File::create(path)
            .with_context(|| format!("Failed to create {}", path.display()))?;

        let default_config = Config::default();

        serde_json::to_writer_pretty(file, &default_config)?;

        Ok(default_config)
    }

    fn load(path: impl AsRef<Path>) -> anyhow::Result<Self> {
        let path = path.as_ref();
        let file = File::open(path)
            .with_context(|| format!("Failed to open {}", path.display()))?;

        let config = serde_json::from_reader(file)
            .context("Failed to deserialize config")?;

        Ok(config)
    }
}
