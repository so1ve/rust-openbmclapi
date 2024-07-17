use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::time::SystemTime;

use anyhow::{bail, Ok, Result};
use reqwest_dav::list_cmd::{ListEntity, ListFile, ListFolder};
use reqwest_dav::{Auth, Client, ClientBuilder, Depth};
use tracing::{error, info};

use super::{BMCLAPIFile, Storage};
use crate::config::WebdavStorageConfig;
use crate::utils::path_basename;

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

    async fn get_local_and_remote_files(
        &mut self,
        files: Vec<BMCLAPIFile>,
    ) -> Result<(Vec<ListFile>, HashMap<String, BMCLAPIFile>)> {
        let remote_files: HashMap<String, BMCLAPIFile> = files
            .into_iter()
            .filter_map(|file| {
                if self.files.contains_key(&file.hash) {
                    return None;
                }
                Some((file.hash.clone(), file))
            })
            .collect();

        let mut folders: Vec<ListFolder> = self
            .webdav_client
            .list(&self.storage_config.download_basepath, Depth::Number(1))
            .await?
            .into_iter()
            .filter_map(|entity| {
                if let ListEntity::Folder(folder) = entity {
                    Some(folder)
                } else {
                    None
                }
            })
            .collect();
        folders.sort_by(|a, b| {
            let a_basename = path_basename(&a.href).unwrap();
            let b_basename = path_basename(&b.href).unwrap();
            a_basename.cmp(b_basename)
        });

        let mut local_files: Vec<Vec<ListFile>> = vec![];
        for folder in folders {
            let files: Vec<ListFile> = self
                .webdav_client
                .list(&folder.href, Depth::Number(1))
                .await?
                .into_iter()
                .filter_map(|entity| {
                    if let ListEntity::File(file) = entity {
                        Some(file)
                    } else {
                        None
                    }
                })
                .collect();
            local_files.push(files);
        }

        Ok((local_files.concat(), remote_files))
    }
}

#[async_trait::async_trait]
impl Storage for WebdavStorage {
    async fn init(&self) -> Result<()> {
        let basepath_exists = self.exists(&self.storage_config.download_basepath).await;
        if !basepath_exists {
            info!(
                "Creating base path {}",
                &self.storage_config.download_basepath
            );
            self.webdav_client
                .mkcol(&self.storage_config.download_basepath)
                .await?;
        }
        info!("Init success");

        Ok(())
    }

    async fn validate(&self) -> Result<()> {
        let temp_file_path = Path::new(&self.storage_config.download_basepath)
            .join(".check")
            .to_string_lossy()
            .to_string();
        let temp_file_content = SystemTime::now().elapsed().unwrap().as_millis().to_string();
        let put_result = self
            .webdav_client
            .put(&temp_file_path, temp_file_content.as_bytes().to_vec())
            .await;
        if let Err(err) = put_result {
            error!("Error checking storage: {}", err);
            bail!(err);
        }
        let delete_result = self.webdav_client.delete(&temp_file_path).await;
        if let Err(err) = delete_result {
            error!("Failed to delete temp file: {}", err);
            bail!(err);
        }
        info!("Validate success");

        Ok(())
    }

    async fn write(&mut self, path: &str, content: &[u8], file: BMCLAPIFile) -> Result<()> {
        if content.len() == 0 {
            self.empty_files.push(file.hash);
            return Ok(());
        }
        let file_path = Path::new(&self.storage_config.download_basepath)
            .join(path)
            .to_string_lossy()
            .to_string();
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
        let absolute_path = Path::new(&self.storage_config.download_basepath)
            .join(path)
            .to_string_lossy()
            .to_string();
        let url = format!("{protocol}://{auth}@{absolute_path}",);

        url
    }

    async fn check_missing_files(&mut self, files: Vec<BMCLAPIFile>) -> Result<Vec<BMCLAPIFile>> {
        let (local_files, mut remote_files) = self.get_local_and_remote_files(files).await?;

        for file in local_files {
            let basename = path_basename(&file.href).unwrap();
            if let Some(remote_file) = remote_files.get(basename) {
                let file_size = file.content_length as usize;
                if remote_file.size == file_size {
                    self.files.insert(
                        basename.to_string(),
                        WebdavFile {
                            size: file_size,
                            path: file.href.clone(),
                        },
                    );
                    remote_files.remove(basename);
                }
            }
        }

        Ok(remote_files.into_iter().map(|(_, file)| file).collect())
    }

    async fn cleanup_unused_files(&mut self, files: Vec<BMCLAPIFile>) -> Result<()> {
        let remote_file_hashes: HashSet<String> =
            // TODO: No more clones
            files.clone().into_iter().map(|file| file.hash).collect();
        let (local_files, _) = self.get_local_and_remote_files(files).await?;
        for file in local_files {
            let basename = path_basename(&file.href).unwrap();
            if !remote_file_hashes.contains(basename) {
                info!("Deleting unused file: {}", &file.href);
                self.webdav_client.delete(&file.href).await?;
                self.files.remove(basename);
            }
        }

        Ok(())
    }
}
