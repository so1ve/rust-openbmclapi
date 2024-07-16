use std::collections::HashMap;
use std::hash::Hash;
use std::time::SystemTime;

use anyhow::{bail, Result};
use reqwest_dav::{Auth, Client, ClientBuilder};
use tracing::info;

use super::{BMCLAPIFile, Storage};
use crate::config::WebdavStorageConfig;

struct WebdavFile {
    path: String,
    size: usize,
}

pub struct WebdavStorage {
    storage_config: WebdavStorageConfig,
    webdav_client: Client,
    files: HashMap<String, WebdavFile>,
    empty_files: Vec<String>,
}

impl WebdavStorage {
    pub fn new(storage_config: WebdavStorageConfig) -> Self {
        let webdav_client = ClientBuilder::new()
            .set_host(storage_config.endpoint.clone())
            .set_auth(Auth::Basic(
                storage_config.username.clone(),
                storage_config.password.clone(),
            ))
            .build()
            .unwrap();
        Self {
            storage_config,
            webdav_client,
            files: HashMap::new(),
            empty_files: vec![],
        }
    }
}

#[async_trait::async_trait]
impl Storage for WebdavStorage {
    async fn init(&self) -> Result<()> {
        let basepath_exists = self.exists(&self.storage_config.basepath).await;
        if !basepath_exists {
            info!("Creating base path {}", &self.storage_config.basepath);
            self.webdav_client
                .mkcol(&self.storage_config.basepath)
                .await?;
        }
        Ok(())
    }

    async fn validate(&self) -> Result<()> {
        let temp_file_path = format!("{}/.check", self.storage_config.basepath);
        let temp_file_content = SystemTime::now().elapsed().unwrap().as_millis().to_string();
        let put_result = self
            .webdav_client
            .put(&temp_file_path, temp_file_content.as_bytes().to_vec())
            .await;
        if let Err(err) = put_result {
            info!("Error checking storage: {}", err);
            bail!(err);
        }
        let delete_result = self.webdav_client.delete(&temp_file_path).await;
        if let Err(err) = delete_result {
            info!("Failed to delete temp file: {}", err);
            bail!(err);
        }
        Ok(())
    }

    async fn write(&mut self, path: &str, content: &[u8], file: BMCLAPIFile) -> Result<()> {
        if content.len() == 0 {
            self.empty_files.push(file.hash);
            return Ok(());
        }
        let file_path = format!("{}/{}", self.storage_config.basepath, path);
        self.webdav_client.put(&file_path, content.to_vec()).await?;
        self.files.insert(
            file.hash,
            WebdavFile {
                size: content.len(),
                path: file.path,
            },
        );
        Ok(())
    }

    async fn exists(&self, path: &str) -> bool {
        self.webdav_client.get(path).await.is_ok()
    }

    async fn get_absolute_path(&self, path: &str) -> String {
        let protocol = if self.storage_config.endpoint.starts_with("https") {
            "https"
        } else {
            "http"
        };
        let auth = format!(
            "{}:{}",
            self.storage_config.username, self.storage_config.password
        );
        let url = format!(
            "{protocol}://{auth}@{}/{path}",
            self.storage_config.basepath
        );
        url
    }

    async fn get_missing_files(&self, files: Vec<BMCLAPIFile>) -> Result<Vec<BMCLAPIFile>> {
        let missing_files: HashMap<String, BMCLAPIFile> = files
            .into_iter()
            .filter_map(|file| {
                if self.files.contains_key(&file.hash) {
                    return None;
                }
                Some((file.hash.clone(), file))
            })
            .collect();

        unimplemented!("get_missing_files")
    }

    async fn garbage_collection(&self) -> Result<()> {
        unimplemented!()
    }
}
