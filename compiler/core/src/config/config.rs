use crate::utils::error::Result;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::str;
use toml;

use serde::Deserialize;

#[derive(Deserialize, Debug, Default)]
pub struct RuuLangConfig {
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

impl RuuLangConfig {
    pub async fn find(file: &Path) -> Option<PathBuf> {
        let mut file_buf = file.to_path_buf();
        let file_name = file_buf
            .file_name()
            .map_or(OsStr::new("ruu.toml"), |f| f)
            .to_str()
            .unwrap()
            .to_string();

        loop {
            if !file_buf.pop() {
                break None;
            }

            let file = &file_buf.join(&file_name);
            let exists = tokio::fs::try_exists(file).await.unwrap_or(false);

            if exists {
                break Some(file.clone());
            }
        }
    }

    pub async fn load(file: &Option<PathBuf>, working_dir: &PathBuf) -> Result<Self> {
        let mut ruulang_config = if let Some(path) = file {
            let contents = tokio::fs::read(path).await?;
            let str_contents = str::from_utf8(contents.as_slice()).unwrap();
            toml::from_str(str_contents)?
        } else {
            RuuLangConfig::default()
        };

        if ruulang_config.workspace.root.is_none() {
            ruulang_config.workspace.root = Some(working_dir.clone());
        }

        if let Some(root) = &ruulang_config.workspace.root {
            let root = root.clone();
            let root = root.canonicalize()?;
            ruulang_config.workspace.root = Some(root);
        }

        Ok(ruulang_config)
    }
}
