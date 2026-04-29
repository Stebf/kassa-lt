use chrono::Utc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use std::path::PathBuf;

use r2d2::{ManageConnection, Pool};

pub struct SqliteManager {
    pub path: PathBuf,
}

impl SqliteManager {
    pub fn file(path: PathBuf) -> Self {
        SqliteManager { path }
    }
}

impl ManageConnection for SqliteManager {
    type Connection = rusqlite::Connection;
    type Error = rusqlite::Error;

    fn connect(&self) -> Result<Self::Connection, Self::Error> {
        rusqlite::Connection::open(&self.path)
    }

    fn is_valid(&self, conn: &mut Self::Connection) -> Result<(), Self::Error> {
        conn.execute_batch("SELECT 1").map(|_| ())
    }

    fn has_broken(&self, _conn: &mut Self::Connection) -> bool {
        false
    }
}

pub type DbPool = Pool<SqliteManager>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Product {
    pub id: i32,
    pub name: String,
    pub price: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CartItem {
    pub id: i32,
    pub name: String,
    pub price: f64,
    pub quantity: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Order {
    pub id: i64,
    pub uuid: String,
    pub created_at: String,
    pub total: f64,
    pub payment_method: String,
    pub items: Vec<OrderItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderItem {
    pub name: String,
    pub price: f64,
    pub quantity: i32,
}

pub fn init_db_with_pool(pool: &DbPool) -> Result<(), String> {
    let conn = pool.get().map_err(|e| e.to_string())?;

    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS products (
            id INTEGER PRIMARY KEY,
            name TEXT NOT NULL,
            price_cents INTEGER NOT NULL
        );
        
        CREATE TABLE IF NOT EXISTS orders (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            uuid TEXT NOT NULL UNIQUE,
            created_at TEXT NOT NULL,
            total_cents INTEGER NOT NULL,
            payment_method TEXT NOT NULL
        );
        
        CREATE TABLE IF NOT EXISTS order_items (
            order_id INTEGER NOT NULL,
            name TEXT NOT NULL,
            price_cents INTEGER NOT NULL,
            quantity INTEGER NOT NULL,
            FOREIGN KEY(order_id) REFERENCES orders(id)
        );"
    )
    .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub fn get_products(pool: tauri::State<'_, DbPool>) -> Result<Vec<Product>, String> {
    let conn = pool.get().map_err(|e| e.to_string())?;

    let mut stmt = conn.prepare("SELECT id, name, price_cents FROM products ORDER BY id")
        .map_err(|e| e.to_string())?;

    let products = stmt.query_map([], |row| {
        Ok(Product {
            id: row.get(0)?,
            name: row.get(1)?,
            price: row.get::<_, i32>(2)? as f64 / 100.0,
        })
    }).map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(products)
}

#[tauri::command]
pub fn add_product(pool: tauri::State<'_, DbPool>, name: String, price: f64) -> Result<Product, String> {
    if name.trim().is_empty() {
        return Err("Product name cannot be empty".to_string());
    }
    
    if price <= 0.0 {
        return Err("Price must be greater than 0".to_string());
    }
    
    let conn = pool.get().map_err(|e| e.to_string())?;
    let price_cents = (price * 100.0).round() as i32;
    
    conn.execute(
        "INSERT INTO products (name, price_cents) VALUES (?1, ?2)",
        rusqlite::params![name.trim(), price_cents],
    ).map_err(|e| e.to_string())?;
    
    let id = conn.last_insert_rowid() as i32;
    
    Ok(Product {
        id,
        name,
        price,
    })
}

#[tauri::command]
pub fn checkout(pool: tauri::State<'_, DbPool>, items: Vec<CartItem>, payment_method: String) -> Result<Order, String> {
    if items.is_empty() {
        return Err("Cart is empty".to_string());
    }
    
    let mut conn = pool.get().map_err(|e| e.to_string())?;
    let tx = conn.transaction().map_err(|e| e.to_string())?;

    let order_uuid = Uuid::new_v4().to_string();
    let created_at = Utc::now().to_rfc3339();
    
    let total_cents: i32 = items.iter()
        .map(|item| ((item.price * item.quantity as f64) * 100.0).round() as i32)
        .sum();
    
    // Insert order
    tx.execute(
        "INSERT INTO orders (uuid, created_at, total_cents, payment_method) VALUES (?1, ?2, ?3, ?4)",
        rusqlite::params![&order_uuid, &created_at, total_cents, &payment_method],
    ).map_err(|e| e.to_string())?;

    let order_id = tx.last_insert_rowid();
    
    // Insert order items
    for item in &items {
        let price_cents = (item.price * 100.0).round() as i32;
        tx.execute(
            "INSERT INTO order_items (order_id, name, price_cents, quantity) VALUES (?1, ?2, ?3, ?4)",
            rusqlite::params![&order_id, &item.name, price_cents, item.quantity],
        ).map_err(|e| e.to_string())?;
    }
    
    tx.commit().map_err(|e| e.to_string())?;

    Ok(Order {
        id: order_id,
        uuid: order_uuid,
        created_at,
        total: total_cents as f64 / 100.0,
        payment_method,
        items: items.iter().map(|i| OrderItem {
            name: i.name.clone(),
            price: i.price,
            quantity: i.quantity,
        }).collect(),
    })
}

#[tauri::command]
pub fn get_orders(pool: tauri::State<'_, DbPool>) -> Result<Vec<Order>, String> {
    let conn = pool.get().map_err(|e| e.to_string())?;

    let mut stmt = conn.prepare(
        "SELECT id, uuid, created_at, total_cents, payment_method FROM orders ORDER BY id DESC"
    ).map_err(|e| e.to_string())?;

    let orders = stmt.query_map([], |row| {
        let order_id: i64 = row.get(0)?;
        let order_uuid: String = row.get(1)?;
        let created_at: String = row.get(2)?;
        let total_cents: i32 = row.get(3)?;
        let payment_method: String = row.get(4)?;

        Ok((order_id, order_uuid, created_at, total_cents, payment_method))
    }).map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    let mut result = Vec::new();
    for (order_id, order_uuid, created_at, total_cents, payment_method) in orders {
        let mut item_stmt = conn.prepare(
            "SELECT name, price_cents, quantity FROM order_items WHERE order_id = ?1"
        ).map_err(|e| e.to_string())?;

        let items = item_stmt.query_map(rusqlite::params![order_id], |row| {
            Ok(OrderItem {
                name: row.get(0)?,
                price: row.get::<_, i32>(1)? as f64 / 100.0,
                quantity: row.get(2)?,
            })
        }).map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;

        result.push(Order {
            id: order_id,
            uuid: order_uuid,
            created_at,
            total: total_cents as f64 / 100.0,
            payment_method,
            items,
        });
    }

    Ok(result)
}
