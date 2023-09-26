use crate::utils::error::Result;
use std::path::{Path, PathBuf};
use std::str;
use toml;

use serde::Deserialize;

#[derive(Deserialize, Debug, Default)]
pub struct SlangConfig {
    pub workspace: ConfigWorkspace,

    pub json: Option<JsonCodegen>,
    pub python: Option<PythonCodegen>,
}

#[derive(Deserialize, Debug, Default)]
pub struct ConfigWorkspace {
    pub root: Option<PathBuf>,
}

#[derive(Deserialize, Debug, Default)]
pub struct JsonCodegen {
    #[serde(default)]
    pub enabled: bool,
}

#[derive(Deserialize, Debug, Default)]
pub struct PythonCodegen {
    #[serde(default)]
    pub enabled: bool,
}

impl SlangConfig {
    pub async fn load(file: &Path, working_dir: &PathBuf) -> Result<Self> {
        let exists = tokio::fs::try_exists(file).await?;

        let mut slang_config = if !exists {
            SlangConfig::default()
        } else {
            let contents = tokio::fs::read(file).await?;
            let str_contents = str::from_utf8(contents.as_slice()).unwrap();
            toml::from_str(str_contents)?
        };

        if slang_config.workspace.root.is_none() {
            slang_config.workspace.root = Some(working_dir.clone());
        }

        if let Some(root) = &slang_config.workspace.root {
            let root = root.clone();
            let root = root.canonicalize()?;
            slang_config.workspace.root = Some(root);
        }

        Ok(slang_config)
    }
}
