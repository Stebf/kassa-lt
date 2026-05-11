use log::info;
use once_cell::sync::OnceCell;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::RwLock;

pub static APP_CONFIG: OnceCell<RwLock<AppConfig>> = OnceCell::new();

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub enable_custom_db_path: bool,
    pub custom_db_path: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        AppConfig {
            enable_custom_db_path: false,
            custom_db_path: String::new(),
        }
    }
}

impl AppConfig {
    pub fn load_from_file(path: &PathBuf) -> Result<Self, String> {
        let config_str = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
        serde_yaml::from_str(&config_str).map_err(|e| e.to_string())
    }

    pub fn save_to_file(&self, path: &PathBuf) -> Result<(), String> {
        let config_str = serde_yaml::to_string(self).map_err(|e| e.to_string())?;
        std::fs::write(path, config_str).map_err(|e| e.to_string())
    }
}

pub fn init_app_config(app_data_dir: &std::path::PathBuf) -> Result<(), String> {
    let config_path = app_data_dir.clone().join("config.yaml");

    let config = match AppConfig::load_from_file(&config_path) {
        Ok(c) => {
            info!("Loaded config from file: {:?}", config_path);
            c
        }
        Err(_) => {
            let default = AppConfig::default();
            default.save_to_file(&config_path)?;
            info!("Created default config file: {:?}", config_path);
            default
        }
    };

    APP_CONFIG
        .set(RwLock::new(config))
        .map_err(|_| "Config already initialized")?;
    Ok(())
}

pub fn save_config(config: &AppConfig, config_path: &PathBuf) -> Result<(), String> {
    config.save_to_file(config_path)
}

pub fn get_config() -> std::sync::LockResult<std::sync::RwLockReadGuard<'static, AppConfig>> {
    APP_CONFIG.get().expect("Config not initialized!").read()
}

pub fn get_config_mut() -> std::sync::LockResult<std::sync::RwLockWriteGuard<'static, AppConfig>> {
    APP_CONFIG.get().expect("Config not initialized!").write()
}
