use std::fmt::{self, Display, Formatter};
use std::fs;
use std::path::PathBuf;

use anyhow::Result;
use serde::Deserialize;
use toml;

#[derive(Clone, Deserialize)]
pub struct LocalStorageConfig {
    pub cache_dir: String,
}

fn dav_basepath_default() -> String {
    "/dav".into()
}

#[derive(Clone, Deserialize)]
pub struct WebdavStorageConfig {
    pub endpoint: String,
    #[serde(default = "dav_basepath_default")]
    pub dav_basepath: String,
    pub download_basepath: String,
    // TODO: Redirect measure requests
    pub measure_basepath: Option<String>,
    pub username: String,
    pub password: String,
}

#[derive(Clone, Deserialize)]
#[serde(tag = "type")]
pub enum StorageType {
    #[serde(rename = "local")]
    Local(LocalStorageConfig),
    #[serde(rename = "webdav")]
    Webdav(WebdavStorageConfig),
}

impl Display for StorageType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Local(_) => write!(f, "local"),
            Self::Webdav(_) => write!(f, "webdav"),
        }
    }
}

fn bmclapi_default() -> String {
    "https://openbmclapi.bangbang93.com".into()
}

#[derive(Deserialize)]
pub struct Config {
    #[serde(default = "bmclapi_default")]
    pub bmclapi: String,
    pub cluster_id: String,
    pub cluster_secret: String,
    pub storage: Vec<StorageType>,
}

pub fn load_config(filename: PathBuf) -> Result<Config> {
    let contents = fs::read_to_string(&filename)?;
    let config: Config = toml::from_str(&contents)?;

    Ok(config)
}
