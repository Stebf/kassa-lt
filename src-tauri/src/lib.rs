pub mod backup_worker;
pub mod commands;
pub mod config;
pub mod database;
pub mod exports;
pub mod logic;
pub mod models;

use crate::database::{init_db_with_pool, DbPool, SqliteManager};
use log::{error, info};
use r2d2::Pool;
use tauri::Manager;
use tauri_plugin_store::StoreExt;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .setup(|app| {
            let app_handle = app.handle();
            if cfg!(debug_assertions) {
                app_handle.plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }

            // Create DB path and connection pool
            let app_data_dir = setup_app_data_dir(app_handle)?;
            let db_path = app_data_dir.join("kassalt.db");
            info!("Using database at: {:?}", db_path);

            let store = app.store("settings.json")?;

            let manager = SqliteManager::file(db_path);
            let pool: DbPool = Pool::new(manager).map_err(|e| e.to_string())?;

            // Initialize database schema using the pool
            if let Err(e) = init_db_with_pool(&pool) {
                error!("Failed to initialize database: {}", e);
            }

            let instance_id = match store.get("instance_id") {
                Some(serde_json::Value::String(id)) => id,
                Some(_) | None => {
                    let id = uuid::Uuid::new_v4().to_string();
                    store.set("instance_id", serde_json::Value::String(id.clone()));
                    id
                }
            };

            config::init_backup_config(&store.clone());

            let config = config::get_backup_config(&store)
                .unwrap_or_else(|| config::default_backup_config());

            let (tx, rx) = tokio::sync::watch::channel(config);

            let worker = backup_worker::BackupWorker::new(
                pool.clone(),
                instance_id,
                app_handle.path().temp_dir().expect("no temp dir"),
                rx,
            );
            worker.start();

            // Make the pool available as managed state
            app.manage(pool);
            // Make the backup worker available to handlers in order to query its state
            app.manage(worker);
            app.manage(tx);

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_products,
            commands::add_product,
            commands::get_categories,
            commands::add_category,
            commands::update_category,
            commands::delete_category,
            commands::get_tabs,
            commands::add_tab,
            commands::update_tab,
            commands::delete_tab,
            commands::checkout,
            commands::get_orders,
            commands::get_product,
            commands::get_product_sales_count,
            commands::update_product,
            commands::delete_product,
            commands::get_backup_config,
            commands::set_backup_config,
            commands::get_backup_state,
            exports::export_orders_csv,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn setup_app_data_dir(app_handle: &tauri::AppHandle) -> Result<std::path::PathBuf, String> {
    let app_data_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| e.to_string())?;
    std::fs::create_dir_all(&app_data_dir).map_err(|e| e.to_string())?;
    Ok(app_data_dir)
}
