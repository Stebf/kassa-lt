use app_lib::database::{
    init_db_with_pool, list_pending_sync_outbox_entries_with_pool, DbPool, SqliteManager,
};
use app_lib::logic;
use app_lib::models::{CartItem, Order};
use app_lib::sync::{OutboxPublisher, SaleCreatedPayload, SyncPublisher, SyncRouter};
use r2d2::Pool;
use tempfile::{tempdir, TempDir};

fn test_pool() -> (TempDir, DbPool) {
    let dir = tempdir().expect("create temp dir");
    let db_path = dir.path().join("sync-test.db");
    let manager = SqliteManager::file(db_path);
    let pool = Pool::new(manager).expect("create sqlite pool");
    init_db_with_pool(&pool).expect("initialize test schema");
    (dir, pool)
}

fn sample_order() -> Order {
    let items = vec![CartItem {
        id: 1,
        name: "Coffee".to_string(),
        price: 2.5,
        quantity: 2,
    }];

    Order {
        id: 1,
        uuid: "order-uuid".to_string(),
        created_at: "2026-07-02T00:00:00Z".to_string(),
        total: logic::cart_total_cents(&items) as f64 / 100.0,
        payment_method: "cash".to_string(),
        items: logic::order_items_from_cart(&items),
        comment: "no sugar".to_string(),
    }
}

#[test]
fn outbox_publisher_writes_sale_created_event() {
    let (_dir, pool) = test_pool();
    let publisher = OutboxPublisher::new(pool.clone());
    let order = sample_order();

    publisher.publish_sale_created(&order).expect("publish");

    let events = list_pending_sync_outbox_entries_with_pool(&pool).expect("load events");
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].event_type, "sale.created");

    let payload: SaleCreatedPayload = serde_json::from_str(&events[0].payload).expect("payload");
    assert_eq!(payload.order.uuid, "order-uuid");
    assert_eq!(payload.order.comment, "no sugar");
}

#[test]
fn sync_router_switches_between_disabled_and_outbox_publishers() {
    let (_dir, pool) = test_pool();
    let outbox = std::sync::Arc::new(OutboxPublisher::new(pool));
    let router = SyncRouter::new_disabled();

    router.set_enabled(true, outbox.clone());
    let enabled_publisher = router.publisher();
    enabled_publisher
        .publish_sale_created(&sample_order())
        .expect("publish");

    router.set_enabled(false, outbox);
    let disabled_publisher = router.publisher();
    disabled_publisher
        .publish_sale_created(&sample_order())
        .expect("noop publish");
}