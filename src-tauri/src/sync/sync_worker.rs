use crate::config::SyncWorkerConfig;
use crate::database::{self, DbPool};

use log::{error, info, warn};
use serde::Serialize;
use std::time::Duration;
use tokio::time;

#[derive(Clone, Serialize)]
#[serde(tag = "type")]
pub enum SyncState {
	NotRunYet,
	Successful {
		timestamp: chrono::DateTime<chrono::Utc>,
	},
	Failed {
		timestamp: chrono::DateTime<chrono::Utc>,
		error: String,
	},
}

#[derive(Clone)]
pub struct SyncWorker {
	pool: DbPool,
	instance_id: String,
	last_state_tx: tokio::sync::watch::Sender<SyncState>,
	last_state_rx: tokio::sync::watch::Receiver<SyncState>,
	config_rx: tokio::sync::watch::Receiver<SyncWorkerConfig>,
}

impl SyncWorker {
	pub fn new(
		pool: DbPool,
		instance_id: String,
		config_rx: tokio::sync::watch::Receiver<SyncWorkerConfig>,
	) -> Self {
		let (last_state_tx, last_state_rx) = tokio::sync::watch::channel(SyncState::NotRunYet);
		Self {
			pool,
			instance_id,
			last_state_tx,
			last_state_rx,
			config_rx,
		}
	}

	pub fn is_enabled(&self) -> bool {
		self.config_rx.borrow().enabled
	}

	pub fn start(&self) {
		let this = self.clone();

		info!("sync_worker: starting; instance_id={}", this.instance_id);

		tauri::async_runtime::spawn(async move {
			match start_sync_worker(
				this.pool,
				this.instance_id,
				this.config_rx,
				this.last_state_tx,
			)
			.await
			{
				Ok(()) => info!("sync_worker: finished worker"),
				Err(e) => error!("sync_worker: failed: {}", e),
			}
		});
	}

	pub fn get_last_state(&self) -> SyncState {
		self.last_state_rx.borrow().clone()
	}

	pub async fn run_sync_now(&self) -> SyncState {
		let config = self.config_rx.borrow().clone();
		run_sync_and_record_state(
			self.pool.clone(),
			self.instance_id.clone(),
			config,
			self.last_state_tx.clone(),
		)
		.await
	}
}

async fn run_sync_once(pool: DbPool, instance_id: &str, config: &SyncWorkerConfig) -> Result<(), String> {
	if !config.enabled {
		info!("sync_worker: disabled; skipping run; instance_id={}", instance_id);
		return Ok(());
	}

	let pending = database::list_pending_sync_outbox_entries_with_pool(&pool)?;
	if pending.is_empty() {
		info!("sync_worker: no pending outbox entries; instance_id={}", instance_id);
		return Ok(());
	}

	let client = tauri_plugin_http::reqwest::Client::new();
	let uri = format!("{}/sync/outbox", config.central_api_base_url.trim_end_matches('/'));

	for entry in pending {
		info!("sync_worker: syncing outbox entry; id={} event_type={}", entry.id, entry.event_type);
		let body = serde_json::to_string(&entry).map_err(|e| e.to_string())?;
		let response = client
			.post(&uri)
			.body(body)
			.send()
			.await
			.map_err(|e| e.to_string())?;

		if response.status().is_success() {
			database::mark_sync_outbox_entry_sent_with_pool(&pool, entry.id)?;
			info!("sync_worker: outbox entry sent; id={} status={}", entry.id, response.status());
		} else {
			let status = response.status();
			let error_text = response.text().await.unwrap_or_default();
			let error = format!("status={} body={}", status, error_text);
			database::mark_sync_outbox_entry_failed_with_pool(&pool, entry.id, &error)?;
			warn!("sync_worker: outbox entry failed; id={} {}", entry.id, error);
			return Err(error);
		}
	}

	Ok(())
}

async fn run_sync_and_record_state(
	pool: DbPool,
	instance_id: String,
	config: SyncWorkerConfig,
	last_state_tx: tokio::sync::watch::Sender<SyncState>,
) -> SyncState {
	let now = chrono::Utc::now();
	let new_state = match run_sync_once(pool, instance_id.as_str(), &config).await {
		Ok(()) => SyncState::Successful { timestamp: now },
		Err(e) => SyncState::Failed {
			timestamp: now,
			error: e,
		},
	};

	if let Err(e) = last_state_tx.send(new_state.clone()) {
		warn!("sync_worker: failed to update state: {}", e);
	}

	new_state
}

async fn start_sync_worker(
	pool: DbPool,
	instance_id: String,
	mut config_rx: tokio::sync::watch::Receiver<SyncWorkerConfig>,
	last_state_tx: tokio::sync::watch::Sender<SyncState>,
) -> Result<(), String> {
	let mut interval = time::interval(Duration::from_secs(30 * 60));
	loop {
		let pool = pool.clone();

		tokio::select! {
			config = config_rx.changed() => {
				config_rx.borrow_and_update();
				match config {
					Ok(()) => info!("sync worker config changed"),
					Err(e) => warn!("sync worker config watch channel closed: {}", e),
				}
				interval = time::interval(Duration::from_secs(30 * 60));
			}
			_ = interval.tick() => {
				let config = config_rx.borrow_and_update().clone();
				info!("sync worker tick; enabled={} central_api_base_url={}", config.enabled, config.central_api_base_url);
				if config.enabled {
					let _ = run_sync_and_record_state(
						pool,
						instance_id.clone(),
						config,
						last_state_tx.clone(),
					)
					.await;
				} else {
					info!("sync worker is disabled; skipping run");
				}
			}
		}
	}
}
