use anyhow::Result;

use crate::config::StorageType;

mod local;
mod webdav;

pub struct File {
    pub path: String,
    pub hash: String,
    pub size: u64,
    pub data: String,
}

#[async_trait::async_trait]
pub trait Storage {
    async fn init(&self) -> Result<()>;
    async fn validate(&self) -> Result<()>;
    async fn write(&self, path: &str, content: &[u8]) -> Result<()>;
    async fn exists(&self, path: &str) -> bool;
    async fn get_absolute_path(&self, path: &str) -> String;
    async fn garbage_collection(&self) -> Result<()>;
    async fn check_missing_files(&self) -> Result<Vec<File>>;
}

pub fn get_storage(storage_type: StorageType) -> Box<dyn Storage> {
    match storage_type {
        StorageType::Local => Box::new(local::LocalStorage::new()),
        StorageType::Webdav(storage_config) => Box::new(webdav::WebdavStorage::new(storage_config)),
    }
}
