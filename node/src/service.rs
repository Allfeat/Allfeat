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

#![warn(unused_extern_crates)]

//! Service implementation. Specialized wrapper over substrate service.

use crate::{
	client::{BaseRuntimeApiCollection, FullClient, RuntimeApiCollection},
	eth::{self, db_config_dir, EthCompatRuntimeApiCollection, EthConfiguration, FrontierBackend, FrontierBackendType},
};
use allfeat_runtime::TransactionConverter;
use fc_consensus::FrontierBlockImport;
use fc_rpc::StorageOverrideHandler;
use frame_benchmarking_cli::SUBSTRATE_REFERENCE_HARDWARE;
use futures::prelude::*;
use grandpa::{BlockNumberOps, SharedVoterState};
use allfeat_primitives::Block;
use harmonie_runtime::RuntimeApi;
use sc_client_api::BlockBackend;
use sc_consensus::BasicQueue;
use sc_consensus_babe::{BabeWorkerHandle, SlotProportion};
use sc_network::{event::Event, NetworkEventStream};
use sc_rpc::SubscriptionTaskExecutor;
use sc_service::{
	config::Configuration, error::Error as ServiceError, TaskManager, WarpSyncParams,
};
use sc_telemetry::{Telemetry, TelemetryWorker};
use sc_transaction_pool_api::OffchainTransactionPoolFactory;
use sp_api::ConstructRuntimeApi;
use sp_core::H256;
use sp_core::U256;
use std::{
	collections::BTreeMap, path::Path, sync::{Arc, Mutex}, time::Duration
};
use sp_runtime::traits::{Block as BlockT, NumberFor};
use sc_client_api::Backend as BackendT;

use crate::rpc;

pub type FullBackend<B> = sc_service::TFullBackend<B>;

pub type Backend = FullBackend<Block>;
pub type Client = FullClient<Block, RuntimeApi>;

type FullSelectChain<B> = sc_consensus::LongestChain<FullBackend<B>, B>;
type GrandpaBlockImport<B, C> = grandpa::GrandpaBlockImport<FullBackend<B>, B, C, FullSelectChain<B>>;
type GrandpaLinkHalf<B, C> = grandpa::LinkHalf<B, C, FullSelectChain<B>>;

/// The minimum period of blocks on which justifications will be
/// imported and generated.
const GRANDPA_JUSTIFICATION_PERIOD: u32 = 512;

/// Creates a new partial node.
pub fn new_partial<B, RA>(
	config: &Configuration,
	eth_rpc_config: &EthConfiguration,
) -> Result<
	sc_service::PartialComponents<
		FullClient<B, RA>,
		FullBackend<B>,
		FullSelectChain<B>,
		sc_consensus::DefaultImportQueue<B>,
		sc_transaction_pool::FullPool<B, FullClient<B, RA>>,
		(
			(
				sc_consensus_babe::BabeBlockImport<
					B,
					FullClient<B, RA>,
					GrandpaBlockImport<B, FullClient<B, RA>>,
				>,
				GrandpaLinkHalf<B, FullClient<B, RA>>,
				sc_consensus_babe::BabeLink<B>,
				BabeWorkerHandle<B>,
			),
			FrontierBackend<B, FullClient<B, RA>>,
			Option<fc_rpc_core::types::FilterPool>,
			fc_rpc_core::types::FeeHistoryCache,
			fc_rpc_core::types::FeeHistoryCacheLimit,
			Option<Telemetry>,
			Option<sc_telemetry::TelemetryWorkerHandle>,
		),
	>,
	ServiceError,
>
where
	B: BlockT<Hash = H256>,
	NumberFor<B>: BlockNumberOps,
	RA: ConstructRuntimeApi<B, FullClient<B, RA>>,
	RA: Send + Sync + 'static,
	RA::RuntimeApi: BaseRuntimeApiCollection<B> + EthCompatRuntimeApiCollection<B>,
{
	let telemetry = config
		.telemetry_endpoints
		.clone()
		.filter(|x| !x.is_empty())
		.map(|endpoints| -> Result<_, sc_telemetry::Error> {
			let worker = TelemetryWorker::new(16)?;
			let telemetry = worker.handle().new_telemetry(endpoints);
			Ok((worker, telemetry))
		})
		.transpose()?;

	let executor = sc_service::new_wasm_executor(config);

	let (client, backend, keystore_container, task_manager) =
		sc_service::new_full_parts::<B, RA, _>(
			config,
			telemetry.as_ref().map(|(_, telemetry)| telemetry.handle()),
			executor,
		)?;
	let telemetry_worker_handle = telemetry.as_ref().map(|(worker, _)| worker.handle());
	let telemetry = telemetry.map(|(worker, telemetry)| {
		task_manager.spawn_handle().spawn("telemetry", None, worker.run());
		telemetry
	});
	let client = Arc::new(client);
	let select_chain = sc_consensus::LongestChain::new(backend.clone());
	let transaction_pool = sc_transaction_pool::BasicPool::new_full(
		config.transaction_pool.clone(),
		config.role.is_authority().into(),
		config.prometheus_registry(),
		task_manager.spawn_essential_handle(),
		client.clone(),
	);

	let (grandpa_block_import, grandpa_link) = grandpa::block_import(
		client.clone(),
		GRANDPA_JUSTIFICATION_PERIOD,
		&client,
		select_chain.clone(),
		telemetry.as_ref().map(|x| x.handle()),
	)?;
	let justification_import = grandpa_block_import.clone();
	let (block_import, babe_link) = sc_consensus_babe::block_import(
		sc_consensus_babe::configuration(&*client)?,
		grandpa_block_import.clone(),
		client.clone(),
	)?;
	let frontier_block_import = FrontierBlockImport::new(grandpa_block_import, client.clone());
	let slot_duration = babe_link.config().slot_duration();
	let (import_queue, babe_worker_handle) =
		sc_consensus_babe::import_queue(sc_consensus_babe::ImportQueueParams {
			link: babe_link.clone(),
			block_import: frontier_block_import.clone(),
			justification_import: Some(Box::new(justification_import)),
			client: client.clone(),
			select_chain: select_chain.clone(),
			create_inherent_data_providers: move |_, ()| async move {
				let timestamp = sp_timestamp::InherentDataProvider::from_system_time();
				let slot =
					sp_consensus_babe::inherents::InherentDataProvider::from_timestamp_and_slot_duration(
						*timestamp,
						slot_duration,
					);
				Ok((slot, timestamp))
			},
			spawner: &task_manager.spawn_essential_handle(),
			registry: config.prometheus_registry(),
			telemetry: telemetry.as_ref().map(|x| x.handle()),
			offchain_tx_pool_factory: OffchainTransactionPoolFactory::new(transaction_pool.clone()),
		})?;
	let import_setup = (block_import, grandpa_link, babe_link, babe_worker_handle);

	// Frontier stuffs.
	let storage_override = Arc::new(StorageOverrideHandler::<B, _, _>::new(client.clone()));
	let frontier_backend = match eth_rpc_config.frontier_backend_type {
		FrontierBackendType::KeyValue => FrontierBackend::KeyValue(Arc::new(fc_db::kv::Backend::open(
			Arc::clone(&client),
			&config.database,
			&db_config_dir(config),
		)?)),
		FrontierBackendType::Sql => {
			let db_path = db_config_dir(config).join("sql");
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
			FrontierBackend::Sql(Arc::new(backend))
		}
	};
	let filter_pool = Some(Arc::new(Mutex::new(BTreeMap::new())));
	let fee_history_cache = Arc::new(Mutex::new(BTreeMap::new()));
	let fee_history_cache_limit = eth_rpc_config.fee_history_limit;

	Ok(sc_service::PartialComponents {
		client,
		backend,
		task_manager,
		keystore_container,
		select_chain,
		import_queue,
		transaction_pool,
		other: (
			import_setup,
			frontier_backend,
			filter_pool,
			fee_history_cache,
			fee_history_cache_limit,
			telemetry,
			telemetry_worker_handle,
		),
	})
}

/// Builds a new service for a full client.
pub async fn new_full<B, RA, NB>(
	mut config: Configuration,
	eth_config: EthConfiguration,
	disable_hardware_benchmarks: bool,
	with_startup_data: impl FnOnce(
		&sc_consensus_babe::BabeBlockImport<
			B,
			FullClient<B, RA>,
			GrandpaBlockImport<B, FullClient<B, RA>>,
		>,
		&sc_consensus_babe::BabeLink<B>,
	),
) -> Result<TaskManager, ServiceError>
where
	B: BlockT<Hash = H256> + std::marker::Unpin,
	NumberFor<B>: BlockNumberOps,
	<B as BlockT>::Header: Unpin,
	RA: ConstructRuntimeApi<B, FullClient<B, RA>>,
	RA: Send + Sync + 'static,
	RA::RuntimeApi: RuntimeApiCollection<B>,
	NB: sc_network::NetworkBackend<B, <B as BlockT>::Hash>,
{
	let hwbench = (!disable_hardware_benchmarks)
		.then_some(config.database.path().map(|database_path| {
			let _ = std::fs::create_dir_all(&database_path);
			sc_sysinfo::gather_hwbench(Some(database_path))
		}))
		.flatten();

	let sc_service::PartialComponents {
		client,
		backend,
		mut task_manager,
		import_queue,
		keystore_container,
		select_chain,
		transaction_pool,
		other:
			(
				import_setup,
				frontier_backend,
				filter_pool,
				fee_history_cache,
				fee_history_cache_limit,
				mut telemetry,
				_,
			),
	} = new_partial(&config, &eth_config)?;

	let metrics = NB::register_notification_metrics(
		config.prometheus_config.as_ref().map(|cfg| &cfg.registry),
	);
	let mut net_config =
		sc_network::config::FullNetworkConfiguration::<_, _, NB>::new(&config.network);

	let peer_store_handle = net_config.peer_store_handle();
	let grandpa_protocol_name = grandpa::protocol_standard_name(
		&client.block_hash(0u32.into()).ok().flatten().expect("Genesis block exists; qed"),
		&config.chain_spec,
	);
	let (grandpa_protocol_config, grandpa_notification_service) =
		grandpa::grandpa_peers_set_config::<_, NB>(
			grandpa_protocol_name.clone(),
			metrics.clone(),
			Arc::clone(&peer_store_handle),
		);
	net_config.add_notification_protocol(grandpa_protocol_config);

	let warp_sync = Arc::new(grandpa::warp_proof::NetworkProvider::new(
		backend.clone(),
		import_setup.1.shared_authority_set().clone(),
		Vec::default(),
	));

	let (network, system_rpc_tx, tx_handler_controller, network_starter, sync_service) =
		sc_service::build_network(sc_service::BuildNetworkParams {
			config: &config,
			net_config,
			client: client.clone(),
			transaction_pool: transaction_pool.clone(),
			spawn_handle: task_manager.spawn_handle(),
			import_queue,
			block_announce_validator_builder: None,
			warp_sync_params: Some(WarpSyncParams::WithProvider(warp_sync)),
			block_relay: None,
			metrics
		})?;

	let shared_voter_state = grandpa::SharedVoterState::empty();
	let auth_disc_publish_non_global_ips = config.network.allow_non_globals_in_dht;
	let auth_disc_public_addresses = config.network.public_addresses.clone();

	let role = config.role.clone();
	let force_authoring = config.force_authoring;
	let backoff_authoring_blocks =
		Some(sc_consensus_slots::BackoffAuthoringOnFinalizedHeadLagging::default());
	let name = config.network.node_name.clone();
	let enable_grandpa = !config.disable_grandpa;
	let prometheus_registry = config.prometheus_registry().cloned();
	let enable_offchain_worker = config.offchain_worker.enabled;
	let frontier_backend = Arc::new(frontier_backend);

	// for ethereum-compatibility rpc.
	config.rpc_id_provider = Some(Box::new(fc_rpc::EthereumSubIdProvider));

	// Sinks for pubsub notifications.
	// Everytime a new subscription is created, a new mpsc channel is added to the sink pool.
	// The MappingSyncWorker sends through the channel on block import and the subscription emits a
	// notification to the subscriber on receiving a message through this channel. This way we avoid
	// race conditions when using native substrate block import notification stream.
	let pubsub_notification_sinks: fc_mapping_sync::EthereumBlockNotificationSinks<
		fc_mapping_sync::EthereumBlockNotification<B>,
	> = Default::default();
	let pubsub_notification_sinks = Arc::new(pubsub_notification_sinks);
	let storage_override = Arc::new(StorageOverrideHandler::<B, _, _>::new(client.clone()));
	
	eth::spawn_tasks(
		&task_manager,
		client.clone(),
		backend.clone(),
		frontier_backend.clone(),
		filter_pool.clone(),
		storage_override.clone(),
		fee_history_cache.clone(),
		fee_history_cache_limit,
		sync_service.clone(),
		pubsub_notification_sinks.clone(),
	);

	let rpc_builder = {
		let (_, grandpa_link, _, babe_worker_handle) = &import_setup;

		let babe_worker_handle = babe_worker_handle.clone();
		let justification_stream = grandpa_link.justification_stream();
		let shared_authority_set = grandpa_link.shared_authority_set().clone();
		let shared_voter_state2 = shared_voter_state.clone();

		let finality_proof_provider = grandpa::FinalityProofProvider::new_for_service(
			backend.clone(),
			Some(shared_authority_set.clone()),
		);
		let execute_gas_limit_multiplier = eth_config.execute_gas_limit_multiplier;

		let network = network.clone();
		let sync_service = sync_service.clone();

		let client = client.clone();
		let pool = transaction_pool.clone();
		let select_chain = select_chain.clone();
		let keystore = keystore_container.keystore();
		let chain_spec = config.chain_spec.cloned_box();

		let is_authority = role.is_authority();
		let enable_dev_signer = eth_config.enable_dev_signer;
		let max_past_logs = eth_config.max_past_logs;
		let filter_pool = filter_pool.clone();
		let frontier_backend = frontier_backend.clone();
		let pubsub_notification_sinks = pubsub_notification_sinks.clone();
		let fee_history_cache = fee_history_cache.clone();
		let block_data_cache = Arc::new(fc_rpc::EthBlockDataCacheTask::new(
			task_manager.spawn_handle(),
			storage_override.clone(),
			eth_config.eth_log_block_cache,
			eth_config.eth_statuses_cache,
			prometheus_registry.clone(),
		));

		let slot_duration = import_setup.2.clone().config().slot_duration();
		let target_gas_price = eth_config.target_gas_price;
		let pending_create_inherent_data_providers = move |_, ()| async move {
			let timestamp = sp_timestamp::InherentDataProvider::from_system_time();
			let slot =
				sp_consensus_babe::inherents::InherentDataProvider::from_timestamp_and_slot_duration(
					*timestamp,
					slot_duration,
				);
			let dynamic_fee = fp_dynamic_fee::InherentDataProvider(U256::from(target_gas_price));
			Ok((slot, timestamp, dynamic_fee))
		};

		Box::new(move |deny_unsafe, subscription_executor: SubscriptionTaskExecutor| {
			let eth_deps = crate::rpc_eth::EthDeps {
				client: client.clone(),
				pool: pool.clone(),
				graph: pool.pool().clone(),
				converter: Some(TransactionConverter::<B>::default()),
				is_authority,
				enable_dev_signer,
				network: network.clone(),
				sync: sync_service.clone(),
				frontier_backend: match &*frontier_backend {
					fc_db::Backend::KeyValue(b) => b.clone(),
					fc_db::Backend::Sql(b) => b.clone(),
				},
				storage_override: storage_override.clone(),
				block_data_cache: block_data_cache.clone(),
				filter_pool: filter_pool.clone(),
				max_past_logs,
				fee_history_cache: fee_history_cache.clone(),
				fee_history_cache_limit,
				execute_gas_limit_multiplier,
				forced_parent_hashes: None,
				pending_create_inherent_data_providers,
			};
			let deps = rpc::FullDeps {
				client: client.clone(),
				pool: pool.clone(),
				select_chain: select_chain.clone(),
				chain_spec: chain_spec.cloned_box(),
				enable_dev_signer,
				deny_unsafe,
				babe: rpc::BabeDeps {
					keystore: keystore.clone(),
					babe_worker_handle: babe_worker_handle.clone(),
				},
				grandpa: rpc::GrandpaDeps {
					shared_voter_state: shared_voter_state2.clone(),
					shared_authority_set: shared_authority_set.clone(),
					justification_stream: justification_stream.clone(),
					subscription_executor: subscription_executor.clone(),
					finality_provider: finality_proof_provider.clone(),
				},
				eth: eth_deps
			};

			rpc::create_full(
				deps,
				subscription_executor,
				pubsub_notification_sinks.clone(),
			)
			.map_err(Into::into)
		})
	};

	sc_service::spawn_tasks(sc_service::SpawnTasksParams {
		config,
		backend: backend.clone(),
		client: client.clone(),
		keystore: keystore_container.keystore(),
		network: network.clone(),
		rpc_builder,
		transaction_pool: transaction_pool.clone(),
		task_manager: &mut task_manager,
		system_rpc_tx,
		tx_handler_controller,
		sync_service: sync_service.clone(),
		telemetry: telemetry.as_mut(),
	})?;

	if let Some(hwbench) = hwbench {
		sc_sysinfo::print_hwbench(&hwbench);
		match SUBSTRATE_REFERENCE_HARDWARE.check_hardware(&hwbench) {
			Err(err) if role.is_authority() => {
				log::warn!(
					"⚠️  The hardware does not meet the minimal requirements {} for role 'Authority'.",
					err
				);
			},
			_ => {},
		}

		if let Some(ref mut telemetry) = telemetry {
			let telemetry_handle = telemetry.handle();
			task_manager.spawn_handle().spawn(
				"telemetry_hwbench",
				None,
				sc_sysinfo::initialize_hwbench_telemetry(telemetry_handle, hwbench),
			);
		}
	}

	let (block_import, grandpa_link, babe_link, _) = import_setup;

	(with_startup_data)(&block_import, &babe_link);

	// Spawn authority discovery and BABE module.
	if role.is_authority() {
		let authority_discovery_role =
			sc_authority_discovery::Role::PublishAndDiscover(keystore_container.keystore());
		let dht_event_stream =
			network.event_stream("authority-discovery").filter_map(|e| async move {
				match e {
					Event::Dht(e) => Some(e),
					_ => None,
				}
			});
		let (authority_discovery_worker, _service) =
			sc_authority_discovery::new_worker_and_service_with_config(
				sc_authority_discovery::WorkerConfig {
					publish_non_global_ips: auth_disc_publish_non_global_ips,
					public_addresses: auth_disc_public_addresses,
					..Default::default()
				},
				client.clone(),
				Arc::new(network.clone()),
				Box::pin(dht_event_stream),
				authority_discovery_role,
				prometheus_registry.clone(),
			);

		task_manager.spawn_handle().spawn(
			"authority-discovery-worker",
			Some("networking"),
			authority_discovery_worker.run(),
		);

		let proposer = sc_basic_authorship::ProposerFactory::new(
			task_manager.spawn_handle(),
			client.clone(),
			transaction_pool.clone(),
			prometheus_registry.as_ref(),
			telemetry.as_ref().map(|x| x.handle()),
		);

		let client_clone = client.clone();
		let slot_duration = babe_link.config().slot_duration();
		let babe_config = sc_consensus_babe::BabeParams {
			keystore: keystore_container.keystore(),
			client: client.clone(),
			select_chain,
			env: proposer,
			block_import,
			sync_oracle: sync_service.clone(),
			justification_sync_link: sync_service.clone(),
			create_inherent_data_providers: move |parent, ()| {
				let client_clone = client_clone.clone();
				async move {
					let timestamp = sp_timestamp::InherentDataProvider::from_system_time();

					let slot =
						sp_consensus_babe::inherents::InherentDataProvider::from_timestamp_and_slot_duration(
							*timestamp,
							slot_duration,
						);

					let storage_proof =
						sp_transaction_storage_proof::registration::new_data_provider(
							&*client_clone,
							&parent,
						)?;

					Ok((slot, timestamp, storage_proof))
				}
			},
			force_authoring,
			backoff_authoring_blocks,
			babe_link,
			block_proposal_slot_portion: SlotProportion::new(2f32 / 3f32),
			max_block_proposal_slot_portion: None,
			telemetry: telemetry.as_ref().map(|x| x.handle()),
		};

		let babe = sc_consensus_babe::start_babe(babe_config)?;
		task_manager.spawn_essential_handle().spawn_blocking(
			"babe-proposer",
			Some("block-authoring"),
			babe,
		);
	}

	if enable_grandpa {
		// if the node isn't actively participating in consensus then it doesn't
		// need a keystore, regardless of which protocol we use below.
		let keystore = if role.is_authority() { Some(keystore_container.keystore()) } else { None };

		let grandpa_config = grandpa::Config {
			// FIXME #1578 make this available through chainspec
			gossip_duration: Duration::from_millis(333),
			justification_generation_period: GRANDPA_JUSTIFICATION_PERIOD,
			name: Some(name),
			observer_enabled: false,
			keystore,
			local_role: role.clone(),
			telemetry: telemetry.as_ref().map(|x| x.handle()),
			protocol_name: grandpa_protocol_name,
		};

		// start the full GRANDPA voter
		// NOTE: non-authorities could run the GRANDPA observer protocol, but at
		// this point the full voter should provide better guarantees of block
		// and vote data availability than the observer. The observer has not
		// been tested extensively yet and having most nodes in a network run it
		// could lead to finality stalls.
		let grandpa_config = grandpa::GrandpaParams {
			config: grandpa_config,
			link: grandpa_link,
			network: network.clone(),
			sync: Arc::new(sync_service),
			notification_service: grandpa_notification_service,
			voting_rule: grandpa::VotingRulesBuilder::default().build(),
			prometheus_registry,
			shared_voter_state: SharedVoterState::empty(),
			telemetry: telemetry.as_ref().map(|x| x.handle()),
			offchain_tx_pool_factory: OffchainTransactionPoolFactory::new(transaction_pool.clone()),
		};

		// the GRANDPA voter task is considered infallible, i.e.
		// if it fails we take down the service with it.
		task_manager.spawn_essential_handle().spawn_blocking(
			"grandpa-voter",
			None,
			grandpa::run_grandpa_voter(grandpa_config)?,
		);
	}

	if enable_offchain_worker {
		task_manager.spawn_handle().spawn(
			"offchain-workers-runner",
			"offchain-work",
			sc_offchain::OffchainWorkers::new(sc_offchain::OffchainWorkerOptions {
				runtime_api_provider: client.clone(),
				keystore: Some(keystore_container.keystore()),
				offchain_db: backend.offchain_storage(),
				transaction_pool: Some(OffchainTransactionPoolFactory::new(transaction_pool)),
				network_provider: Arc::new(network.clone()),
				is_validator: role.is_authority(),
				enable_http_requests: true,
				custom_extensions: |_| vec![],
			})
			.run(client.clone(), task_manager.spawn_handle())
			.boxed(),
		);
	}

	network_starter.start_network();
	Ok(task_manager)
}

pub fn new_chain_ops(
	config: &mut Configuration,
	eth_config: &EthConfiguration,
) -> Result<
	(Arc<Client>, Arc<Backend>, BasicQueue<Block>, TaskManager, FrontierBackend<Block, Client>),
	ServiceError,
> {
	config.keystore = sc_service::config::KeystoreConfig::InMemory;
	let sc_service::PartialComponents {
		client, backend, import_queue, task_manager, other, ..
	} = new_partial(config, eth_config)?;
	Ok((client, backend, import_queue, task_manager, other.1))
}
