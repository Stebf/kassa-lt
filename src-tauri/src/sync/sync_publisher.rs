use std::sync::{Arc, RwLock};

pub trait SyncPublisher: Send + Sync {
    fn publish_sale_created(&self, order: &crate::models::Order) -> Result<(), String>;
}

pub struct NoopPublisher;
impl SyncPublisher for NoopPublisher {
    fn publish_sale_created(&self, _order: &crate::models::Order) -> Result<(), String> {
        Ok(())
    }
}

pub struct OutboxPublisher { /* pool, serializer ... */ }
impl SyncPublisher for OutboxPublisher {
    fn publish_sale_created(&self, order: &crate::models::Order) -> Result<(), String> {
        // insert into outbox_events
        Ok(())
    }
}

pub struct SyncRouter {
    inner: RwLock<Arc<dyn SyncPublisher>>,
}
impl SyncRouter {
    pub fn new_disabled() -> Self {
        Self { inner: RwLock::new(Arc::new(NoopPublisher)) }
    }
    pub fn set_enabled(&self, enabled: bool, outbox: Arc<OutboxPublisher>) {
        let mut w = self.inner.write().unwrap();
        *w = if enabled { outbox } else { Arc::new(NoopPublisher) };
    }
    pub fn publisher(&self) -> Arc<dyn SyncPublisher> {
        self.inner.read().unwrap().clone()
    }
}