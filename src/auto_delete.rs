use std::{env, time::Duration};

use log::{debug, error, info};
use tokio::{
    fs::{read_dir, remove_dir_all},
    time::sleep,
};

use crate::data_source::DataSource;

// implete auto delete feature for data source
impl DataSource {
    pub fn start_auto_delete(self) {
        let interval = env::var("AUTO_DELETE_CHECK_INTERVAL")
            .unwrap_or_else(|_| "120".to_string()) // 2 minutes
            .parse::<u64>()
            .expect("Error while parsing environment variable 'AUTO_DELETE_CHECK_INTERVAL'");
        let delete_older_than = env::var("AUTO_DELETE_OLDER_THAN")
            .unwrap_or_else(|_| "172800".to_string()) // 2 days
            .parse::<u64>()
            .expect("Error while parsing environment variable 'AUTO_DELETE_OLDER_THANL'");
        if interval == 0 || delete_older_than == 0 {
            // auto delete is disabled
            return;
        }
        tokio::spawn(async move {
            info!("Auto deletion started");
            loop {
                let mut content = read_dir(&self.directory)
                    .await
                    .expect("Error while listing directories");
                while let Some(dir) = content.next_entry().await.unwrap() {
                    if let Ok(metadata) = dir.metadata().await {
                        if let Ok(modified) = metadata.modified() {
                            let elpased = modified.elapsed().unwrap();
                            if elpased >= Duration::from_secs(delete_older_than) {
                                debug!(
                                    "Delete directory {}",
                                    dir.file_name().to_str().unwrap_or_default()
                                );
                                if let Err(e) = remove_dir_all(dir.path()).await {
                                    error!(
                                        "Error while removing directory file {}: {e:?}",
                                        dir.file_name().to_str().unwrap_or_default()
                                    );
                                }
                            }
                        }
                    }
                }
                sleep(Duration::from_secs(interval)).await;
            }
        });
    }
}
