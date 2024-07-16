use anyhow::Result;
use tracing::info;

use crate::config::StorageType;

mod local;
mod webdav;

pub struct BMCLAPIFile {
    pub path: String,
    pub hash: String,
    pub size: usize,
    pub mtime: u64,
}

#[async_trait::async_trait]
pub trait Storage {
    async fn init(&self) -> Result<()>;
    async fn validate(&self) -> Result<()>;
    async fn write(&mut self, path: &str, content: &[u8], file: BMCLAPIFile) -> Result<()>;
    async fn exists(&self, path: &str) -> bool;
    async fn get_absolute_path(&self, path: &str) -> String;
    async fn get_missing_files(&self, files: Vec<BMCLAPIFile>) -> Result<Vec<BMCLAPIFile>>;
    async fn garbage_collection(&self) -> Result<()>;
}

pub fn get_storage(storage_type: StorageType) -> Box<dyn Storage> {
    info!("Using storage type: {}", storage_type);
    match storage_type {
        StorageType::Local => Box::new(local::LocalStorage::new()),
        StorageType::Webdav(storage_config) => Box::new(webdav::WebdavStorage::new(storage_config)),
    }
}
