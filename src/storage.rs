use anyhow::Result;
use tracing::info;

use crate::config::StorageType;

mod local;
mod webdav;

#[derive(Clone, Debug)]
pub struct BMCLAPIFile {
    pub path: String,
    pub hash: String,
    pub size: usize,
    pub mtime: u64,
}

#[async_trait::async_trait]
pub trait Storage {
    async fn init(&self) -> Result<()> {
        Ok(())
    }
    async fn validate(&self) -> Result<()>;
    async fn write(&mut self, path: &str, content: &[u8], file: BMCLAPIFile) -> Result<()>;
    async fn exists(&self, path: &str) -> bool;
    async fn get_absolute_path(&self, path: &str) -> String;
    async fn check_missing_files(&self, files: Vec<BMCLAPIFile>) -> Result<Vec<BMCLAPIFile>>;
    async fn cleanup_unused_files(&mut self, files: Vec<BMCLAPIFile>) -> Result<()>;
}

pub fn get_storage(storage_type: StorageType) -> Box<dyn Storage> {
    info!("Using storage type: {}", storage_type);
    match storage_type {
        StorageType::Local(storage_config) => Box::new(local::LocalStorage::new(storage_config)),
        StorageType::Webdav(storage_config) => Box::new(webdav::WebdavStorage::new(storage_config)),
    }
}
