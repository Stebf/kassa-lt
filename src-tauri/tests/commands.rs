use app_lib::database::{
    add_category_with_pool, add_product_with_pool, checkout_with_pool, delete_category_with_pool,
    delete_product_with_pool, get_categories_with_pool, get_orders_with_pool,
    get_product_sales_count_with_pool, get_product_with_pool, get_products_with_pool,
    init_db_with_pool, update_category_with_pool, update_product_with_pool, DbPool, SqliteManager,
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

    let product =
        add_product_with_pool(&pool, "  Espresso  ".to_string(), 2.5, None, None, None)
            .expect("add product");

    assert_eq!(product.name, "Espresso");
    assert_eq!(product.price, 2.5);
    assert_eq!(product.sales_limit, None);
    assert_eq!(product.sales_used, 0);

    let products = get_products_with_pool(&pool).expect("fetch products");
    assert_eq!(products.len(), 1);
    assert_eq!(products[0].name, "Espresso");
    assert_eq!(products[0].price, 2.5);
}

#[test]
fn add_product_rejects_invalid_input() {
    let (_dir, pool) = test_pool();

    assert_eq!(
        add_product_with_pool(&pool, "   ".to_string(), 1.0, None, None, None).unwrap_err(),
        "Product name cannot be empty"
    );
    assert_eq!(
        add_product_with_pool(&pool, "Tea".to_string(), -0.1, None, None, None).unwrap_err(),
        "Price must be greater than or equal to 0"
    );
}

#[test]
fn add_product_can_set_sales_limit() {
    let (_dir, pool) = test_pool();

    let product =
        add_product_with_pool(&pool, "Juice".to_string(), 3.10, None, None, Some(12))
            .expect("add product");

    assert_eq!(product.sales_limit, Some(12));
    assert_eq!(product.sales_used, 0);

    let fetched = get_product_with_pool(&pool, product.id).expect("fetch product");
    assert_eq!(fetched.sales_limit, Some(12));
    assert_eq!(fetched.sales_used, 0);
}

#[test]
fn add_product_rejects_negative_sales_limit() {
    let (_dir, pool) = test_pool();

    assert_eq!(
        add_product_with_pool(&pool, "Milk".to_string(), 1.90, None, None, Some(-1))
            .unwrap_err(),
        "Sales limit must be greater than or equal to 0"
    );
}

#[test]
fn checkout_persists_order_and_items() {
    let (_dir, pool) = test_pool();

    let tea = add_product_with_pool(&pool, "Tea".to_string(), 1.25, None, None, None)
        .expect("add tea");
    let cake = add_product_with_pool(&pool, "Cake".to_string(), 2.10, None, None, None)
        .expect("add cake");

    let items = vec![
        CartItem {
            id: tea.id,
            name: "Tea".to_string(),
            price: 1.25,
            quantity: 2,
        },
        CartItem {
            id: cake.id,
            name: "Cake".to_string(),
            price: 2.10,
            quantity: 1,
        },
    ];

    let order = checkout_with_pool(
        &pool,
        items.clone(),
        "card".to_string(),
        "Customer requested extra cream".to_string(),
    )
    .expect("checkout");

    assert_eq!(order.payment_method, "card");
    assert_eq!(order.total, 4.60);
    assert_eq!(order.items.len(), 2);
    assert_eq!(order.comment, "Customer requested extra cream");

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

    assert_eq!(
        checkout_with_pool(&pool, Vec::new(), "cash".to_string(), "".to_string()).unwrap_err(),
        "Cart is empty"
    );
}

#[test]
fn add_and_get_product_round_trips_through_database() {
    let (_dir, pool) = test_pool();

    let created = add_product_with_pool(&pool, "Coffee".to_string(), 2.50, None, None, None)
        .unwrap();
    let fetched = get_product_with_pool(&pool, created.id).unwrap();

    assert_eq!(fetched.id, created.id);
    assert_eq!(fetched.name, "Coffee");
    assert_eq!(fetched.price, 2.50);
    assert_eq!(fetched.sales_limit, None);
    assert_eq!(fetched.sales_used, 0);
}

#[test]
fn update_product_can_change_name_only() {
    let (_dir, pool) = test_pool();

    let created = add_product_with_pool(&pool, "Tea".to_string(), 1.20, None, None, None)
        .unwrap();
    let updated = update_product_with_pool(
        &pool,
        created.id,
        Some("Green Tea".to_string()),
        None,
        None,
        None,
        None,
    )
    .unwrap();

    assert_eq!(updated.id, created.id);
    assert_eq!(updated.name, "Green Tea");
    assert_eq!(updated.price, 1.20);

    let fetched = get_product_with_pool(&pool, created.id).unwrap();
    assert_eq!(fetched.name, "Green Tea");
    assert_eq!(fetched.price, 1.20);
}

#[test]
fn update_product_can_clear_sales_limit() {
    let (_dir, pool) = test_pool();

    let created = add_product_with_pool(&pool, "Soda".to_string(), 1.50, None, None, Some(5)).unwrap();

    // clear the sales limit by passing Some(None) through the command API
    let updated = update_product_with_pool(&pool, created.id, None, None, None, None, Some(None)).unwrap();

    assert_eq!(updated.sales_limit, None);

    let fetched = get_product_with_pool(&pool, created.id).unwrap();
    assert_eq!(fetched.sales_limit, None);
}

#[test]
fn update_product_can_change_price_only() {
    let (_dir, pool) = test_pool();

    let created = add_product_with_pool(&pool, "Cake".to_string(), 3.40, None, None, None)
        .unwrap();
    let updated = update_product_with_pool(&pool, created.id, None, Some(4.10), None, None, None)
        .unwrap();

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

    let created = add_product_with_pool(&pool, "Latte".to_string(), 2.90, None, None, None)
        .unwrap();
    delete_product_with_pool(&pool, created.id).unwrap();

    assert_eq!(
        get_product_with_pool(&pool, created.id).unwrap_err(),
        "Product not found"
    );
}

#[test]
fn default_category_exists_after_initialization() {
    let (_dir, pool) = test_pool();

    let categories = get_categories_with_pool(&pool).expect("fetch categories");

    assert_eq!(categories.len(), 1);
    assert_eq!(categories[0].id, 1);
    assert_eq!(categories[0].name, "Default");
}

#[test]
fn add_category_persists_and_trims_name() {
    let (_dir, pool) = test_pool();

    let category = add_category_with_pool(&pool, "  Bakery  ".to_string()).expect("add category");

    assert_eq!(category.name, "Bakery");
    assert!(category.id > 1);

    let categories = get_categories_with_pool(&pool).expect("fetch categories");
    assert_eq!(categories.len(), 2);
    assert!(categories.iter().any(|item| item.name == "Bakery"));
}

#[test]
fn update_category_changes_name() {
    let (_dir, pool) = test_pool();

    let category = add_category_with_pool(&pool, "Drinks".to_string()).expect("add category");
    let updated = update_category_with_pool(&pool, category.id, "Beverages".to_string())
        .expect("update category");

    assert_eq!(updated.id, category.id);
    assert_eq!(updated.name, "Beverages");

    let categories = get_categories_with_pool(&pool).expect("fetch categories");
    assert!(categories
        .iter()
        .any(|item| item.id == category.id && item.name == "Beverages"));
}

#[test]
fn delete_category_reassigns_products_to_default() {
    let (_dir, pool) = test_pool();

    let category = add_category_with_pool(&pool, "Specials".to_string()).expect("add category");
    let product = add_product_with_pool(
        &pool,
        "Soup".to_string(),
        4.20,
        Some(category.name.clone()),
        None,
        None,
    )
    .expect("add product");

    delete_category_with_pool(&pool, category.id).expect("delete category");

    let fetched = get_product_with_pool(&pool, product.id).expect("fetch product");
    assert_eq!(fetched.category_id, 1);
    assert_eq!(fetched.category_name, "Default");

    let categories = get_categories_with_pool(&pool).expect("fetch categories");
    assert!(!categories.iter().any(|item| item.id == category.id));
}

#[test]
fn default_category_cannot_be_deleted() {
    let (_dir, pool) = test_pool();

    assert_eq!(
        delete_category_with_pool(&pool, 1).unwrap_err(),
        "Default category cannot be deleted"
    );
}

#[test]
fn get_product_sales_count_returns_correct_counts() {
    let (_dir, pool) = test_pool();

    let tea = add_product_with_pool(&pool, "Tea".to_string(), 1.25, None, None, None)
        .expect("add tea");
    let cake = add_product_with_pool(&pool, "Cake".to_string(), 2.10, None, None, None)
        .expect("add cake");

    let order_1 = vec![
        CartItem {
            id: tea.id,
            name: "Tea".to_string(),
            price: 1.25,
            quantity: 2,
        },
        CartItem {
            id: cake.id,
            name: "Cake".to_string(),
            price: 2.10,
            quantity: 1,
        },
    ];
    let order_2 = vec![CartItem {
        id: tea.id,
        name: "Tea".to_string(),
        price: 1.25,
        quantity: 4,
    }];

    checkout_with_pool(&pool, order_1.clone(), "card".to_string(), "".to_string())
        .expect("checkout");
    checkout_with_pool(&pool, order_2.clone(), "card".to_string(), "".to_string())
        .expect("checkout");

    let counts = get_product_sales_count_with_pool(&pool).expect("get sales count");
    assert_eq!(counts.len(), 2);
    let tea_count = counts.iter().find(|c| c.product_name == "Tea").unwrap();
    let cake_count = counts.iter().find(|c| c.product_name == "Cake").unwrap();
    assert_eq!(tea_count.count, 6);
    assert_eq!(cake_count.count, 1);
}

#[test]
fn checkout_increments_sales_used_and_blocks_when_limit_is_exceeded() {
    let (_dir, pool) = test_pool();

    let product =
        add_product_with_pool(&pool, "Soda".to_string(), 1.50, None, None, Some(3))
            .expect("add product");

    checkout_with_pool(
        &pool,
        vec![CartItem {
            id: product.id,
            name: product.name.clone(),
            price: product.price,
            quantity: 2,
        }],
        "cash".to_string(),
        "".to_string(),
    )
    .expect("first checkout");

    let fetched = get_product_with_pool(&pool, product.id).expect("fetch product");
    assert_eq!(fetched.sales_used, 2);
    assert_eq!(fetched.sales_limit, Some(3));

    let error = checkout_with_pool(
        &pool,
        vec![CartItem {
            id: product.id,
            name: product.name.clone(),
            price: product.price,
            quantity: 2,
        }],
        "cash".to_string(),
        "".to_string(),
    )
    .unwrap_err();

    assert_eq!(error, "Sales limit exceeded for product Soda");

    let fetched = get_product_with_pool(&pool, product.id).expect("fetch product again");
    assert_eq!(fetched.sales_used, 2);
}
