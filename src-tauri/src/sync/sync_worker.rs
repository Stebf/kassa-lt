use crate::config::SyncWorkerConfig;
use crate::database::{self, DbPool};

pub struct SyncWorker {
	pool: DbPool,
	config: SyncWorkerConfig,
}

impl SyncWorker {
	pub fn new(pool: DbPool, config: SyncWorkerConfig) -> Self {
		Self { pool, config }
	}

	pub fn is_enabled(&self) -> bool {
		self.config.enabled
	}

	pub async fn run_once(&self) -> Result<(), String> {
		if !self.config.enabled {
			return Ok(());
		}

		let pending = database::list_pending_sync_outbox_entries_with_pool(&self.pool)?;
		if pending.is_empty() {
			return Ok(());
		}

		for entry in pending {
			let _ = entry;
		}

		Ok(())
	}
}
