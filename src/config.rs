use std::fs;
use std::path::PathBuf;

use serde::Deserialize;
use toml;

#[derive(Deserialize, Debug)]
pub enum StorageType {
    #[serde(rename = "local")]
    Local,
    #[serde(rename = "webdav")]
    Webdav {
        url: String,
        basepath: String,
        username: String,
        password: String,
    },
}

#[derive(Deserialize, Debug)]
pub struct Config {
    pub cluster_id: String,
    pub cluster_secret: String,
    pub storage: StorageType,
}

pub fn load_config(filename: PathBuf) -> Option<Config> {
    let contents = fs::read_to_string(&filename).ok()?;
    let config: Config = toml::from_str(&contents).ok()?;

    Some(config)
}
