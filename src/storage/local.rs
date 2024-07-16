use anyhow::Result;

use super::{File, Storage};

pub struct LocalStorage {}

impl LocalStorage {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait::async_trait]
impl Storage for LocalStorage {
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
