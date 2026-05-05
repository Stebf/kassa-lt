pub mod database;
pub mod commands;
pub mod exports;
pub mod logic;
pub mod models;
pub mod config;

use crate::database::{DbPool, SqliteManager, init_db_with_pool};
use r2d2::Pool;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
    .plugin(tauri_plugin_fs::init())
    .setup(|app| {
    // Create DB path and connection pool
        let app_handle = app.handle();
        let app_data_dir = setup_app_data_dir(app_handle)?;
        config::init_app_config(&app_data_dir)?;
        let db_path = app_data_dir.join("kassalt.db");
        println!("Using database at: {:?}", db_path);

        let manager = SqliteManager::file(db_path);
        let pool: DbPool = Pool::new(manager).map_err(|e| e.to_string())?;

        // Initialize database schema using the pool
        if let Err(e) = init_db_with_pool(&pool) {
            eprintln!("Failed to initialize database: {}", e);
        }

        // Make the pool available as managed state
        app.manage(pool);

        if cfg!(debug_assertions) {
            app_handle.plugin(
                tauri_plugin_log::Builder::default()
                    .level(log::LevelFilter::Info)
                    .build(),
            )?;
        }
        Ok(())
    })
    .invoke_handler(tauri::generate_handler![
        commands::get_products,
        commands::add_product,
        commands::get_categories,
        commands::add_category,
        commands::update_category,
        commands::delete_category,
        commands::checkout,
        commands::get_orders,
        commands::get_product,
        commands::update_product,
        commands::delete_product,
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