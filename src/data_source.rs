use std::{
    env,
    path::{Path, PathBuf},
};

use axum::{http::StatusCode, response::IntoResponse};
use log::{error, info};
use tokio::{
    fs::{self, create_dir, remove_dir, remove_file, File, OpenOptions},
    io::{AsyncReadExt, AsyncWriteExt},
};

use crate::id::{generate_change_key, generate_file_id, hash_change_key};

#[derive(Clone)]
pub struct DataSource {
    pub directory: PathBuf,
    pub max_size: u64,
}

#[derive(Debug)]
pub enum DataSourceError {
    InvalidFilename,
    EmptyBody,
    TooBig,
    InvalidChangeKey,
    NotFound,
    InternalServerError,
}

impl From<tokio::io::Error> for DataSourceError {
    fn from(_value: tokio::io::Error) -> Self {
        Self::InternalServerError
    }
}

impl IntoResponse for DataSourceError {
    fn into_response(self) -> axum::response::Response {
        match self {
            Self::InvalidFilename => (StatusCode::BAD_REQUEST, "Invalid filename"),
            Self::EmptyBody => (StatusCode::BAD_REQUEST, "Empty body"),
            Self::TooBig => (StatusCode::BAD_REQUEST, "File is too big"),
            Self::InvalidChangeKey => (StatusCode::FORBIDDEN, "Invalid change key"),
            Self::NotFound => (StatusCode::NOT_FOUND, "Not Found"),
            Self::InternalServerError => {
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error")
            }
        }
        .into_response()
    }
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

    fn check_filename(filename: &str) -> bool {
        if filename.contains('/') {
            return false;
        }
        if filename.contains("change.key") {
            return false;
        }
        true
    }

    pub async fn get_file(&self, id: String, filename: String) -> Option<Vec<u8>> {
        if !Self::check_filename(&id) || !Self::check_filename(&filename) {
            return None;
        }
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
            match fs::read(path).await {
                Ok(contents) => Some(contents),
                Err(e) => {
                    error!("Error while reading file to get {id}: {e:?}");
                    None
                }
            }
        } else {
            None
        }
    }

    pub async fn create_file(
        &self,
        filename: String,
        bytes: Vec<u8>,
    ) -> Result<(String, String), DataSourceError> {
        if !Self::check_filename(&filename) {
            return Err(DataSourceError::InvalidFilename);
        }
        if bytes.is_empty() {
            return Err(DataSourceError::EmptyBody);
        }
        if bytes.len() as u64 > self.max_size {
            return Err(DataSourceError::TooBig);
        }
        let id = generate_file_id();
        let path = self.directory.clone().join(&id);
        create_dir(&path)
            .await
            .map_err(|_| DataSourceError::InternalServerError)?;

        // generate and save change key
        let change_key = generate_change_key();
        {
            let change_key_path = path.join("change.key");
            let mut change_key_file = File::create(change_key_path).await?;
            let hashed_change_key = hash_change_key(&change_key);
            change_key_file.write_all(&hashed_change_key).await?;
        }

        // create and save file
        let file_path = path.join(filename);
        let mut file = File::create(file_path)
            .await
            .map_err(|_| DataSourceError::InternalServerError)?;
        file.write_all(&bytes)
            .await
            .map_err(|_| DataSourceError::InternalServerError)?;
        Ok((id, change_key))
    }

    async fn verify_change_key(
        &self,
        path: &Path,
        change_key: String,
    ) -> Result<(), DataSourceError> {
        let change_key_path = path.join("change.key");
        let mut change_key_file = File::open(change_key_path).await?;
        let mut change_key_buf: [u8; 32] = [0; 32];
        change_key_file.read_exact(&mut change_key_buf).await?;
        let hashed_change_key = hash_change_key(&change_key);
        if change_key_buf.into_iter().collect::<Vec<u8>>() != hashed_change_key {
            return Err(DataSourceError::InvalidChangeKey);
        }
        Ok(())
    }

    pub async fn modify_file(
        &self,
        id: String,
        filename: String,
        change_key: String,
        bytes: Vec<u8>,
    ) -> Result<(), DataSourceError> {
        if !Self::check_filename(&id) || !Self::check_filename(&filename) {
            return Err(DataSourceError::NotFound);
        }

        if bytes.is_empty() {
            return Err(DataSourceError::EmptyBody);
        }
        if bytes.len() as u64 > self.max_size {
            return Err(DataSourceError::TooBig);
        }

        let path = self.directory.clone().join(id);
        if !path.exists() {
            return Err(DataSourceError::NotFound);
        }

        self.verify_change_key(&path, change_key).await?;

        let file_path = path.join(filename);
        let mut file = OpenOptions::new().write(true).open(file_path).await?;
        file.write_all(&bytes).await?;

        Ok(())
    }

    pub async fn delete_file(
        &self,
        id: String,
        filename: String,
        change_key: String,
    ) -> Result<(), DataSourceError> {
        if !Self::check_filename(&id) || !Self::check_filename(&filename) {
            return Err(DataSourceError::NotFound);
        }

        let path = self.directory.clone().join(id);
        if !path.exists() {
            return Err(DataSourceError::NotFound);
        }

        self.verify_change_key(&path, change_key).await?;

        let file_path = path.join(filename);
        if !file_path.exists() {
            return Err(DataSourceError::NotFound);
        }

        remove_file(&file_path).await?;
        remove_file(path.join("change.key")).await?;
        remove_dir(&path).await?;

        Ok(())
    }
}
