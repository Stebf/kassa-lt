use chrono::Utc;
use uuid::Uuid;
use std::path::PathBuf;

use r2d2::{ManageConnection, Pool};

use crate::models::{CartItem, Order, OrderItem, Product};
use crate::logic;

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

pub fn get_products_with_pool(pool: &DbPool) -> Result<Vec<Product>, String> {
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

pub fn add_product_with_pool(pool: &DbPool, name: String, price: f64) -> Result<Product, String> {
    let normalized_name = logic::normalize_product_name(&name)?;
    logic::validate_price(price)?;

    let conn = pool.get().map_err(|e| e.to_string())?;
    let price_cents = logic::price_to_cents(price);

    conn.execute(
        "INSERT INTO products (name, price_cents) VALUES (?1, ?2)",
        rusqlite::params![normalized_name, price_cents],
    ).map_err(|e| e.to_string())?;

    let id = conn.last_insert_rowid() as i32;

    Ok(Product {
        id,
        name: normalized_name,
        price,
    })
}

pub fn checkout_with_pool(pool: &DbPool, items: Vec<CartItem>, payment_method: String) -> Result<Order, String> {
    if items.is_empty() {
        return Err("Cart is empty".to_string());
    }

    let mut conn = pool.get().map_err(|e| e.to_string())?;
    let tx = conn.transaction().map_err(|e| e.to_string())?;

    let order_uuid = Uuid::new_v4().to_string();
    let created_at = Utc::now().to_rfc3339();

    let total_cents = logic::cart_total_cents(&items);

    tx.execute(
        "INSERT INTO orders (uuid, created_at, total_cents, payment_method) VALUES (?1, ?2, ?3, ?4)",
        rusqlite::params![&order_uuid, &created_at, total_cents, &payment_method],
    ).map_err(|e| e.to_string())?;

    let order_id = tx.last_insert_rowid();

    for item in &items {
        let price_cents = logic::price_to_cents(item.price);
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
        items: logic::order_items_from_cart(&items),
    })
}

pub fn get_orders_with_pool(pool: &DbPool) -> Result<Vec<Order>, String> {
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

pub fn get_product_with_pool(pool: &DbPool, id: i32) -> Result<Product, String> {
    let conn = pool.get().map_err(|e| e.to_string())?;

    let mut stmt = conn.prepare("SELECT id, name, price_cents FROM products WHERE id = ?1")
        .map_err(|e| e.to_string())?;

    let mut rows = stmt.query(rusqlite::params![id]).map_err(|e| e.to_string())?;

    if let Some(row) = rows.next().map_err(|e| e.to_string())? {
        let id: i32 = row.get::<_, i32>(0).map_err(|e| e.to_string())?;
        let name: String = row.get::<_, String>(1).map_err(|e| e.to_string())?;
        let price_cents: i32 = row.get::<_, i32>(2).map_err(|e| e.to_string())?;

        let product = Product {
            id,
            name,
            price: price_cents as f64 / 100.0,
        };

        Ok(product)
    } else {
        Err("Product not found".to_string())
    }
}

pub fn update_product_with_pool(pool: &DbPool, id: i32, name: Option<String>, price: Option<f64>) -> Result<Product, String> {
    let conn = pool.get().map_err(|e| e.to_string())?;

    // Fetch current product to get existing values
    let current = conn.query_row(
        "SELECT id, name, price_cents FROM products WHERE id = ?1",
        rusqlite::params![id],
        |row| {
            Ok((
                row.get::<_, i32>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, i32>(2)?,
            ))
        },
    ).map_err(|e| e.to_string())?;

    // Use provided values or existing ones
    let final_name = if let Some(new_name) = name {
        logic::normalize_product_name(&new_name)?
    } else {
        current.1
    };

    let final_price = if let Some(new_price) = price {
        logic::validate_price(new_price)?;
        new_price
    } else {
        current.2 as f64 / 100.0
    };

    let price_cents = logic::price_to_cents(final_price);

    let affected = conn.execute(
        "UPDATE products SET name = ?1, price_cents = ?2 WHERE id = ?3",
        rusqlite::params![final_name, price_cents, id],
    ).map_err(|e| e.to_string())?;

    if affected == 0 {
        return Err("Product not found".to_string());
    }

    Ok(Product { id, name: final_name, price: final_price })
}

pub fn delete_product_with_pool(pool: &DbPool, id: i32) -> Result<(), String> {
    let conn = pool.get().map_err(|e| e.to_string())?;

    let affected = conn.execute(
        "DELETE FROM products WHERE id = ?1",
        rusqlite::params![id],
    ).map_err(|e| e.to_string())?;

    if affected == 0 {
        return Err("Product not found".to_string());
    }

    Ok(())
}

#[tauri::command]
pub fn get_products(pool: tauri::State<'_, DbPool>) -> Result<Vec<Product>, String> {
    get_products_with_pool(pool.inner())
}

#[tauri::command]
pub fn add_product(pool: tauri::State<'_, DbPool>, name: String, price: f64) -> Result<Product, String> {
    add_product_with_pool(pool.inner(), name, price)
}

#[tauri::command]
pub fn checkout(pool: tauri::State<'_, DbPool>, items: Vec<CartItem>, payment_method: String) -> Result<Order, String> {
    checkout_with_pool(pool.inner(), items, payment_method)
}

#[tauri::command]
pub fn get_orders(pool: tauri::State<'_, DbPool>) -> Result<Vec<Order>, String> {
    get_orders_with_pool(pool.inner())
}

#[tauri::command]
pub fn get_product(pool: tauri::State<'_, DbPool>, id: i32) -> Result<Product, String> {
    get_product_with_pool(pool.inner(), id)
}

#[tauri::command]
pub fn update_product(pool: tauri::State<'_, DbPool>, id: i32, name: Option<String>, price: Option<f64>) -> Result<Product, String> {
    update_product_with_pool(pool.inner(), id, name, price)
}

#[tauri::command]
pub fn delete_product(pool: tauri::State<'_, DbPool>, id: i32) -> Result<(), String> {
    delete_product_with_pool(pool.inner(), id)
}
