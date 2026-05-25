use serde::Serialize;
use std::{path::PathBuf, time::Duration};
use tokio::{fs, time};

use log::{error, info, warn};
use reqwest_dav::{Auth, ClientBuilder};

use crate::{
    config,
    database::{self, DbPool},
};

#[derive(Clone, Serialize)]
#[serde(tag = "type")]
pub enum BackupState {
    NotRunYet,
    Successful {
        timestamp: chrono::DateTime<chrono::Utc>,
    },
    Failed {
        timestamp: chrono::DateTime<chrono::Utc>,
        error: String,
    },
}

#[derive(Clone)]
pub struct BackupWorker {
    pool: DbPool,
    instance_id: String,
    temp_dir: PathBuf,
    last_state_tx: tokio::sync::watch::Sender<BackupState>,
    last_state_rx: tokio::sync::watch::Receiver<BackupState>,
    config_rx: tokio::sync::watch::Receiver<config::BackupWorkerConfig>,
}

impl BackupWorker {
    pub fn new(
        pool: DbPool,
        instance_id: String,
        temp_dir: PathBuf,
        config_rx: tokio::sync::watch::Receiver<config::BackupWorkerConfig>,
    ) -> Self {
        let (tx, rx) = tokio::sync::watch::channel(BackupState::NotRunYet);
        BackupWorker {
            pool,
            instance_id,
            temp_dir,
            last_state_rx: rx,
            last_state_tx: tx,
            config_rx,
        }
    }

    pub fn start(&self) {
        let this = self.clone();

        info!("backup_worker: starting; instance_id={}", this.instance_id);

        tauri::async_runtime::spawn(async move {
            match start_backup_worker(
                this.pool,
                this.instance_id,
                this.temp_dir,
                this.config_rx,
                this.last_state_tx,
            )
            .await
            {
                Ok(_) => {
                    info!("backup_worker: finished worker");
                }
                Err(e) => {
                    error!("backup_worker: failed: {}", e);
                }
            }
        });
    }

    pub async fn run_backup_now(&self) -> BackupState {
        let config = self.config_rx.borrow().clone();
        run_backup_and_record_state(
            self.pool.clone(),
            self.instance_id.clone(),
            self.temp_dir.clone(),
            config,
            self.last_state_tx.clone(),
        )
        .await
    }

    pub fn is_enabled(&self) -> bool {
        self.config_rx.borrow().enabled
    }

    pub fn get_last_state(&self) -> BackupState {
        self.last_state_rx.borrow().clone()
    }
}

async fn run_backup_once(
    pool: DbPool,
    instance_id: &str,
    temp_dir: &PathBuf,
    config: &config::BackupWorkerConfig,
) -> Result<(), String> {
    let mut backup_path = temp_dir.clone();
    let current_time = chrono::Utc::now();
    let current_time_s = current_time
        .to_rfc3339_opts(chrono::SecondsFormat::Secs, false)
        // Remove colon since WebDAV does not allow these as filenames (because Windows)
        .replace(":", "");
    let database_filename = format!("{}-{}.sqlite", instance_id, current_time_s);
    backup_path.push(database_filename.clone());
    info!("Backing up to: {:?}", backup_path);

    {
        let backup_path = backup_path.clone();
        let pool = pool.clone();
        tokio::task::spawn_blocking(move || {
            database::backup_with_pool(&pool, &backup_path).map_err(|e| e.to_string())
        })
        .await
        .map_err(|e| e.to_string())??;
    }

    let client = ClientBuilder::new()
        .set_host(config.webdav_url.clone())
        .set_auth(match config.auth_method {
            config::WebDavAuthMethod::Basic => {
                Auth::Basic(config.username.clone(), config.password.clone())
            }
            config::WebDavAuthMethod::Digest => {
                Auth::Digest(config.username.clone(), config.password.clone())
            }
        })
        .build()
        .map_err(|e| e.to_string())?;

    let content = fs::read(backup_path).await.map_err(|e| e.to_string())?;
    info!(
        "backup_worker: uploading; filename={} bytes={}",
        database_filename,
        content.len()
    );

    if let Err(e) = client
        .put(&format!("/{}", database_filename), content)
        .await
    {
        error!("backup_worker: upload failed; url={} err={}", config.webdav_url, e);
        return Err(e.to_string());
    }
    info!("backup_worker: upload succeeded; filename={}", database_filename);

    Ok(())
}

async fn run_backup_and_record_state(
    pool: DbPool,
    instance_id: String,
    temp_dir: PathBuf,
    config: config::BackupWorkerConfig,
    backup_state_tx: tokio::sync::watch::Sender<BackupState>,
) -> BackupState {
    let now = chrono::Utc::now();
    let new_state = match run_backup_once(pool, &instance_id, &temp_dir, &config).await {
        Ok(()) => {
            info!("successfully backed up database");
            BackupState::Successful { timestamp: now }
        }
        Err(e) => {
            error!("failed to backup database due to: {}", e);
            BackupState::Failed {
                timestamp: now,
                error: e,
            }
        }
    };

    if let Err(e) = backup_state_tx.send(new_state.clone()) {
        warn!("Failed to update backup state: {}", e)
    }

    new_state
}

async fn start_backup_worker(
    pool: DbPool,
    instance_id: String,
    temp_dir: PathBuf,
    mut config_rx: tokio::sync::watch::Receiver<config::BackupWorkerConfig>,
    backup_state_tx: tokio::sync::watch::Sender<BackupState>,
) -> Result<(), String> {
    let mut interval = time::interval(Duration::from_mins(30));
    loop {
        let pool = pool.clone();

        tokio::select! {
            config = config_rx.changed() => {
                config_rx.borrow_and_update();
                match config {
                    Ok(()) => info!("backup worker config changed"),
                    Err(e) => warn!("backup worker config watch channel closed: {}", e),
                }
                // Resetting interval to trigger backup immediately on config change.
                interval = time::interval(Duration::from_mins(30));
            }
            _ = interval.tick() => {
                let config = config_rx.borrow_and_update().clone();
                info!("backup worker tick; enabled={} url={}", config.enabled, config.webdav_url);
                if config.enabled {
                    let _ = run_backup_and_record_state(
                        pool,
                        instance_id.clone(),
                        temp_dir.clone(),
                        config,
                        backup_state_tx.clone(),
                    )
                    .await;
                } else {
                    info!("backup worker is disabled; skipping run");
                }
            }
        }
    }
}
