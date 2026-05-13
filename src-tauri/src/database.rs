use chrono::Utc;
use rusqlite::params;
use rusqlite::OptionalExtension;
use std::collections::HashMap;
use std::path::PathBuf;
use uuid::Uuid;

use r2d2::{ManageConnection, Pool};

use crate::logic;
use crate::models::ProductSalesCount;
use crate::models::{CartItem, Order, OrderItem, Product};

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
        "CREATE TABLE IF NOT EXISTS categories (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL UNIQUE
        );

        CREATE TABLE IF NOT EXISTS products (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            price_cents INTEGER NOT NULL,
            category_id INTEGER NOT NULL,
            FOREIGN KEY(category_id) REFERENCES categories(id)
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
        );

        CREATE TABLE IF NOT EXISTS audit_log (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            action TEXT NOT NULL,
            table_name TEXT NOT NULL,
            record_id TEXT,
            old_values TEXT,
            new_values TEXT,
            timestamp DATETIME DEFAULT CURRENT_TIMESTAMP
        );",
    )
    .map_err(|e| e.to_string())?;

    // Ensure default category exists
    conn.execute(
        "INSERT OR IGNORE INTO categories (id, name) VALUES (1, 'Default')",
        [],
    )
    .map_err(|e| e.to_string())?;

    Ok(())
}

pub fn get_products_with_pool(pool: &DbPool) -> Result<Vec<Product>, String> {
    let conn = pool.get().map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare(
            "SELECT p.id, p.name, p.price_cents, c.id, c.name
         FROM products p
         JOIN categories c ON p.category_id = c.id
         ORDER BY p.id",
        )
        .map_err(|e| e.to_string())?;

    let products = stmt
        .query_map([], |row| {
            Ok(Product {
                id: row.get(0)?,
                name: row.get(1)?,
                price: row.get::<_, i32>(2)? as f64 / 100.0,
                category_id: row.get(3)?,
                category_name: row.get(4)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(products)
}

pub fn get_category_by_name(
    tx: &rusqlite::Transaction,
    category_name: &str,
) -> Result<Option<i32>, String> {
    let mut stmt = tx
        .prepare("SELECT id FROM categories WHERE name = ?1")
        .map_err(|e| e.to_string())?;

    let res = stmt
        .query_row(params![category_name], |row| row.get::<_, i32>(0))
        .optional()
        .map_err(|e| e.to_string())?;
    Ok(res)
}

pub fn create_category(tx: &rusqlite::Transaction, category_name: &str) -> Result<i32, String> {
    tx.execute(
        "INSERT INTO categories (name) VALUES (?1)",
        params![category_name],
    )
    .map_err(|e| e.to_string())?;
    Ok(tx.last_insert_rowid() as i32)
}

pub fn add_product_with_pool(
    pool: &DbPool,
    name: String,
    price: f64,
    category: Option<String>,
) -> Result<Product, String> {
    let normalized_name = logic::normalize_product_name(&name)?;
    logic::validate_price(price)?;

    let mut conn = pool.get().map_err(|e| e.to_string())?;
    let tx = conn.transaction().map_err(|e| e.to_string())?;
    let price_cents = logic::price_to_cents(price);

    let category_name = category.unwrap_or_else(|| "Default".to_string());
    let category_id = match get_category_by_name(&tx, &category_name)? {
        Some(id) => id,
        None => create_category(&tx, &category_name)?,
    };

    tx.execute(
        "INSERT INTO products (name, price_cents, category_id) VALUES (?1, ?2, ?3)",
        params![normalized_name, price_cents, category_id],
    )
    .map_err(|e| e.to_string())?;

    let id = tx.last_insert_rowid() as i32;

    tx.execute(
        "INSERT INTO audit_log (action, table_name, record_id, new_values) VALUES (?, ?, ?, ?)",
        params![
            "INSERT",
            "products",
            id,
            serde_json::to_string(&Product {
                id,
                name: normalized_name.clone(),
                price,
                category_id,
                category_name: category_name.clone()
            })
            .unwrap_or_else(|_| String::new())
        ],
    )
    .map_err(|e| e.to_string())?;

    tx.commit().map_err(|e| e.to_string())?;
    Ok(Product {
        id,
        name: normalized_name,
        price,
        category_id,
        category_name,
    })
}

pub fn checkout_with_pool(
    pool: &DbPool,
    items: Vec<CartItem>,
    payment_method: String,
) -> Result<Order, String> {
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
        params![&order_uuid, &created_at, total_cents, &payment_method],
    ).map_err(|e| e.to_string())?;

    let order_id = tx.last_insert_rowid();

    for item in &items {
        let price_cents = logic::price_to_cents(item.price);
        tx.execute(
            "INSERT INTO order_items (order_id, name, price_cents, quantity) VALUES (?1, ?2, ?3, ?4)",
            params![&order_id, &item.name, price_cents, item.quantity],
        ).map_err(|e| e.to_string())?;
    }

    tx.execute(
        "INSERT INTO audit_log (action, table_name, record_id, new_values) VALUES (?, ?, ?, ?)",
        params![
            "INSERT",
            "orders",
            order_uuid,
            serde_json::to_string(&Order {
                id: order_id,
                uuid: order_uuid.clone(),
                created_at: created_at.clone(),
                total: total_cents as f64 / 100.0,
                payment_method: payment_method.clone(),
                items: logic::order_items_from_cart(&items),
            })
            .unwrap_or_else(|_| String::new())
        ],
    )
    .map_err(|e| e.to_string())?;

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

    let mut stmt = conn
        .prepare(
            "SELECT id, uuid, created_at, total_cents, payment_method FROM orders ORDER BY id DESC",
        )
        .map_err(|e| e.to_string())?;

    let orders = stmt
        .query_map([], |row| {
            let order_id: i64 = row.get(0)?;
            let order_uuid: String = row.get(1)?;
            let created_at: String = row.get(2)?;
            let total_cents: i32 = row.get(3)?;
            let payment_method: String = row.get(4)?;

            Ok((
                order_id,
                order_uuid,
                created_at,
                total_cents,
                payment_method,
            ))
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    let mut result = Vec::new();
    for (order_id, order_uuid, created_at, total_cents, payment_method) in orders {
        let mut item_stmt = conn
            .prepare("SELECT name, price_cents, quantity FROM order_items WHERE order_id = ?1")
            .map_err(|e| e.to_string())?;

        let items = item_stmt
            .query_map(rusqlite::params![order_id], |row| {
                Ok(OrderItem {
                    name: row.get(0)?,
                    price: row.get::<_, i32>(1)? as f64 / 100.0,
                    quantity: row.get(2)?,
                })
            })
            .map_err(|e| e.to_string())?
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

pub fn get_product_sales_count_with_pool(pool: &DbPool) -> Result<Vec<ProductSalesCount>, String> {
    let result = get_orders_with_pool(pool)?
        .into_iter()
        .flat_map(|order| order.items.into_iter())
        .fold(HashMap::new(), |mut acc, item| {
            *acc.entry(item.name).or_insert(0) += item.quantity;
            acc
        })
        .into_iter()
        .map(|(product_name, count)| ProductSalesCount {
            product_name,
            count,
        })
        .collect();

    Ok(result)
}

pub fn get_product_with_pool(pool: &DbPool, id: i32) -> Result<Product, String> {
    let conn = pool.get().map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare(
            "SELECT p.id, p.name, p.price_cents, c.id, c.name
         FROM products p
         JOIN categories c ON p.category_id = c.id
         WHERE p.id = ?1",
        )
        .map_err(|e| e.to_string())?;

    let mut rows = stmt
        .query(rusqlite::params![id])
        .map_err(|e| e.to_string())?;

    if let Some(row) = rows.next().map_err(|e| e.to_string())? {
        let id: i32 = row.get::<_, i32>(0).map_err(|e| e.to_string())?;
        let name: String = row.get::<_, String>(1).map_err(|e| e.to_string())?;
        let price_cents: i32 = row.get::<_, i32>(2).map_err(|e| e.to_string())?;
        let category_id: i32 = row.get::<_, i32>(3).map_err(|e| e.to_string())?;
        let category_name: String = row.get::<_, String>(4).map_err(|e| e.to_string())?;

        let product = Product {
            id,
            name,
            price: price_cents as f64 / 100.0,
            category_id,
            category_name,
        };

        Ok(product)
    } else {
        Err("Product not found".to_string())
    }
}

pub fn update_product_with_pool(
    pool: &DbPool,
    id: i32,
    name: Option<String>,
    price: Option<f64>,
    category: Option<String>,
) -> Result<Product, String> {
    let mut conn = pool.get().map_err(|e| e.to_string())?;
    let tx = conn.transaction().map_err(|e| e.to_string())?;

    // Fetch current product to get existing values
    let current = tx
        .query_row(
            "SELECT p.id, p.name, p.price_cents, p.category_id, c.name
         FROM products p
         JOIN categories c ON p.category_id = c.id
         WHERE p.id = ?1",
            params![id],
            |row| {
                Ok((
                    row.get::<_, i32>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, i32>(2)?,
                    row.get::<_, i32>(3)?,
                    row.get::<_, String>(4)?,
                ))
            },
        )
        .map_err(|e| e.to_string())?;

    let old_values = Product {
        id: current.0,
        name: current.1.clone(),
        price: current.2 as f64 / 100.0,
        category_id: current.3,
        category_name: current.4.clone(),
    };

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

    let final_category_name = if let Some(new_category) = category {
        new_category
    } else {
        current.4
    };

    let final_category_id = match get_category_by_name(&tx, &final_category_name)? {
        Some(id) => id,
        None => create_category(&tx, &final_category_name)?,
    };

    let price_cents = logic::price_to_cents(final_price);

    let affected = tx
        .execute(
            "UPDATE products SET name = ?1, price_cents = ?2, category_id = ?3 WHERE id = ?4",
            params![final_name, price_cents, final_category_id, id],
        )
        .map_err(|e| e.to_string())?;

    tx.execute(
        "INSERT INTO audit_log (action, table_name, record_id, old_values, new_values) VALUES (?, ?, ?, ?, ?)",
        params![
            "UPDATE",
            "products",
            id.to_string(),
            serde_json::to_string(&old_values).unwrap_or_else(|_| String::new()),
            serde_json::to_string(&Product { id, name: final_name.clone(), price: final_price, category_id: final_category_id, category_name: final_category_name.clone() }).unwrap_or_else(|_| String::new())
        ],
    ).map_err(|e| e.to_string())?;

    tx.commit().map_err(|e| e.to_string())?;
    if affected == 0 {
        return Err("Product not found".to_string());
    }
    Ok(Product {
        id,
        name: final_name,
        price: final_price,
        category_id: final_category_id,
        category_name: final_category_name,
    })
}

pub fn delete_product_with_pool(pool: &DbPool, id: i32) -> Result<(), String> {
    let mut conn = pool.get().map_err(|e| e.to_string())?;
    let tx = conn.transaction().map_err(|e| e.to_string())?;

    let affected = tx
        .execute("DELETE FROM products WHERE id = ?1", params![id])
        .map_err(|e| e.to_string())?;

    tx.execute(
        "INSERT INTO audit_log (action, table_name, record_id) VALUES (?, ?, ?)",
        params!["DELETE", "products", id.to_string()],
    )
    .map_err(|e| e.to_string())?;

    tx.commit().map_err(|e| e.to_string())?;

    if affected == 0 {
        return Err("Product not found".to_string());
    }

    Ok(())
}

pub fn add_category_with_pool(
    pool: &DbPool,
    name: String,
) -> Result<crate::models::Category, String> {
    let normalized_name = logic::normalize_product_name(&name)?;

    let mut conn = pool.get().map_err(|e| e.to_string())?;
    let tx = conn.transaction().map_err(|e| e.to_string())?;

    if get_category_by_name(&tx, &normalized_name)?.is_some() {
        return Err("Category already exists".to_string());
    }

    tx.execute(
        "INSERT INTO categories (name) VALUES (?1)",
        params![normalized_name],
    )
    .map_err(|e| e.to_string())?;

    let id = tx.last_insert_rowid() as i32;
    let category = crate::models::Category {
        id,
        name: normalized_name.clone(),
    };

    tx.execute(
        "INSERT INTO audit_log (action, table_name, record_id, new_values) VALUES (?, ?, ?, ?)",
        params![
            "INSERT",
            "categories",
            id.to_string(),
            serde_json::to_string(&category).unwrap_or_else(|_| String::new()),
        ],
    )
    .map_err(|e| e.to_string())?;

    tx.commit().map_err(|e| e.to_string())?;

    Ok(category)
}

pub fn update_category_with_pool(
    pool: &DbPool,
    id: i32,
    name: String,
) -> Result<crate::models::Category, String> {
    let normalized_name = logic::normalize_product_name(&name)?;

    let mut conn = pool.get().map_err(|e| e.to_string())?;
    let tx = conn.transaction().map_err(|e| e.to_string())?;

    let old_category = tx
        .query_row(
            "SELECT id, name FROM categories WHERE id = ?1",
            params![id],
            |row| {
                Ok(crate::models::Category {
                    id: row.get(0)?,
                    name: row.get(1)?,
                })
            },
        )
        .optional()
        .map_err(|e| e.to_string())?;

    let old_category = match old_category {
        Some(category) => category,
        None => return Err("Category not found".to_string()),
    };

    if old_category.name != normalized_name {
        if let Some(existing_id) = get_category_by_name(&tx, &normalized_name)? {
            if existing_id != id {
                return Err("Category already exists".to_string());
            }
        }
    }

    let affected = tx
        .execute(
            "UPDATE categories SET name = ?1 WHERE id = ?2",
            params![normalized_name, id],
        )
        .map_err(|e| e.to_string())?;

    if affected == 0 {
        return Err("Category not found".to_string());
    }

    let category = crate::models::Category {
        id,
        name: normalized_name.clone(),
    };

    tx.execute(
        "INSERT INTO audit_log (action, table_name, record_id, old_values, new_values) VALUES (?, ?, ?, ?, ?)",
        params![
            "UPDATE",
            "categories",
            id.to_string(),
            serde_json::to_string(&old_category).unwrap_or_else(|_| String::new()),
            serde_json::to_string(&category).unwrap_or_else(|_| String::new()),
        ],
    ).map_err(|e| e.to_string())?;

    tx.commit().map_err(|e| e.to_string())?;

    Ok(category)
}

pub fn delete_category_with_pool(pool: &DbPool, id: i32) -> Result<(), String> {
    if id == 1 {
        return Err("Default category cannot be deleted".to_string());
    }

    let mut conn = pool.get().map_err(|e| e.to_string())?;
    let tx = conn.transaction().map_err(|e| e.to_string())?;

    let old_category = tx
        .query_row(
            "SELECT id, name FROM categories WHERE id = ?1",
            params![id],
            |row| {
                Ok(crate::models::Category {
                    id: row.get(0)?,
                    name: row.get(1)?,
                })
            },
        )
        .optional()
        .map_err(|e| e.to_string())?;

    let old_category = match old_category {
        Some(category) => category,
        None => return Err("Category not found".to_string()),
    };

    tx.execute(
        "UPDATE products SET category_id = 1 WHERE category_id = ?1",
        params![id],
    )
    .map_err(|e| e.to_string())?;

    let affected = tx
        .execute("DELETE FROM categories WHERE id = ?1", params![id])
        .map_err(|e| e.to_string())?;

    if affected == 0 {
        return Err("Category not found".to_string());
    }

    tx.execute(
        "INSERT INTO audit_log (action, table_name, record_id, old_values) VALUES (?, ?, ?, ?)",
        params![
            "DELETE",
            "categories",
            id.to_string(),
            serde_json::to_string(&old_category).unwrap_or_else(|_| String::new()),
        ],
    )
    .map_err(|e| e.to_string())?;

    tx.commit().map_err(|e| e.to_string())?;

    Ok(())
}

pub fn get_categories_with_pool(pool: &DbPool) -> Result<Vec<crate::models::Category>, String> {
    let conn = pool.get().map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare("SELECT id, name FROM categories ORDER BY name")
        .map_err(|e| e.to_string())?;

    let categories = stmt
        .query_map([], |row| {
            Ok(crate::models::Category {
                id: row.get(0)?,
                name: row.get(1)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(categories)
}
