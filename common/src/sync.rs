use anki::collection::Collection;
use anki::sync::collection::normal::SyncOutput;
use anki::sync::login::{sync_login, SyncAuth};
use anki_proto::sync::sync_status_response;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone)]
pub enum SyncStatus {
    Idle,
    Authenticating,
    CheckingStatus,
    SyncingCollection {
        progress: String,
    },
    SyncingMedia {
        checked: String,
        added: String,
        removed: String,
    },
    Complete {
        message: String,
    },
    Error {
        message: String,
    },
}

#[derive(Debug, Clone)]
pub struct SyncResult {
    pub success: bool,
    pub message: String,
    pub server_message: Option<String>,
}

pub struct SyncManager {
    http_client: reqwest::Client,
    sync_status: Arc<Mutex<SyncStatus>>,
}

impl SyncManager {
    pub fn new() -> Self {
        Self {
            http_client: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(60))
                .build()
                .unwrap(),
            sync_status: Arc::new(Mutex::new(SyncStatus::Idle)),
        }
    }

    pub fn get_status(&self) -> SyncStatus {
        self.sync_status.lock().unwrap().clone()
    }

    pub async fn login(
        &self,
        username: &str,
        password: &str,
    ) -> Result<String, Box<dyn std::error::Error>> {
        *self.sync_status.lock().unwrap() = SyncStatus::Authenticating;

        let auth = sync_login(username, password, None, self.http_client.clone()).await?;

        Ok(auth.hkey)
    }

    pub async fn sync_collection(
        &self,
        collection: &mut Collection,
        hkey: &str,
    ) -> Result<SyncResult, Box<dyn std::error::Error>> {
        let auth = SyncAuth {
            hkey: hkey.to_string(),
            endpoint: None,
            io_timeout_secs: Some(60),
        };

        *self.sync_status.lock().unwrap() = SyncStatus::CheckingStatus;

        let sync_required = collection.sync_status_offline()?;

        let mut result = SyncResult {
            success: true,
            message: String::new(),
            server_message: None,
        };

        match sync_required {
            sync_status_response::Required::NoChanges => {
                result.message = "No changes to sync".into();
                *self.sync_status.lock().unwrap() = SyncStatus::Complete {
                    message: result.message.clone(),
                };
            }
            sync_status_response::Required::NormalSync => {
                *self.sync_status.lock().unwrap() = SyncStatus::SyncingCollection {
                    progress: "Syncing collection...".into(),
                };

                let sync_output: SyncOutput = collection
                    .normal_sync(auth.clone(), self.http_client.clone())
                    .await?;

                result.message = "Collection synced successfully".into();
                result.server_message = Some(sync_output.server_message);

                *self.sync_status.lock().unwrap() = SyncStatus::Complete {
                    message: result.message.clone(),
                };
            }
            sync_status_response::Required::FullSync => {
                result.success = false;
                result.message = "Full sync required. Please sync via desktop Anki first.".into();
                *self.sync_status.lock().unwrap() = SyncStatus::Error {
                    message: result.message.clone(),
                };
            }
        }

        Ok(result)
    }

    pub async fn sync_media(
        &self,
        _collection: &mut Collection,
        _hkey: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        Err("Media sync not yet implemented - requires progress handler refactoring".into())
    }

    pub async fn full_sync(
        &self,
        _collection: &mut Collection,
        _hkey: &str,
        _upload: bool,
    ) -> Result<SyncResult, Box<dyn std::error::Error>> {
        Err("Full sync not yet implemented - requires Collection ownership transfer".into())
    }
}

impl Default for SyncManager {
    fn default() -> Self {
        Self::new()
    }
}
