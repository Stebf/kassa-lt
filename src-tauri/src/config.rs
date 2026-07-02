use std::sync::Arc;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum WebDavAuthMethod {
    Basic,
    Digest,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum BackupProtocol {
    WebDAV,
    HttpPut,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SyncWorkerConfig {
    pub enabled: bool,
    pub central_api_base_url: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct BackupWorkerConfig {
    pub protocol: BackupProtocol,
    pub webdav_url: String,
    pub username: String,
    pub password: String,
    pub auth_method: WebDavAuthMethod,
    pub enabled: bool,
}

pub fn get_backup_config<R: tauri::Runtime>(
    store: &Arc<tauri_plugin_store::Store<R>>,
) -> Option<BackupWorkerConfig> {
    store
        .get("backupConfig")
        .and_then(|v| serde_json::from_value::<BackupWorkerConfig>(v).ok())
}

pub fn set_backup_config<R: tauri::Runtime>(
    store: &Arc<tauri_plugin_store::Store<R>>,
    config: &BackupWorkerConfig,
) {
    let v = serde_json::to_value(config).expect("failed to serialize backupconfig");
    store.set("backupConfig", v);
}

pub fn default_backup_config() -> BackupWorkerConfig {
    BackupWorkerConfig {
        protocol: BackupProtocol::WebDAV,
        webdav_url: "http://localhost:8080".to_string(),
        username: "alice".to_string(),
        password: "secret1234".to_string(),
        auth_method: WebDavAuthMethod::Digest,
        enabled: true,
    }
}

pub fn get_sync_config<R: tauri::Runtime>(
    store: &Arc<tauri_plugin_store::Store<R>>,
) -> Option<SyncWorkerConfig> {
    store
        .get("syncConfig")
        .and_then(|v| serde_json::from_value::<SyncWorkerConfig>(v).ok())
}

pub fn set_sync_config<R: tauri::Runtime>(
    store: &Arc<tauri_plugin_store::Store<R>>,
    config: &SyncWorkerConfig,
) {
    let v = serde_json::to_value(config).expect("failed to serialize syncconfig");
    store.set("syncConfig", v);
}

pub fn default_sync_config() -> SyncWorkerConfig {
    SyncWorkerConfig {
        enabled: false,
        central_api_base_url: "http://localhost:8080/api/v1".to_string(),
    }
}

pub fn init_sync_config<R: tauri::Runtime>(store: &Arc<tauri_plugin_store::Store<R>>) {
    if get_sync_config(store).is_none() {
        set_sync_config(store, &default_sync_config());
    }
}

pub fn init_backup_config<R: tauri::Runtime>(store: &Arc<tauri_plugin_store::Store<R>>) {
    if get_backup_config(store).is_none() {
        set_backup_config(store, &default_backup_config());
    }
}
