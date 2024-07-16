use anyhow::Result;
use reqwest_dav::{Auth, Client, ClientBuilder};

use super::{File, Storage};
use crate::config::WebdavStorageConfig;

pub struct WebdavStorage {
    basepath: String,
    webdav_client: Client,
}

impl WebdavStorage {
    pub fn new(storage_config: WebdavStorageConfig) -> Self {
        let webdav_client = ClientBuilder::new()
            .set_host(storage_config.endpoint)
            .set_auth(Auth::Basic(
                storage_config.username,
                storage_config.password,
            ))
            .build()
            .unwrap();
        Self {
            basepath: storage_config.basepath,
            webdav_client,
        }
    }
}

#[async_trait::async_trait]
impl Storage for WebdavStorage {
    async fn init(&self) -> Result<()> {
        unimplemented!()
    }

    async fn validate(&self) -> Result<()> {
        unimplemented!()
    }

    async fn write(&self, path: &str, content: &[u8]) -> Result<()> {
        unimplemented!()
    }

    async fn exists(&self, path: &str) -> bool {
        unimplemented!()
    }

    async fn get_absolute_path(&self, path: &str) -> String {
        unimplemented!()
    }

    async fn garbage_collection(&self) -> Result<()> {
        unimplemented!()
    }

    async fn check_missing_files(&self) -> Result<Vec<File>> {
        unimplemented!()
    }
}
