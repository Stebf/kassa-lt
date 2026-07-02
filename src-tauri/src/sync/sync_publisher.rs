use std::sync::{Arc, RwLock};

use serde::{Deserialize, Serialize};

use crate::database::{enqueue_sync_outbox_event_with_pool, DbPool};
use crate::models::Order;

const SALE_CREATED_EVENT_TYPE: &str = "sale.created";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaleCreatedPayload {
    pub order: Order,
}

pub trait SyncPublisher: Send + Sync {
    fn publish_sale_created(&self, order: &Order) -> Result<(), String>;
}

pub struct NoopPublisher;
impl SyncPublisher for NoopPublisher {
    fn publish_sale_created(&self, _order: &Order) -> Result<(), String> {
        Ok(())
    }
}

#[derive(Clone)]
pub struct OutboxPublisher {
    pool: DbPool,
}

impl OutboxPublisher {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }
}

impl SyncPublisher for OutboxPublisher {
    fn publish_sale_created(&self, order: &Order) -> Result<(), String> {
        let payload = serde_json::to_value(SaleCreatedPayload {
            order: order.clone(),
        })
        .map_err(|e| e.to_string())?;

        enqueue_sync_outbox_event_with_pool(&self.pool, SALE_CREATED_EVENT_TYPE, &payload)?;
        Ok(())
    }
}

pub struct SyncRouter {
    inner: RwLock<Arc<dyn SyncPublisher>>,
}
impl SyncRouter {
    pub fn new_disabled() -> Self {
        Self {
            inner: RwLock::new(Arc::new(NoopPublisher)),
        }
    }
    pub fn set_enabled(&self, enabled: bool, outbox: Arc<OutboxPublisher>) {
        let mut w = self.inner.write().unwrap();
        *w = if enabled { outbox } else { Arc::new(NoopPublisher) };
    }
    pub fn publisher(&self) -> Arc<dyn SyncPublisher> {
        self.inner.read().unwrap().clone()
    }
}