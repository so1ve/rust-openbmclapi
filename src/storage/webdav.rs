use std::collections::{BTreeMap, HashMap, HashSet};
use std::path::Path;
use std::sync::Arc;
use std::time::SystemTime;

use anyhow::{bail, Ok, Result};
use regex::Regex;
use reqwest_dav::list_cmd::{ListEntity, ListFile, ListFolder};
use reqwest_dav::{Auth, Client, ClientBuilder, Depth};
use tokio::sync::Mutex;
use tracing::{error, info, trace};

use super::{BMCLAPIFile, Storage};
use crate::config::WebdavStorageConfig;
use crate::utils::path_basename;

struct WebdavFile {
    path: String,
    size: usize,
}

#[derive(Clone)]
pub struct WebdavStorage {
    storage_config: WebdavStorageConfig,
    webdav_client: Client,
    files: Arc<Mutex<HashMap<String, WebdavFile>>>,
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
            files: Arc::new(Mutex::new(HashMap::new())),
            empty_files: vec![],
        }
    }

    fn download_basepath_with_dav_basepath(&self) -> String {
        Path::new(self.storage_config.dav_basepath.as_str())
            .join(self.storage_config.download_basepath.as_str())
            .to_string_lossy()
            .to_string()
    }

    async fn get_local_and_remote_files(
        &self,
        files: Vec<BMCLAPIFile>,
    ) -> Result<(
        BTreeMap<String, Vec<ListFile>>,
        HashMap<String, BMCLAPIFile>,
    )> {
        let self_files = self.files.lock().await;
        let remote_files: HashMap<String, BMCLAPIFile> = files
            .into_iter()
            .filter_map(|file| {
                if self_files.contains_key(&file.hash) {
                    return None;
                }
                Some((file.hash.clone(), file))
            })
            .collect();

        let folders: Vec<ListFolder> = self
            .webdav_client
            .list(
                &self.download_basepath_with_dav_basepath(),
                Depth::Number(1),
            )
            .await?
            .into_iter()
            .filter_map(|entity| {
                if let ListEntity::Folder(folder) = entity {
                    if path_basename(&folder.href)
                        != path_basename(&self.download_basepath_with_dav_basepath())
                    {
                        Some(folder)
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .collect();

        let mut local_files: BTreeMap<String, Vec<ListFile>> = BTreeMap::new();
        let mut tasks = Vec::with_capacity(folders.len());

        for folder in folders {
            let webdav_client = self.webdav_client.clone();
            let depth = Depth::Number(1);

            tasks.push(tokio::spawn(async move {
                let basename = path_basename(&folder.href).unwrap();
                let files: Vec<ListFile> = webdav_client
                    .list(&folder.href, depth)
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

                Ok((basename.to_owned(), files))
            }));
        }

        for task in tasks {
            let (folder, files) = task.await.unwrap().unwrap();
            trace!("Listed files in folder: {}", folder);
            local_files.insert(folder, files);
        }

        Ok((local_files, remote_files))
    }
}

#[async_trait::async_trait]
impl Storage for WebdavStorage {
    async fn init(&self) -> Result<()> {
        let basepath_exists = self
            .exists(&self.download_basepath_with_dav_basepath())
            .await;
        if !basepath_exists {
            info!(
                "Creating base path {}",
                &self.storage_config.download_basepath,
            );
            self.webdav_client
                .mkcol(&self.download_basepath_with_dav_basepath())
                .await?;
        }
        info!("Init success");

        Ok(())
    }

    async fn validate(&self) -> Result<()> {
        let temp_file_path = Path::new(&self.download_basepath_with_dav_basepath())
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
        let file_path = Path::new(&self.download_basepath_with_dav_basepath())
            .join(path)
            .to_string_lossy()
            .to_string();
        self.webdav_client.put(&file_path, content.to_vec()).await?;
        self.files.lock().await.insert(
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
        let regexp = Regex::new(&format!("^({protocol}?://)")).unwrap();
        let url = Path::new(
            &regexp
                .replace(&self.storage_config.endpoint, format!("$1{auth}@").as_str())
                .to_string(),
        )
        .join(&self.download_basepath_with_dav_basepath())
        .join(path)
        .to_string_lossy()
        .to_string();

        url
    }

    async fn check_missing_files(&self, files: Vec<BMCLAPIFile>) -> Result<Vec<BMCLAPIFile>> {
        let (local_files, remote_files) = self.get_local_and_remote_files(files).await?;

        let tasks = local_files.into_iter().map(|(folder, files)| {
            let clone = self.clone();
            let remote_files_clone = remote_files.clone();
            tokio::spawn(async move {
                trace!("Checking folder: {}", folder);
                let mut files_to_remove = vec![];
                for file in files {
                    let basename = path_basename(&file.href).unwrap();
                    if let Some(remote_file) = remote_files_clone.get(basename) {
                        let file_size = file.content_length as usize;
                        if remote_file.size == file_size {
                            clone.files.lock().await.insert(
                                basename.to_string(),
                                WebdavFile {
                                    size: file_size,
                                    path: file.href.clone(),
                                },
                            );
                            files_to_remove.push(basename.to_owned());
                        }
                    }
                }
                trace!("Checked folder: {}", folder);
                files_to_remove
            })
        });

        let mut remote_files = remote_files.clone();
        for task in tasks {
            let files_to_remove = task.await?;
            for file in files_to_remove {
                remote_files.remove(&file);
            }
        }

        Ok(remote_files.into_iter().map(|f| f.1).collect())
    }

    async fn cleanup_unused_files(&mut self, files: Vec<BMCLAPIFile>) -> Result<()> {
        let remote_file_hashes: HashSet<String> =
            // TODO: No more clones
            files.clone().into_iter().map(|file| file.hash).collect();
        let (local_files, _) = self.get_local_and_remote_files(files).await?;
        for files in local_files {
            for file in files.1 {
                let basename = path_basename(&file.href).unwrap();
                if !remote_file_hashes.contains(basename) {
                    info!("Deleting unused file: {}", &file.href);
                    self.webdav_client.delete(&file.href).await?;
                    self.files.lock().await.remove(basename);
                }
            }
        }

        Ok(())
    }
}
