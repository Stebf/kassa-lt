use tauri_plugin_store::StoreExt;

use log::error;

use crate::backup_worker;
use crate::config;
use crate::database::{
    add_category_with_pool, add_product_with_pool, add_tab_with_pool, checkout_with_pool,
    delete_category_with_pool, delete_product_with_pool, delete_tab_with_pool,
    get_categories_with_pool, get_orders_with_pool, get_product_sales_count_with_pool,
    get_product_with_pool, get_products_with_pool, get_tabs_with_pool, update_category_with_pool,
    update_product_with_pool, update_tab_with_pool, DbPool,
};
use crate::models::{CartItem, Order, Product, ProductSalesCount, Tab};
use crate::sync::{OutboxPublisher, SyncRouter};

#[tauri::command]
pub fn get_products(pool: tauri::State<'_, DbPool>) -> Result<Vec<Product>, String> {
    get_products_with_pool(pool.inner())
}

#[tauri::command]
pub fn add_product(
    pool: tauri::State<'_, DbPool>,
    name: String,
    price: f64,
    category: Option<String>,
    tab_ids: Option<Vec<i32>>,
    sales_limit: Option<i32>,
) -> Result<Product, String> {
    add_product_with_pool(pool.inner(), name, price, category, tab_ids, sales_limit)
}

#[tauri::command]
pub fn get_categories(
    pool: tauri::State<'_, DbPool>,
) -> Result<Vec<crate::models::Category>, String> {
    get_categories_with_pool(pool.inner())
}

#[tauri::command]
pub fn checkout(
    pool: tauri::State<'_, DbPool>,
    sync_router: tauri::State<'_, SyncRouter>,
    items: Vec<CartItem>,
    payment_method: String,
    comment: String,
) -> Result<Order, String> {
    let order = checkout_with_pool(pool.inner(), items, payment_method, comment)?;

    let publisher = sync_router.publisher();
    if let Err(err) = publisher.publish_sale_created(&order) {
        error!("sync checkout event was not queued: {}", err);
    }

    Ok(order)
}

#[tauri::command]
pub fn get_orders(pool: tauri::State<'_, DbPool>) -> Result<Vec<Order>, String> {
    get_orders_with_pool(pool.inner())
}

#[tauri::command]
pub fn get_product_sales_count(
    pool: tauri::State<'_, DbPool>,
) -> Result<Vec<ProductSalesCount>, String> {
    get_product_sales_count_with_pool(pool.inner())
}

#[tauri::command]
pub fn get_product(pool: tauri::State<'_, DbPool>, id: i32) -> Result<Product, String> {
    get_product_with_pool(pool.inner(), id)
}

#[tauri::command]
pub fn update_product(
    pool: tauri::State<'_, DbPool>,
    id: i32,
    name: Option<String>,
    price: Option<f64>,
    category: Option<String>,
    tab_ids: Option<Vec<i32>>,
    sales_limit: Option<i32>,
    sales_limit_changed: Option<bool>,
) -> Result<Product, String> {
    let sales_limit_param: Option<Option<i32>> = match sales_limit_changed {
        Some(true) => Some(sales_limit),
        _ => None,
    };

    update_product_with_pool(pool.inner(), id, name, price, category, tab_ids, sales_limit_param)
}

#[tauri::command]
pub fn get_tabs(pool: tauri::State<'_, DbPool>) -> Result<Vec<Tab>, String> {
    get_tabs_with_pool(pool.inner())
}

#[tauri::command]
pub fn add_tab(pool: tauri::State<'_, DbPool>, name: String) -> Result<Tab, String> {
    add_tab_with_pool(pool.inner(), name)
}

#[tauri::command]
pub fn update_tab(pool: tauri::State<'_, DbPool>, id: i32, name: String) -> Result<Tab, String> {
    update_tab_with_pool(pool.inner(), id, name)
}

#[tauri::command]
pub fn delete_tab(pool: tauri::State<'_, DbPool>, id: i32) -> Result<(), String> {
    delete_tab_with_pool(pool.inner(), id)
}

#[tauri::command]
pub fn delete_product(pool: tauri::State<'_, DbPool>, id: i32) -> Result<(), String> {
    delete_product_with_pool(pool.inner(), id)
}

#[tauri::command]
pub fn add_category(
    pool: tauri::State<'_, DbPool>,
    name: String,
) -> Result<crate::models::Category, String> {
    add_category_with_pool(pool.inner(), name)
}

#[tauri::command]
pub fn update_category(
    pool: tauri::State<'_, DbPool>,
    id: i32,
    name: String,
) -> Result<crate::models::Category, String> {
    update_category_with_pool(pool.inner(), id, name)
}

#[tauri::command]
pub fn delete_category(pool: tauri::State<'_, DbPool>, id: i32) -> Result<(), String> {
    delete_category_with_pool(pool.inner(), id)
}

#[tauri::command]
pub fn get_backup_config(
    app_handle: tauri::AppHandle,
) -> Result<config::BackupWorkerConfig, String> {
    let store = app_handle
        .store("settings.json")
        .map_err(|e| e.to_string())?;
    Ok(config::get_backup_config(&store).unwrap_or(config::default_backup_config()))
}

#[tauri::command]
pub fn get_sync_config(app_handle: tauri::AppHandle) -> Result<config::SyncWorkerConfig, String> {
    let store = app_handle
        .store("settings.json")
        .map_err(|e| e.to_string())?;
    Ok(config::get_sync_config(&store).unwrap_or(config::default_sync_config()))
}

#[tauri::command]
pub fn set_backup_config(
    // store: tauri::State<'_, Arc<tauri_plugin_store::Store<R>>>,
    app_handle: tauri::AppHandle,
    backup_config_tx: tauri::State<'_, tokio::sync::watch::Sender<config::BackupWorkerConfig>>,
    config: config::BackupWorkerConfig,
) -> Result<(), String> {
    let store = app_handle
        .store("settings.json")
        .map_err(|e| e.to_string())?;
    config::set_backup_config(&store, &config);
    let _ = backup_config_tx.send(config);
    Ok(())
}

#[tauri::command]
pub fn set_sync_config(
    app_handle: tauri::AppHandle,
    sync_router: tauri::State<'_, SyncRouter>,
    sync_outbox: tauri::State<'_, std::sync::Arc<OutboxPublisher>>,
    config: config::SyncWorkerConfig,
) -> Result<(), String> {
    let store = app_handle
        .store("settings.json")
        .map_err(|e| e.to_string())?;
    config::set_sync_config(&store, &config);
    sync_router.set_enabled(config.enabled, sync_outbox.inner().clone());
    Ok(())
}

#[tauri::command]
pub fn get_backup_state(
    backup_worker: tauri::State<'_, backup_worker::BackupWorker>,
) -> Result<backup_worker::BackupState, String> {
    Ok(backup_worker.get_last_state())
}

#[tauri::command]
pub async fn run_backup_now(
    backup_worker: tauri::State<'_, backup_worker::BackupWorker>,
) -> Result<backup_worker::BackupState, String> {
    if !backup_worker.is_enabled() {
        return Err("Backup module is disabled".to_string());
    }

    Ok(backup_worker.run_backup_now().await)
}
