// This file is part of Allfeat.

// Copyright (C) 2022-2024 Allfeat.
// SPDX-License-Identifier: GPL-3.0-or-later

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.

// std
use std::{
	path::{Path, PathBuf},
	sync::Arc,
	time::Duration,
};
use fc_rpc::StorageOverride;
// crates.io
use futures::{future, StreamExt};
// use tokio::sync::Semaphore;
// Allfeat
use allfeat_primitives::{BlockNumber, Block, Hash, Hashing};
// frontier
use fc_mapping_sync::{EthereumBlockNotification, EthereumBlockNotificationSinks};
use fc_rpc_core::types::{FeeHistoryCache, FeeHistoryCacheLimit, FilterPool};
// polkadot-sdk
use sc_network_sync::SyncingService;
use sc_service::{Configuration, TaskManager};
use substrate_prometheus_endpoint::Registry as PrometheusRegistry;

use crate::cli::{EthRpcConfig, FrontierBackendType};

#[allow(clippy::too_many_arguments)]
pub fn spawn_tasks<B, BE, C>(
	task_manager: &TaskManager,
	client: Arc<C>,
	backend: Arc<BE>,
	frontier_backend: Arc<fc_db::Backend<B, C>>,
	filter_pool: Option<FilterPool>,
	overrides: Arc<dyn StorageOverride<B>>,
	fee_history_cache: FeeHistoryCache,
	fee_history_cache_limit: FeeHistoryCacheLimit,
	sync: Arc<SyncingService<B>>,
	pubsub_notification_sinks: Arc<EthereumBlockNotificationSinks<EthereumBlockNotification<B>>>,
	eth_rpc_config: EthRpcConfig,
	prometheus: Option<PrometheusRegistry>,
) where
	C: 'static
		+ sp_api::ProvideRuntimeApi<B>
		+ sp_blockchain::HeaderBackend<B>
		+ sp_blockchain::HeaderMetadata<B, Error = sp_blockchain::Error>
		+ sc_client_api::BlockOf
		+ sc_client_api::BlockchainEvents<B>
		+ sc_client_api::backend::StorageProvider<B, BE>,
	C::Api: sp_block_builder::BlockBuilder<B> + fp_rpc::EthereumRuntimeRPCApi<B>,
	// + moonbeam_rpc_primitives_debug::DebugRuntimeApi<B>,
	B: 'static + Send + Sync + sp_runtime::traits::Block<Hash = Hash>,
	B::Header: sp_runtime::traits::Header<Number = BlockNumber>,
	BE: 'static + sc_client_api::backend::Backend<B>,
	BE::State: sc_client_api::backend::StateBackend<Hashing>,
{
	match &*frontier_backend {
		fc_db::Backend::KeyValue(bd) => {
			task_manager.spawn_essential_handle().spawn(
				"frontier-mapping-sync-worker",
				Some("frontier"),
				fc_mapping_sync::kv::MappingSyncWorker::new(
					client.import_notification_stream(),
					Duration::new(6, 0),
					client.clone(),
					backend.clone(),
					overrides.clone(),
					bd.clone(),
					3,
					0,
					fc_mapping_sync::SyncStrategy::Normal,
					sync,
					pubsub_notification_sinks,
				)
				.for_each(|()| future::ready(())),
			);
		},
		fc_db::Backend::Sql(bd) => {
			task_manager.spawn_essential_handle().spawn_blocking(
				"frontier-mapping-sync-worker",
				Some("frontier"),
				fc_mapping_sync::sql::SyncWorker::run(
					client.clone(),
					backend.clone(),
					bd.clone(),
					client.import_notification_stream(),
					fc_mapping_sync::sql::SyncWorkerConfig {
						read_notification_timeout: Duration::from_secs(30),
						check_indexed_blocks_interval: Duration::from_secs(60),
					},
					fc_mapping_sync::SyncStrategy::Parachain,
					sync,
					pubsub_notification_sinks,
				),
			);
		},
	}

	// Spawn Frontier EthFilterApi maintenance task.
	if let Some(filter_pool) = filter_pool {
		// Each filter is allowed to stay in the pool for 100 blocks.
		const FILTER_RETAIN_THRESHOLD: u64 = 100;
		task_manager.spawn_essential_handle().spawn(
			"frontier-filter-pool",
			Some("frontier"),
			fc_rpc::EthTask::filter_pool_task(client.clone(), filter_pool, FILTER_RETAIN_THRESHOLD),
		);
	}

	// Spawn Frontier FeeHistory cache maintenance task.
	task_manager.spawn_essential_handle().spawn(
		"frontier-fee-history",
		Some("frontier"),
		fc_rpc::EthTask::fee_history_task(
			client.clone(),
			overrides,
			fee_history_cache,
			fee_history_cache_limit,
		),
	);

	/*
	if eth_rpc_config.tracing_api.contains(&TracingApi::Debug)
		|| eth_rpc_config.tracing_api.contains(&TracingApi::Trace)
	{
		let permit_pool = Arc::new(Semaphore::new(eth_rpc_config.tracing_max_permits as usize));
		let (trace_filter_task, trace_filter_requester) =
			if eth_rpc_config.tracing_api.contains(&TracingApi::Trace) {
				let (trace_filter_task, trace_filter_requester) = CacheTask::create(
					Arc::clone(&client),
					Arc::clone(&backend),
					Duration::from_secs(eth_rpc_config.tracing_cache_duration),
					Arc::clone(&permit_pool),
					Arc::clone(&overrides),
					prometheus,
				);
				(Some(trace_filter_task), Some(trace_filter_requester))
			} else {
				(None, None)
			};

		let (debug_task, debug_requester) =
			if eth_rpc_config.tracing_api.contains(&TracingApi::Debug) {
				let (debug_task, debug_requester) = DebugHandler::task(
					Arc::clone(&client),
					Arc::clone(&backend),
					match &*frontier_backend {
						fc_db::Backend::KeyValue(bd) => bd.clone(),
						fc_db::Backend::Sql(bd) => bd.clone(),
					},
					Arc::clone(&permit_pool),
					Arc::clone(&overrides),
					eth_rpc_config.tracing_raw_max_memory_usage,
				);
				(Some(debug_task), Some(debug_requester))
			} else {
				(None, None)
			};

		// `trace_filter` cache task. Essential.
		// Proxies rpc requests to it's handler.
		if let Some(trace_filter_task) = trace_filter_task {
			task_manager.spawn_essential_handle().spawn(
				"trace-filter-cache",
				Some("eth-tracing"),
				trace_filter_task,
			);
		}

		// `debug` task if enabled. Essential.
		// Proxies rpc requests to it's handler.
		if let Some(debug_task) = debug_task {
			task_manager.spawn_essential_handle().spawn(
				"tracing_api-debug",
				Some("eth-tracing"),
				debug_task,
			);
		}

		RpcRequesters { debug: debug_requester, trace: trace_filter_requester }
	} else {
		RpcRequesters { debug: None, trace: None }
	}
	*/
}

pub(crate) fn db_config_dir(config: &Configuration) -> PathBuf {
	config.base_path.config_dir(config.chain_spec.id())
}

/// Create a Frontier backend.
pub(crate) fn backend<BE, C>(
	client: Arc<C>,
	config: &sc_service::Configuration,
	storage_override: Arc<dyn StorageOverride<Block>>,
	eth_rpc_config: EthRpcConfig,
) -> Result<fc_db::Backend<Block, C>, String>
where
	BE: 'static + sc_client_api::backend::Backend<Block>,
	C: 'static
		+ sp_api::ProvideRuntimeApi<Block>
		+ sp_blockchain::HeaderBackend<Block>
		+ sc_client_api::backend::StorageProvider<Block, BE>,
	C::Api: fp_rpc::EthereumRuntimeRPCApi<Block>,
{
	let db_config_dir = db_config_dir(config);
	match eth_rpc_config.frontier_backend_type {
		FrontierBackendType::KeyValue => Ok(fc_db::Backend::<Block, C>::KeyValue(Arc::new(
			fc_db::kv::Backend::open(Arc::clone(&client), &config.database, &db_config_dir)?,
		))),
		FrontierBackendType::Sql => {
			let db_path = db_config_dir.join("sql");
			std::fs::create_dir_all(&db_path).expect("failed creating sql db directory");
			let backend = futures::executor::block_on(fc_db::sql::Backend::new(
				fc_db::sql::BackendConfig::Sqlite(fc_db::sql::SqliteBackendConfig {
					path: Path::new("sqlite:///")
						.join(db_path)
						.join("frontier.db3")
						.to_str()
						.unwrap(),
					create_if_missing: true,
					thread_count: eth_rpc_config.frontier_sql_backend_thread_count,
					cache_size: eth_rpc_config.frontier_sql_backend_cache_size,
				}),
				eth_rpc_config.frontier_sql_backend_pool_size,
				std::num::NonZeroU32::new(eth_rpc_config.frontier_sql_backend_num_ops_timeout),
				storage_override.clone(),
			))
			.unwrap_or_else(|err| panic!("failed creating sql backend: {:?}", err));
			Ok(fc_db::Backend::<Block, C>::Sql(Arc::new(backend)))
		},
	}
}
