use std::fs;
use std::path::PathBuf;

use anyhow::Result;
use serde::Deserialize;
use toml;

#[derive(Deserialize)]
pub struct WebdavStorageConfig {
    pub endpoint: String,
    pub basepath: String,
    pub username: String,
    pub password: String,
}

#[derive(Deserialize)]
#[serde(tag = "type")]
pub enum StorageType {
    #[serde(rename = "local")]
    Local,
    #[serde(rename = "webdav")]
    Webdav(WebdavStorageConfig),
}

#[derive(Deserialize)]
pub struct Config {
    pub cluster_id: String,
    pub cluster_secret: String,
    pub storage: Vec<StorageType>,
}

pub fn load_config(filename: PathBuf) -> Result<Config> {
    let contents = fs::read_to_string(&filename)?;
    let config: Config = toml::from_str(&contents)?;

    Ok(config)
}
