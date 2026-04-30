use app_lib::commands::{
    add_product_with_pool,
    delete_product_with_pool,
    checkout_with_pool,
    get_product_with_pool,
    get_orders_with_pool,
    get_products_with_pool,
    init_db_with_pool,
    DbPool,
    SqliteManager,
    update_product_with_pool,
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

#[test]
fn add_and_get_product_round_trips_through_database() {
    let (_dir, pool) = test_pool();

    let created = add_product_with_pool(&pool, "Coffee".to_string(), 2.50).unwrap();
    let fetched = get_product_with_pool(&pool, created.id).unwrap();

    assert_eq!(fetched.id, created.id);
    assert_eq!(fetched.name, "Coffee");
    assert_eq!(fetched.price, 2.50);
}

#[test]
fn update_product_can_change_name_only() {
    let (_dir, pool) = test_pool();

    let created = add_product_with_pool(&pool, "Tea".to_string(), 1.20).unwrap();
    let updated = update_product_with_pool(&pool, created.id, Some("Green Tea".to_string()), None).unwrap();

    assert_eq!(updated.id, created.id);
    assert_eq!(updated.name, "Green Tea");
    assert_eq!(updated.price, 1.20);

    let fetched = get_product_with_pool(&pool, created.id).unwrap();
    assert_eq!(fetched.name, "Green Tea");
    assert_eq!(fetched.price, 1.20);
}

#[test]
fn update_product_can_change_price_only() {
    let (_dir, pool) = test_pool();

    let created = add_product_with_pool(&pool, "Cake".to_string(), 3.40).unwrap();
    let updated = update_product_with_pool(&pool, created.id, None, Some(4.10)).unwrap();

    assert_eq!(updated.id, created.id);
    assert_eq!(updated.name, "Cake");
    assert_eq!(updated.price, 4.10);

    let fetched = get_product_with_pool(&pool, created.id).unwrap();
    assert_eq!(fetched.name, "Cake");
    assert_eq!(fetched.price, 4.10);
}

#[test]
fn delete_product_removes_row() {
    let (_dir, pool) = test_pool();

    let created = add_product_with_pool(&pool, "Latte".to_string(), 2.90).unwrap();
    delete_product_with_pool(&pool, created.id).unwrap();

    assert_eq!(get_product_with_pool(&pool, created.id).unwrap_err(), "Product not found");
}