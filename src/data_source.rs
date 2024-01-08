use std::{env, path::PathBuf};

use log::{error, info};
use tokio::fs::{self, create_dir};

#[derive(Clone)]
pub struct DataSource {
    directory: PathBuf,
    max_size: u64,
}

impl DataSource {
    pub fn new(directory: PathBuf, max_size: u64) -> Self {
        Self {
            directory,
            max_size,
        }
    }

    pub fn from_env() -> Self {
        Self::new(
            env::var("DATA_DIRECTORY")
                .unwrap_or_else(|_| "data".to_string())
                .into(),
            env::var("MAX_FILE_SIZE")
                .unwrap_or_else(|_| "1048576".to_string())
                .parse()
                .expect("Error while parsing 'MAX_FILE_SIZE' environment variable"),
        )
    }

    pub async fn create_directory(&self) {
        if !self.directory.exists() {
            create_dir(&self.directory)
                .await
                .expect("Error while creating data directory");
            info!("Data directory created");
        }
    }

    pub async fn get_file(&self, id: String, filename: String) -> Option<Vec<u8>> {
        let path = self.directory.clone().join(&id).join(&filename);
        if path.exists() && path.is_file() {
            if let Ok(metadata) = path.metadata() {
                let size = metadata.len();
                if size > self.max_size {
                    info!(
                        "File {id} filename {filename} is bigger than {}, file will be ignored",
                        self.max_size
                    );
                }
            }
            let contents = fs::read(path).await;
            if let Err(err) = contents {
                error!("Error while reading file to get {id}: {err:?}");
                return None;
            }
            let contents = contents.unwrap();
            Some(contents)
        } else {
            None
        }
    }
}
