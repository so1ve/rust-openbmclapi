use std::fs;
use std::io::Write;
use std::path::Path;

use anyhow::{bail, Result};
use tracing::error;

use super::{BMCLAPIFile, Storage};
use crate::config::LocalStorageConfig;

pub struct LocalStorage {
    storage_config: LocalStorageConfig,
}

impl LocalStorage {
    pub fn new(storage_config: LocalStorageConfig) -> Self {
        Self { storage_config }
    }
}

#[async_trait::async_trait]
impl Storage for LocalStorage {
    async fn validate(&self) -> Result<()> {
        let cache_dir = Path::new(&self.storage_config.cache_dir);
        if let Err(err) = fs::create_dir(cache_dir) {
            error!("Failed to create cache dir: {}", err);
            bail!(err);
        };
        let temp_file = cache_dir.join(".check");
        let mut file = match fs::File::create(&temp_file) {
            Ok(file) => file,
            Err(err) => {
                error!("Failed to create temp file: {}", err);
                bail!(err);
            }
        };
        if let Err(err) = file.write_all(b"") {
            error!("Failed to write temp file: {}", err);
            bail!(err);
        }
        if let Err(err) = fs::remove_file(&temp_file) {
            error!("Failed to delete temp file: {}", err);
            bail!(err);
        }

        Ok(())
    }

    async fn write(&mut self, path: &str, content: &[u8], file: BMCLAPIFile) -> Result<()> {
        let file_path = Path::new(&self.storage_config.cache_dir).join(path);
        let mut file = match fs::File::create(&file_path) {
            Ok(file) => file,
            Err(err) => {
                error!("Failed to create file: {}", err);
                bail!(err);
            }
        };
        if let Err(err) = file.write_all(content) {
            error!("Failed to write file: {}", err);
            bail!(err);
        }

        Ok(())
    }

    async fn exists(&self, path: &str) -> bool {
        Path::new(&self.storage_config.cache_dir)
            .join(path)
            .exists()
    }

    async fn get_absolute_path(&self, path: &str) -> String {
        Path::new(&self.storage_config.cache_dir)
            .join(path)
            .to_string_lossy()
            .to_string()
    }

    async fn check_missing_files(&mut self, files: Vec<BMCLAPIFile>) -> Result<Vec<BMCLAPIFile>> {
        unimplemented!()
    }

    async fn cleanup_unused_files(&mut self, files: Vec<BMCLAPIFile>) -> Result<()> {
        unimplemented!()
    }
}
