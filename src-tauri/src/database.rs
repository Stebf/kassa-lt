use chrono::Utc;
use rusqlite::params;
use rusqlite::OptionalExtension;
use std::collections::HashMap;
use std::path::PathBuf;
use uuid::Uuid;

use r2d2::{ManageConnection, Pool};

use crate::logic;
use crate::models::ProductSalesCount;
use crate::models::{CartItem, Order, OrderItem, Product, Tab};

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

fn migrate_orders_comment_column(conn: &rusqlite::Connection) -> Result<(), String> {
    let has_comment_column: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM pragma_table_info('orders') WHERE name = 'comment'",
            [],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;

    if has_comment_column == 0 {
        conn.execute(
            "ALTER TABLE orders ADD COLUMN comment TEXT NOT NULL DEFAULT ''",
            [],
        )
        .map_err(|e| e.to_string())?;
    }

    Ok(())
}

fn migrate_products_tabs_table(conn: &rusqlite::Connection) -> Result<(), String> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS product_tabs (
            product_id INTEGER NOT NULL,
            tab_id INTEGER NOT NULL,
            PRIMARY KEY(product_id, tab_id),
            FOREIGN KEY(product_id) REFERENCES products(id) ON DELETE CASCADE,
            FOREIGN KEY(tab_id) REFERENCES tabs(id) ON DELETE CASCADE
        );",
    )
    .map_err(|e| e.to_string())?;

    let has_tab_id_column: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM pragma_table_info('products') WHERE name = 'tab_id'",
            [],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;

    if has_tab_id_column > 0 {
        conn.execute(
            "INSERT OR IGNORE INTO product_tabs (product_id, tab_id)
             SELECT p.id, COALESCE((SELECT id FROM tabs t WHERE t.id = p.tab_id), 1)
             FROM products p",
            [],
        )
        .map_err(|e| e.to_string())?;
    }

    let has_tab_name_column: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM pragma_table_info('products') WHERE name = 'tab_name'",
            [],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;

    if has_tab_name_column > 0 {
        conn.execute(
            "INSERT OR IGNORE INTO tabs (name)
             SELECT DISTINCT TRIM(tab_name)
             FROM products
             WHERE tab_name IS NOT NULL AND TRIM(tab_name) != ''",
            [],
        )
        .map_err(|e| e.to_string())?;

        conn.execute(
            "INSERT OR IGNORE INTO product_tabs (product_id, tab_id)
             SELECT p.id, t.id
             FROM products p
             JOIN tabs t ON t.name = TRIM(p.tab_name)
             WHERE p.tab_name IS NOT NULL AND TRIM(p.tab_name) != ''",
            [],
        )
        .map_err(|e| e.to_string())?;
    }

    Ok(())
}

fn migrate_products_tab_id_column(conn: &rusqlite::Connection) -> Result<(), String> {
    let has_tab_id_column: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM pragma_table_info('products') WHERE name = 'tab_id'",
            [],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;

    if has_tab_id_column == 0 {
        conn.execute(
            "ALTER TABLE products ADD COLUMN tab_id INTEGER NOT NULL DEFAULT 1",
            [],
        )
        .map_err(|e| e.to_string())?;
    }

    let has_tab_name_column: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM pragma_table_info('products') WHERE name = 'tab_name'",
            [],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;

    if has_tab_name_column > 0 {
        conn.execute(
            "INSERT OR IGNORE INTO tabs (name)
             SELECT DISTINCT TRIM(tab_name)
             FROM products
             WHERE tab_name IS NOT NULL AND TRIM(tab_name) != ''",
            [],
        )
        .map_err(|e| e.to_string())?;

        conn.execute(
            "UPDATE products
             SET tab_id = COALESCE((SELECT t.id FROM tabs t WHERE t.name = TRIM(products.tab_name)), 1)
             WHERE tab_name IS NOT NULL AND TRIM(tab_name) != ''",
            [],
        )
        .map_err(|e| e.to_string())?;
    }

    Ok(())
}

fn migrate_product_sales_state_table(conn: &rusqlite::Connection) -> Result<(), String> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS product_sales_state (
            product_id INTEGER PRIMARY KEY,
            sales_limit INTEGER,
            sales_used INTEGER NOT NULL DEFAULT 0,
            FOREIGN KEY(product_id) REFERENCES products(id) ON DELETE CASCADE
        );",
    )
    .map_err(|e| e.to_string())?;

    conn.execute(
        "INSERT OR IGNORE INTO product_sales_state (product_id, sales_limit, sales_used)
         SELECT id, NULL, 0
         FROM products",
        [],
    )
    .map_err(|e| e.to_string())?;

    Ok(())
}

pub fn init_db_with_pool(pool: &DbPool) -> Result<(), String> {
    let conn = pool.get().map_err(|e| e.to_string())?;

    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS categories (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL UNIQUE
        );

        CREATE TABLE IF NOT EXISTS tabs (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL UNIQUE
        );

        CREATE TABLE IF NOT EXISTS products (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            price_cents INTEGER NOT NULL,
            category_id INTEGER NOT NULL,
            tab_id INTEGER NOT NULL DEFAULT 1,
            FOREIGN KEY(category_id) REFERENCES categories(id),
            FOREIGN KEY(tab_id) REFERENCES tabs(id)
        );

        CREATE TABLE IF NOT EXISTS product_tabs (
            product_id INTEGER NOT NULL,
            tab_id INTEGER NOT NULL,
            PRIMARY KEY(product_id, tab_id),
            FOREIGN KEY(product_id) REFERENCES products(id) ON DELETE CASCADE,
            FOREIGN KEY(tab_id) REFERENCES tabs(id) ON DELETE CASCADE
        );

        CREATE TABLE IF NOT EXISTS product_sales_state (
            product_id INTEGER PRIMARY KEY,
            sales_limit INTEGER,
            sales_used INTEGER NOT NULL DEFAULT 0,
            FOREIGN KEY(product_id) REFERENCES products(id) ON DELETE CASCADE
        );

        CREATE TABLE IF NOT EXISTS orders (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            uuid TEXT NOT NULL UNIQUE,
            created_at TEXT NOT NULL,
            total_cents INTEGER NOT NULL,
            payment_method TEXT NOT NULL,
            comment TEXT
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

    // Ensure default category and tab exist before running migrations that may
    // insert rows referencing them (e.g. product_tabs migration).
    conn.execute(
        "INSERT OR IGNORE INTO categories (id, name) VALUES (1, 'Default')",
        [],
    )
    .map_err(|e| e.to_string())?;

    conn.execute(
        "INSERT OR IGNORE INTO tabs (id, name) VALUES (1, 'Alle')",
        [],
    )
    .map_err(|e| e.to_string())?;

    conn.execute("UPDATE tabs SET name = 'Alle' WHERE id = 1", [])
        .map_err(|e| e.to_string())?;

    migrate_orders_comment_column(&conn)?;
    migrate_products_tabs_table(&conn)?;
    migrate_products_tab_id_column(&conn)?;
    migrate_product_sales_state_table(&conn)?;

    Ok(())
}

pub fn get_products_with_pool(pool: &DbPool) -> Result<Vec<Product>, String> {
    let conn = pool.get().map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare(
            "SELECT p.id, p.name, p.price_cents, c.id, c.name, s.sales_limit, COALESCE(s.sales_used, 0)
         FROM products p
         JOIN categories c ON p.category_id = c.id
         LEFT JOIN product_sales_state s ON s.product_id = p.id
         ORDER BY p.id",
        )
        .map_err(|e| e.to_string())?;

    let products = stmt
        .query_map([], |row| {
            Ok((
                row.get::<_, i32>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, i32>(2)?,
                row.get::<_, i32>(3)?,
                row.get::<_, String>(4)?,
                row.get::<_, Option<i32>>(5)?,
                row.get::<_, i32>(6)?,
            ))
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?
        .into_iter()
        .map(|(id, name, price_cents, category_id, category_name, sales_limit, sales_used)| {
            let tabs = get_tabs_for_product(&conn, id)?;
            Ok(Product {
                id,
                name,
                price: price_cents as f64 / 100.0,
                category_id,
                category_name,
                sales_limit,
                sales_used,
                tabs,
            })
        })
        .collect::<Result<Vec<_>, String>>()?;

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

pub fn get_tab_by_name(tx: &rusqlite::Transaction, tab_name: &str) -> Result<Option<i32>, String> {
    let mut stmt = tx
        .prepare("SELECT id FROM tabs WHERE name = ?1")
        .map_err(|e| e.to_string())?;

    let res = stmt
        .query_row(params![tab_name], |row| row.get::<_, i32>(0))
        .optional()
        .map_err(|e| e.to_string())?;
    Ok(res)
}

pub fn get_tabs_for_product(
    conn: &rusqlite::Connection,
    product_id: i32,
) -> Result<Vec<Tab>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT t.id, t.name
             FROM product_tabs pt
             JOIN tabs t ON pt.tab_id = t.id
             WHERE pt.product_id = ?1
             ORDER BY t.name",
        )
        .map_err(|e| e.to_string())?;

    let tabs = stmt
        .query_map(params![product_id], |row| {
            Ok(Tab {
                id: row.get(0)?,
                name: row.get(1)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(tabs)
}

pub fn get_tabs_by_ids(conn: &rusqlite::Connection, tab_ids: &[i32]) -> Result<Vec<Tab>, String> {
    if tab_ids.is_empty() {
        return Err("At least one tab is required".to_string());
    }

    let mut tabs = Vec::with_capacity(tab_ids.len());

    for tab_id in tab_ids {
        let tab = conn
            .query_row(
                "SELECT id, name FROM tabs WHERE id = ?1",
                params![tab_id],
                |row| {
                    Ok(Tab {
                        id: row.get(0)?,
                        name: row.get(1)?,
                    })
                },
            )
            .optional()
            .map_err(|e| e.to_string())?
            .ok_or_else(|| format!("Tab not found: {}", tab_id))?;

        tabs.push(tab);
    }

    Ok(tabs)
}

pub fn add_product_with_pool(
    pool: &DbPool,
    name: String,
    price: f64,
    category: Option<String>,
    tab_ids: Option<Vec<i32>>,
    sales_limit: Option<i32>,
) -> Result<Product, String> {
    let normalized_name = logic::normalize_product_name(&name)?;
    logic::validate_price(price)?;
    // Validate incoming sales_limit: only allow None or non-negative integers.
    if let Some(v) = sales_limit {
        if v < 0 {
            return Err("Sales limit must be greater than or equal to 0".to_string());
        }
    }
    let normalized_sales_limit = sales_limit;

    let mut conn = pool.get().map_err(|e| e.to_string())?;
    let tx = conn.transaction().map_err(|e| e.to_string())?;
    let price_cents = logic::price_to_cents(price);

    let category_name = category.unwrap_or_else(|| "Default".to_string());
    let category_id = match get_category_by_name(&tx, &category_name)? {
        Some(id) => id,
        None => create_category(&tx, &category_name)?,
    };

    let final_tab_ids = normalize_tab_ids(tab_ids)?;
    let primary_tab_id = final_tab_ids[0];

    tx.execute(
        "INSERT INTO products (name, price_cents, category_id, tab_id) VALUES (?1, ?2, ?3, ?4)",
        params![normalized_name, price_cents, category_id, primary_tab_id],
    )
    .map_err(|e| e.to_string())?;

    let id = tx.last_insert_rowid() as i32;

    tx.execute(
        "INSERT INTO product_sales_state (product_id, sales_limit, sales_used) VALUES (?1, ?2, 0)",
        params![id, normalized_sales_limit],
    )
    .map_err(|e| e.to_string())?;

    for tab_id in &final_tab_ids {
        tx.execute(
            "INSERT OR IGNORE INTO product_tabs (product_id, tab_id) VALUES (?1, ?2)",
            params![id, tab_id],
        )
        .map_err(|e| e.to_string())?;
    }

    let tabs = get_tabs_for_product(&tx, id)?;
    let category_name_for_audit = category_name.clone();

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
                category_name: category_name_for_audit,
                sales_limit,
                sales_used: 0,
                tabs: tabs.clone(),
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
        sales_limit: normalized_sales_limit,
        sales_used: 0,
        tabs,
    })
}

fn normalize_tab_ids(tab_ids: Option<Vec<i32>>) -> Result<Vec<i32>, String> {
    let mut ids = tab_ids.unwrap_or_else(|| vec![1]);

    if ids.is_empty() {
        ids.push(1);
    }

    ids.sort_unstable();
    ids.dedup();

    if ids.iter().any(|id| *id <= 0) {
        return Err("Invalid tab id".to_string());
    }

    Ok(ids)
}

pub fn checkout_with_pool(
    pool: &DbPool,
    items: Vec<CartItem>,
    payment_method: String,
    comment: String,
) -> Result<Order, String> {
    if items.is_empty() {
        return Err("Cart is empty".to_string());
    }

    let mut conn = pool.get().map_err(|e| e.to_string())?;
    let tx = conn.transaction().map_err(|e| e.to_string())?;

    let mut items_by_product_id: HashMap<i32, i32> = HashMap::new();
    for item in &items {
        *items_by_product_id.entry(item.id).or_insert(0) += item.quantity;
    }

    let mut quota_updates = Vec::with_capacity(items_by_product_id.len());
    for (product_id, quantity) in &items_by_product_id {
        let product = tx
            .query_row(
                "SELECT p.name, s.sales_limit, COALESCE(s.sales_used, 0)
                 FROM products p
                 LEFT JOIN product_sales_state s ON s.product_id = p.id
                 WHERE p.id = ?1",
                params![product_id],
                |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, Option<i32>>(1)?,
                        row.get::<_, i32>(2)?,
                    ))
                },
            )
            .optional()
            .map_err(|e| e.to_string())?
            .ok_or_else(|| format!("Product not found: {}", product_id))?;

        if let Some(sales_limit) = product.1 {
            if product.2 + *quantity > sales_limit {
                return Err(format!("Sales limit exceeded for product {}", product.0));
            }
        }

        quota_updates.push((*product_id, *quantity));
    }

    let order_uuid = Uuid::new_v4().to_string();
    let created_at = Utc::now().to_rfc3339();

    let total_cents = logic::cart_total_cents(&items);

    tx.execute(
        "INSERT INTO orders (uuid, created_at, total_cents, payment_method, comment) VALUES (?1, ?2, ?3, ?4, ?5)",
        params![&order_uuid, &created_at, total_cents, &payment_method, &comment],
    ).map_err(|e| e.to_string())?;

    let order_id = tx.last_insert_rowid();

    for item in &items {
        let price_cents = logic::price_to_cents(item.price);
        tx.execute(
            "INSERT INTO order_items (order_id, name, price_cents, quantity) VALUES (?1, ?2, ?3, ?4)",
            params![&order_id, &item.name, price_cents, item.quantity],
        ).map_err(|e| e.to_string())?;
    }

    for (product_id, quantity) in quota_updates {
        tx.execute(
            "UPDATE product_sales_state SET sales_used = sales_used + ?2 WHERE product_id = ?1",
            params![product_id, quantity],
        )
        .map_err(|e| e.to_string())?;
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
                comment: comment.clone(),
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
        comment,
        items: logic::order_items_from_cart(&items),
    })
}

pub fn get_orders_with_pool(pool: &DbPool) -> Result<Vec<Order>, String> {
    let conn = pool.get().map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare(
            "SELECT id, uuid, created_at, total_cents, payment_method, comment FROM orders ORDER BY id DESC",
        )
        .map_err(|e| e.to_string())?;

    let orders = stmt
        .query_map([], |row| {
            let order_id: i64 = row.get(0)?;
            let order_uuid: String = row.get(1)?;
            let created_at: String = row.get(2)?;
            let total_cents: i32 = row.get(3)?;
            let payment_method: String = row.get(4)?;
            let comment: String = row.get(5)?;

            Ok((
                order_id,
                order_uuid,
                created_at,
                total_cents,
                payment_method,
                comment,
            ))
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    let mut result = Vec::new();
    for (order_id, order_uuid, created_at, total_cents, payment_method, comment) in orders {
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
            comment,
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
            "SELECT p.id, p.name, p.price_cents, c.id, c.name, s.sales_limit, COALESCE(s.sales_used, 0)
         FROM products p
         JOIN categories c ON p.category_id = c.id
         LEFT JOIN product_sales_state s ON s.product_id = p.id
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
        let sales_limit: Option<i32> = row.get::<_, Option<i32>>(5).map_err(|e| e.to_string())?;
        let sales_used: i32 = row.get::<_, i32>(6).map_err(|e| e.to_string())?;
        let tabs = get_tabs_for_product(&conn, id)?;

        let product = Product {
            id,
            name,
            price: price_cents as f64 / 100.0,
            category_id,
            category_name,
            sales_limit,
            sales_used,
            tabs,
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
    tab_ids: Option<Vec<i32>>,
    sales_limit: Option<Option<i32>>,
) -> Result<Product, String> {
    let mut conn = pool.get().map_err(|e| e.to_string())?;
    let tx = conn.transaction().map_err(|e| e.to_string())?;

    let current = tx
        .query_row(
            "SELECT p.id, p.name, p.price_cents, p.category_id, c.name, s.sales_limit, COALESCE(s.sales_used, 0)
         FROM products p
         JOIN categories c ON p.category_id = c.id
         LEFT JOIN product_sales_state s ON s.product_id = p.id
         WHERE p.id = ?1",
            params![id],
            |row| {
                Ok((
                    row.get::<_, i32>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, i32>(2)?,
                    row.get::<_, i32>(3)?,
                    row.get::<_, String>(4)?,
                    row.get::<_, Option<i32>>(5)?,
                    row.get::<_, i32>(6)?,
                ))
            },
        )
        .map_err(|e| e.to_string())?;

    let current_tabs = get_tabs_for_product(&tx, id)?;

    let old_values = Product {
        id: current.0,
        name: current.1.clone(),
        price: current.2 as f64 / 100.0,
        category_id: current.3,
        category_name: current.4.clone(),
        sales_limit: current.5,
        sales_used: current.6,
        tabs: current_tabs.clone(),
    };

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

    // Normalize incoming sentinel -1 to NULL: sales_limit is Option<Option<i32>>
    let final_sales_limit = if let Some(new_sales_limit) = sales_limit {
        // new_sales_limit: Option<i32>
        if let Some(v) = new_sales_limit {
            if v < 0 {
                return Err("Sales limit must be greater than or equal to 0".to_string());
            }
        }
        new_sales_limit
    } else {
        current.5
    };

    let final_tab_ids = if let Some(new_tab_ids) = tab_ids {
        normalize_tab_ids(Some(new_tab_ids))?
    } else {
        current_tabs.iter().map(|tab| tab.id).collect()
    };

    let final_tabs = get_tabs_by_ids(&tx, &final_tab_ids)?;
    let final_primary_tab_id = final_tabs[0].id;

    let final_category_id = match get_category_by_name(&tx, &final_category_name)? {
        Some(id) => id,
        None => create_category(&tx, &final_category_name)?,
    };

    let price_cents = logic::price_to_cents(final_price);

    let affected = tx
        .execute(
            "UPDATE products SET name = ?1, price_cents = ?2, category_id = ?3, tab_id = ?4 WHERE id = ?5",
            params![final_name, price_cents, final_category_id, final_primary_tab_id, id],
        )
        .map_err(|e| e.to_string())?;

    tx.execute(
        "INSERT OR IGNORE INTO product_sales_state (product_id, sales_limit, sales_used) VALUES (?1, ?2, ?3)",
        params![id, final_sales_limit, current.6],
    )
    .map_err(|e| e.to_string())?;

    tx.execute(
        "UPDATE product_sales_state SET sales_limit = ?2 WHERE product_id = ?1",
        params![id, final_sales_limit],
    )
    .map_err(|e| e.to_string())?;

    tx.execute(
        "DELETE FROM product_tabs WHERE product_id = ?1",
        params![id],
    )
    .map_err(|e| e.to_string())?;

    for tab_id in &final_tab_ids {
        tx.execute(
            "INSERT OR IGNORE INTO product_tabs (product_id, tab_id) VALUES (?1, ?2)",
            params![id, tab_id],
        )
        .map_err(|e| e.to_string())?;
    }

    tx.execute(
        "INSERT INTO audit_log (action, table_name, record_id, old_values, new_values) VALUES (?, ?, ?, ?, ?)",
        params![
            "UPDATE",
            "products",
            id.to_string(),
            serde_json::to_string(&old_values).unwrap_or_else(|_| String::new()),
            serde_json::to_string(&Product { id, name: final_name.clone(), price: final_price, category_id: final_category_id, category_name: final_category_name.clone(), sales_limit: final_sales_limit, sales_used: current.6, tabs: final_tabs.clone() }).unwrap_or_else(|_| String::new())
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
        sales_limit: final_sales_limit,
        sales_used: current.6,
        tabs: final_tabs,
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

pub fn add_tab_with_pool(pool: &DbPool, name: String) -> Result<Tab, String> {
    let normalized_name = logic::normalize_product_name(&name)?;

    let mut conn = pool.get().map_err(|e| e.to_string())?;
    let tx = conn.transaction().map_err(|e| e.to_string())?;

    if get_tab_by_name(&tx, &normalized_name)?.is_some() {
        return Err("Tab already exists".to_string());
    }

    tx.execute(
        "INSERT INTO tabs (name) VALUES (?1)",
        params![normalized_name],
    )
    .map_err(|e| e.to_string())?;

    let id = tx.last_insert_rowid() as i32;
    let tab = Tab {
        id,
        name: normalized_name.clone(),
    };

    tx.execute(
        "INSERT INTO audit_log (action, table_name, record_id, new_values) VALUES (?, ?, ?, ?)",
        params![
            "INSERT",
            "tabs",
            id.to_string(),
            serde_json::to_string(&tab).unwrap_or_else(|_| String::new()),
        ],
    )
    .map_err(|e| e.to_string())?;

    tx.commit().map_err(|e| e.to_string())?;

    Ok(tab)
}

pub fn update_tab_with_pool(pool: &DbPool, id: i32, name: String) -> Result<Tab, String> {
    let normalized_name = logic::normalize_product_name(&name)?;

    let mut conn = pool.get().map_err(|e| e.to_string())?;
    let tx = conn.transaction().map_err(|e| e.to_string())?;

    let old_tab = tx
        .query_row(
            "SELECT id, name FROM tabs WHERE id = ?1",
            params![id],
            |row| {
                Ok(Tab {
                    id: row.get(0)?,
                    name: row.get(1)?,
                })
            },
        )
        .optional()
        .map_err(|e| e.to_string())?;

    let old_tab = match old_tab {
        Some(tab) => tab,
        None => return Err("Tab not found".to_string()),
    };

    if old_tab.name != normalized_name {
        if let Some(existing_id) = get_tab_by_name(&tx, &normalized_name)? {
            if existing_id != id {
                return Err("Tab already exists".to_string());
            }
        }
    }

    let affected = tx
        .execute(
            "UPDATE tabs SET name = ?1 WHERE id = ?2",
            params![normalized_name, id],
        )
        .map_err(|e| e.to_string())?;

    if affected == 0 {
        return Err("Tab not found".to_string());
    }

    let tab = Tab {
        id,
        name: normalized_name.clone(),
    };

    tx.execute(
        "INSERT INTO audit_log (action, table_name, record_id, old_values, new_values) VALUES (?, ?, ?, ?, ?)",
        params![
            "UPDATE",
            "tabs",
            id.to_string(),
            serde_json::to_string(&old_tab).unwrap_or_else(|_| String::new()),
            serde_json::to_string(&tab).unwrap_or_else(|_| String::new()),
        ],
    )
    .map_err(|e| e.to_string())?;

    tx.commit().map_err(|e| e.to_string())?;

    Ok(tab)
}

pub fn delete_tab_with_pool(pool: &DbPool, id: i32) -> Result<(), String> {
    if id == 1 {
        return Err("Default tab cannot be deleted".to_string());
    }

    let mut conn = pool.get().map_err(|e| e.to_string())?;
    let tx = conn.transaction().map_err(|e| e.to_string())?;

    let old_tab = tx
        .query_row(
            "SELECT id, name FROM tabs WHERE id = ?1",
            params![id],
            |row| {
                Ok(Tab {
                    id: row.get(0)?,
                    name: row.get(1)?,
                })
            },
        )
        .optional()
        .map_err(|e| e.to_string())?;

    let old_tab = match old_tab {
        Some(tab) => tab,
        None => return Err("Tab not found".to_string()),
    };

    tx.execute("DELETE FROM product_tabs WHERE tab_id = ?1", params![id])
        .map_err(|e| e.to_string())?;

    let affected = tx
        .execute("DELETE FROM tabs WHERE id = ?1", params![id])
        .map_err(|e| e.to_string())?;

    if affected == 0 {
        return Err("Tab not found".to_string());
    }

    tx.execute(
        "INSERT INTO audit_log (action, table_name, record_id, old_values) VALUES (?, ?, ?, ?)",
        params![
            "DELETE",
            "tabs",
            id.to_string(),
            serde_json::to_string(&old_tab).unwrap_or_else(|_| String::new()),
        ],
    )
    .map_err(|e| e.to_string())?;

    tx.commit().map_err(|e| e.to_string())?;

    Ok(())
}

pub fn get_tabs_with_pool(pool: &DbPool) -> Result<Vec<Tab>, String> {
    let conn = pool.get().map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare("SELECT id, name FROM tabs ORDER BY name")
        .map_err(|e| e.to_string())?;

    let tabs = stmt
        .query_map([], |row| {
            Ok(Tab {
                id: row.get(0)?,
                name: row.get(1)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(tabs)
}

pub fn backup_with_pool(pool: &DbPool, backup_path: &PathBuf) -> Result<u64, String> {
    let conn = pool.get().map_err(|e| e.to_string())?;

    conn.backup(rusqlite::MAIN_DB, backup_path, None)
        .map_err(|e| e.to_string())?;

    // Get resulting file size for diagnostics
    let metadata = std::fs::metadata(backup_path).map_err(|e| e.to_string())?;
    Ok(metadata.len())
}
