use app_lib::commands::{
    add_product_with_pool,
    checkout_with_pool,
    get_orders_with_pool,
    get_products_with_pool,
    init_db_with_pool,
    DbPool,
    SqliteManager,
};
use app_lib::models::CartItem;
use r2d2::Pool;
use tempfile::{tempdir, TempDir};

fn test_pool() -> (TempDir, DbPool) {
    let dir = tempdir().expect("create temp dir");
    let db_path = dir.path().join("test.db");
    let manager = SqliteManager::file(db_path);
    let pool = Pool::new(manager).expect("create sqlite pool");
    init_db_with_pool(&pool).expect("initialize test schema");
    (dir, pool)
}

#[test]
fn add_product_persists_and_trims_name() {
    let (_dir, pool) = test_pool();

    let product = add_product_with_pool(&pool, "  Espresso  ".to_string(), 2.5).expect("add product");

    assert_eq!(product.name, "Espresso");
    assert_eq!(product.price, 2.5);

    let products = get_products_with_pool(&pool).expect("fetch products");
    assert_eq!(products.len(), 1);
    assert_eq!(products[0].name, "Espresso");
    assert_eq!(products[0].price, 2.5);
}

#[test]
fn add_product_rejects_invalid_input() {
    let (_dir, pool) = test_pool();

    assert_eq!(add_product_with_pool(&pool, "   ".to_string(), 1.0).unwrap_err(), "Product name cannot be empty");
    assert_eq!(add_product_with_pool(&pool, "Tea".to_string(), 0.0).unwrap_err(), "Price must be greater than 0");
}

#[test]
fn checkout_persists_order_and_items() {
    let (_dir, pool) = test_pool();

    let items = vec![
        CartItem { id: 1, name: "Tea".to_string(), price: 1.25, quantity: 2 },
        CartItem { id: 2, name: "Cake".to_string(), price: 2.10, quantity: 1 },
    ];

    let order = checkout_with_pool(&pool, items.clone(), "card".to_string()).expect("checkout");

    assert_eq!(order.payment_method, "card");
    assert_eq!(order.total, 4.60);
    assert_eq!(order.items.len(), 2);

    let orders = get_orders_with_pool(&pool).expect("fetch orders");
    assert_eq!(orders.len(), 1);
    assert_eq!(orders[0].uuid, order.uuid);
    assert_eq!(orders[0].payment_method, "card");
    assert_eq!(orders[0].total, 4.60);
    assert_eq!(orders[0].items.len(), 2);
    assert_eq!(orders[0].items[0].name, "Tea");
    assert_eq!(orders[0].items[1].name, "Cake");
}

#[test]
fn checkout_rejects_empty_cart() {
    let (_dir, pool) = test_pool();

    assert_eq!(checkout_with_pool(&pool, Vec::new(), "cash".to_string()).unwrap_err(), "Cart is empty");
}