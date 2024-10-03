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

//! Service and service factory implementation. Specialized wrapper over substrate service.

pub mod frontier;

mod instant_finalize;

use fc_consensus::FrontierBlockImport;
use fc_rpc::StorageOverride;
use grandpa::{GrandpaBlockImport, SharedVoterState};
pub use harmonie_runtime::RuntimeApi as HarmonieRuntimeApi;
use sc_consensus_babe::{BabeBlockImport, BabeLink, BabeWorkerHandle, SlotProportion};
use sc_consensus_slots::BackoffAuthoringOnFinalizedHeadLagging;
use sc_rpc::SubscriptionTaskExecutor;
use sc_service::WarpSyncParams;
use sc_transaction_pool_api::OffchainTransactionPoolFactory;

// std
use futures::StreamExt;
use std::{
	collections::BTreeMap,
	sync::{Arc, Mutex}, time::Duration,
};
// crates.io
use futures::FutureExt;
// darwinia
use allfeat_primitives::*;
// polkadot-sdk
use sc_client_api::{Backend, BlockBackend};
use sc_network::{
	event::Event, Multiaddr, NetworkEventStream, NetworkWorker,
};
use sp_runtime::traits::NumberFor;

/// The minimum period of blocks on which justifications will be
/// imported and generated.
const GRANDPA_JUSTIFICATION_PERIOD: u32 = 512;

#[cfg(all(feature = "runtime-benchmarks", feature = "evm-tracing"))]
type HostFunctions = (
	frame_benchmarking::benchmarking::HostFunctions,
	moonbeam_primitives_ext::moonbeam_ext::HostFunctions,
	sp_io::SubstrateHostFunctions,
);
#[cfg(all(feature = "runtime-benchmarks", not(feature = "evm-tracing")))]
type HostFunctions =
	(frame_benchmarking::benchmarking::HostFunctions, sp_io::SubstrateHostFunctions);
#[cfg(all(not(feature = "runtime-benchmarks"), feature = "evm-tracing"))]
type HostFunctions =
	(moonbeam_primitives_ext::moonbeam_ext::HostFunctions, sp_io::SubstrateHostFunctions);
#[cfg(not(any(feature = "evm-tracing", feature = "runtime-benchmarks")))]
type HostFunctions = sp_io::SubstrateHostFunctions;

/// Full client backend type.
type FullBackend = sc_service::TFullBackend<Block>;
/// Full client type.
type FullClient<RuntimeApi> =
	sc_service::TFullClient<Block, RuntimeApi, sc_executor::WasmExecutor<HostFunctions>>;
type FullSelectChain = sc_consensus::LongestChain<FullBackend, Block>;
type FullGrandpaBlockImport<RA> =
	grandpa::GrandpaBlockImport<FullBackend, Block, FullClient<RA>, FullSelectChain>;
type GrandpaLinkHalf<RA> = grandpa::LinkHalf<Block, FullClient<RA>, FullSelectChain>;

type Service<RuntimeApi> = sc_service::PartialComponents<
	FullClient<RuntimeApi>,
	FullBackend,
	FullSelectChain,
	sc_consensus::DefaultImportQueue<Block>,
	sc_transaction_pool::FullPool<Block, FullClient<RuntimeApi>>,
	(
		(
			BabeBlockImport<
				Block,
				FullClient<RuntimeApi>,
				FrontierBlockImport<
					Block,
					FullGrandpaBlockImport<RuntimeApi>,
					FullClient<RuntimeApi>
				>
			>,
			GrandpaLinkHalf<RuntimeApi>,
			sc_consensus_babe::BabeLink<Block>,
			BabeWorkerHandle<Block>,
		),
		fc_db::Backend<Block, FullClient<RuntimeApi>>,
		Arc<dyn StorageOverride<Block>>,
		Option<fc_rpc_core::types::FilterPool>,
		fc_rpc_core::types::FeeHistoryCache,
		fc_rpc_core::types::FeeHistoryCacheLimit,
		Option<sc_telemetry::Telemetry>,
		Option<sc_telemetry::TelemetryWorkerHandle>,
	),
>;

/// Can be called for a `Configuration` to check if it is the specific network.
pub trait IdentifyVariant {
	/// Get spec id.
	fn id(&self) -> &str;

	/// Returns if this is a configuration for the `Crab` network.
	fn is_harmonie(&self) -> bool {
		self.id().starts_with("harmonie")
	}

	/// Returns true if this configuration is for a development network.
	fn is_dev(&self) -> bool {
		// Fulfill Polkadot.JS metadata upgrade requirements.
		self.id().ends_with("-d")
	}
}
impl IdentifyVariant for Box<dyn sc_service::ChainSpec> {
	fn id(&self) -> &str {
		sc_service::ChainSpec::id(&**self)
	}
}

/// A set of APIs that allfeat-like runtimes must implement.
pub trait RuntimeApiCollection:
	fp_rpc::ConvertTransactionRuntimeApi<Block>
	+ fp_rpc::EthereumRuntimeRPCApi<Block>
	// + moonbeam_rpc_primitives_debug::DebugRuntimeApi<Block>
	+ pallet_transaction_payment_rpc::TransactionPaymentRuntimeApi<Block, Balance>
	+ sp_api::ApiExt<Block>
	+ sp_api::Metadata<Block>
	+ sp_block_builder::BlockBuilder<Block>
	+ sp_consensus_babe::BabeApi<Block>
	+ sp_consensus_grandpa::GrandpaApi<Block>
	+ sp_offchain::OffchainWorkerApi<Block>
	+ sp_session::SessionKeys<Block>
	+ sp_transaction_pool::runtime_api::TaggedTransactionQueue<Block>
	+ substrate_frame_rpc_system::AccountNonceApi<Block, AccountId, Nonce>
	+ sp_authority_discovery::AuthorityDiscoveryApi<Block>
{
}
impl<Api> RuntimeApiCollection for Api where
	Api: fp_rpc::ConvertTransactionRuntimeApi<Block>
		+ fp_rpc::EthereumRuntimeRPCApi<Block>
		// + moonbeam_rpc_primitives_debug::DebugRuntimeApi<Block>
		+ pallet_transaction_payment_rpc::TransactionPaymentRuntimeApi<Block, Balance>
		+ sp_api::ApiExt<Block>
		+ sp_api::Metadata<Block>
		+ sp_block_builder::BlockBuilder<Block>
		+ sp_consensus_babe::BabeApi<Block>
		+ sp_consensus_grandpa::GrandpaApi<Block>
		+ sp_offchain::OffchainWorkerApi<Block>
		+ sp_session::SessionKeys<Block>
		+ sp_transaction_pool::runtime_api::TaggedTransactionQueue<Block>
		+ substrate_frame_rpc_system::AccountNonceApi<Block, AccountId, Nonce>
		+ sp_authority_discovery::AuthorityDiscoveryApi<Block>
{
}

/// Starts a `ServiceBuilder` for a full service.
///
/// Use this macro if you don't actually need the full service, but just the builder in order to
/// be able to perform chain operations.
pub fn new_partial<RuntimeApi>(
	config: &sc_service::Configuration,
	eth_rpc_config: &crate::cli::EthRpcConfig,
) -> Result<Service<RuntimeApi>, sc_service::Error>
where
	RuntimeApi: 'static + Send + Sync + sp_api::ConstructRuntimeApi<Block, FullClient<RuntimeApi>>,
	RuntimeApi::RuntimeApi: RuntimeApiCollection,
{
	let telemetry = config
		.telemetry_endpoints
		.clone()
		.filter(|x| !x.is_empty())
		.map(|endpoints| -> Result<_, sc_telemetry::Error> {
			let worker = sc_telemetry::TelemetryWorker::new(16)?;
			let telemetry = worker.handle().new_telemetry(endpoints);
			Ok((worker, telemetry))
		})
		.transpose()?;
	let executor = sc_service::new_wasm_executor::<HostFunctions>(config);
	let (client, backend, keystore_container, task_manager) =
		sc_service::new_full_parts::<Block, RuntimeApi, _>(
			config,
			telemetry.as_ref().map(|(_, telemetry)| telemetry.handle()),
			executor,
		)?;
	let client = Arc::new(client);
	let telemetry_worker_handle = telemetry.as_ref().map(|(worker, _)| worker.handle());
	let telemetry = telemetry.map(|(worker, telemetry)| {
		task_manager.spawn_handle().spawn("telemetry", None, worker.run());
		telemetry
	});
	let transaction_pool = sc_transaction_pool::BasicPool::new_full(
		config.transaction_pool.clone(),
		config.role.is_authority().into(),
		config.prometheus_registry(),
		task_manager.spawn_essential_handle(),
		client.clone(),
	);
	let select_chain = sc_consensus::LongestChain::new(backend.clone());
	let (grandpa_block_import, grandpa_link) = grandpa::block_import(
		client.clone(),
		GRANDPA_JUSTIFICATION_PERIOD,
		&client,
		select_chain.clone(),
		telemetry.as_ref().map(|x| x.handle()),
	)?;
	let justification_import = grandpa_block_import.clone();

	let frontier_block_import = FrontierBlockImport::new(grandpa_block_import.clone(), client.clone());

	let (block_import, babe_link) = sc_consensus_babe::block_import(
		sc_consensus_babe::configuration(&*client)?,
		frontier_block_import.clone(),
		client.clone(),
	)?;

	let slot_duration = babe_link.config().slot_duration();
	let (import_queue, babe_worker_handle) =
		sc_consensus_babe::import_queue(sc_consensus_babe::ImportQueueParams {
			link: babe_link.clone(),
			block_import: frontier_block_import,
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
	let storage_override = Arc::new(fc_rpc::RuntimeApiStorageOverride::<Block, _>::new(client.clone()));
	let frontier_backend = frontier::backend(client.clone(), config, storage_override.clone(), eth_rpc_config.clone())?;
	let filter_pool = Some(Arc::new(Mutex::new(BTreeMap::new())));
	let fee_history_cache = Arc::new(Mutex::new(BTreeMap::new()));
	let fee_history_cache_limit = eth_rpc_config.fee_history_limit;

	Ok(sc_service::PartialComponents {
		backend: backend.clone(),
		client,
		import_queue,
		keystore_container,
		task_manager,
		transaction_pool,
		select_chain: sc_consensus::LongestChain::new(backend),
		other: (
			import_setup,
			frontier_backend,
			storage_override,
			filter_pool,
			fee_history_cache,
			fee_history_cache_limit,
			telemetry,
			telemetry_worker_handle,
		),
	})
}

/// Start a node with the given chain `Configuration`.
///
/// This is the actual implementation that is abstract over the executor and the runtime api.
#[allow(clippy::too_many_arguments)]
#[sc_tracing::logging::prefix_logs_with("Blockchain")]
async fn start_node_impl<RuntimeApi, SC, NB>(
	mut config: sc_service::Configuration,
	start_consensus: SC,
	no_hardware_benchmarks: bool,
	storage_monitor: sc_storage_monitor::StorageMonitorParams,
	eth_rpc_config: &crate::cli::EthRpcConfig,
) -> sc_service::error::Result<(sc_service::TaskManager, Arc<FullClient<RuntimeApi>>)>
where
	RuntimeApi: 'static + Send + Sync + sp_api::ConstructRuntimeApi<Block, FullClient<RuntimeApi>>,
	RuntimeApi::RuntimeApi: RuntimeApiCollection,
	NB: sc_network::NetworkBackend<Block, Hash>,
	SC: FnOnce(
		Arc<FullClient<RuntimeApi>>,
		Arc<dyn sc_network::service::traits::NetworkService>,
		FullSelectChain,
		// Babe related
		(
			BabeLink<Block>,
			BabeBlockImport<
				Block,
				FullClient<RuntimeApi>,
				FrontierBlockImport<
					Block,
					FullGrandpaBlockImport<RuntimeApi>,
					FullClient<RuntimeApi>
				>
			>,
			bool,
			Option<BackoffAuthoringOnFinalizedHeadLagging<NumberFor<Block>>>
		),
		// Grandpa related
		Option<&substrate_prometheus_endpoint::Registry>,
		Option<sc_telemetry::TelemetryHandle>,
		&sc_service::TaskManager,
		Arc<sc_transaction_pool::FullPool<Block, FullClient<RuntimeApi>>>,
		Arc<sc_network_sync::SyncingService<Block>>,
		sp_keystore::KeystorePtr,
		bool,
		Vec<Multiaddr>,
	) -> Result<(), sc_service::Error>,
{
	let sc_service::PartialComponents {
		backend,
		client,
		import_queue,
		keystore_container,
		mut task_manager,
		transaction_pool,
		select_chain,
		other:
			(
				import_setup,
				frontier_backend,
				storage_override,
				filter_pool,
				fee_history_cache,
				fee_history_cache_limit,
				mut telemetry,
				_,
			),
	} = new_partial::<RuntimeApi>(&config, eth_rpc_config)?;
	let database_path = config.database.path().map(|p| p.to_path_buf());
	let hwbench = (!no_hardware_benchmarks)
		.then_some(database_path.as_ref().map(|p| {
			let _ = std::fs::create_dir_all(p);

			sc_sysinfo::gather_hwbench(Some(p))
		}))
		.flatten();
	let frontier_backend = Arc::new(frontier_backend);
	let validator = config.role.is_authority();
	let prometheus_registry = config.prometheus_registry().cloned();
	let mut net_config =
		sc_network::config::FullNetworkConfiguration::<Block, Hash, NB>::new(&config.network);
		

	let metrics = NB::register_notification_metrics(
		config.prometheus_config.as_ref().map(|cfg| &cfg.registry),
	);
	let warp_sync = Arc::new(grandpa::warp_proof::NetworkProvider::new(
		backend.clone(),
		import_setup.1.shared_authority_set().clone(),
		Vec::default(),
	));

	let peer_store_handle = net_config.peer_store_handle();
	let grandpa_protocol_name = grandpa::protocol_standard_name(
		&client
			.block_hash(0u32.into())
			.ok()
			.flatten()
			.expect("Genesis block exists; qed"),
		&config.chain_spec,
	);
	let (grandpa_protocol_config, grandpa_notification_service) =
		grandpa::grandpa_peers_set_config::<_, NB>(
			grandpa_protocol_name.clone(),
			metrics.clone(),
			Arc::clone(&peer_store_handle),
		);
	net_config.add_notification_protocol(grandpa_protocol_config);


	let auth_disc_publish_non_global_ips = config.network.allow_non_globals_in_dht;
	let auth_disc_public_addresses = config.network.public_addresses.clone();
	let force_authoring = config.force_authoring;
	let backoff_authoring_blocks =
		Some(sc_consensus_slots::BackoffAuthoringOnFinalizedHeadLagging::default());
	let role = config.role.clone();
	let name = config.network.node_name.clone();

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
			metrics,
		})?;

	if config.offchain_worker.enabled {
		task_manager.spawn_handle().spawn(
			"offchain-workers-runner",
			"offchain-work",
			sc_offchain::OffchainWorkers::new(sc_offchain::OffchainWorkerOptions {
				runtime_api_provider: client.clone(),
				keystore: Some(keystore_container.keystore()),
				offchain_db: backend.offchain_storage(),
				transaction_pool: Some(
					sc_transaction_pool_api::OffchainTransactionPoolFactory::new(
						transaction_pool.clone(),
					),
				),
				network_provider: Arc::new(network.clone()),
				is_validator: role.is_authority(),
				enable_http_requests: false,
				custom_extensions: move |_| Vec::new(),
			})
			.run(client.clone(), task_manager.spawn_handle())
			.boxed(),
		);
	}

	let block_data_cache = Arc::new(fc_rpc::EthBlockDataCacheTask::new(
		task_manager.spawn_handle(),
		storage_override.clone(),
		eth_rpc_config.eth_log_block_cache,
		eth_rpc_config.eth_statuses_cache,
		prometheus_registry.clone(),
	));
	let pubsub_notification_sinks: fc_mapping_sync::EthereumBlockNotificationSinks<
		fc_mapping_sync::EthereumBlockNotification<Block>,
	> = Default::default();
	let pubsub_notification_sinks = Arc::new(pubsub_notification_sinks);
	// for ethereum-compatibility rpc.
	config.rpc_id_provider = Some(Box::new(fc_rpc::EthereumSubIdProvider));

	let rpc_builder = {
		let (_, grandpa_link, _, babe_worker_handle) = &import_setup;

		let babe_worker_handle = babe_worker_handle.clone();
		let justification_stream = grandpa_link.justification_stream();
		let shared_authority_set = grandpa_link.shared_authority_set().clone();
		let shared_voter_state = grandpa::SharedVoterState::empty();
		let finality_proof_provider = grandpa::FinalityProofProvider::new_for_service(
			backend.clone(),
			Some(shared_authority_set.clone()),
		);

		let client = client.clone();
		let pool = transaction_pool.clone();
		let network = network.clone();
		let select_chain = select_chain.clone();
		let keystore = keystore_container.keystore();
		let chain_spec = config.chain_spec.cloned_box();
		let filter_pool = filter_pool.clone();
		let pubsub_notification_sinks = pubsub_notification_sinks.clone();
		let frontier_backend = frontier_backend.clone();
		let storage_override = storage_override.clone();
		let fee_history_cache = fee_history_cache.clone();
		let max_past_logs = eth_rpc_config.max_past_logs;
		let validator = config.role.is_authority();
		let sync_service = sync_service.clone();
		let slot_duration = import_setup.2.clone().config().slot_duration();
		let pending_create_inherent_data_providers = move |_, ()| async move {
			let timestamp = sp_timestamp::InherentDataProvider::from_system_time();
			let slot =
				sp_consensus_babe::inherents::InherentDataProvider::from_timestamp_and_slot_duration(
					*timestamp,
					slot_duration,
				);
			Ok((slot, timestamp))
		};

		Box::new(move |deny_unsafe, subscription_executor: SubscriptionTaskExecutor| {
			let deps = crate::rpc::FullDeps {
				client: client.clone(),
				pool: pool.clone(),
				babe: crate::rpc::BabeDeps {
					keystore: keystore.clone(),
					babe_worker_handle: babe_worker_handle.clone(),
				},
				grandpa: crate::rpc::GrandpaDeps {
					shared_voter_state: shared_voter_state.clone(),
					shared_authority_set: shared_authority_set.clone(),
					justification_stream: justification_stream.clone(),
					subscription_executor: subscription_executor.clone(),
					finality_provider: finality_proof_provider.clone(),
				},
				graph: pool.pool().clone(),
				deny_unsafe,
				select_chain: select_chain.clone(),
				chain_spec: chain_spec.cloned_box(),
				is_authority: validator,
				network: network.clone(),
				sync: sync_service.clone(),
				filter_pool: filter_pool.clone(),
				frontier_backend: match &*frontier_backend {
					fc_db::Backend::KeyValue(bd) => bd.clone(),
					fc_db::Backend::Sql(bd) => bd.clone(),
				},
				max_past_logs,
				fee_history_cache: fee_history_cache.clone(),
				fee_history_cache_limit,
				storage_override: storage_override.clone(),
				block_data_cache: block_data_cache.clone(),
				forced_parent_hashes: None,
				pending_create_inherent_data_providers,
			};

			crate::rpc::create_full::<_, _, _, _, _, crate::rpc::DefaultEthConfig<_, _>, _>(
				deps,
				subscription_executor,
				pubsub_notification_sinks.clone(),
				// None,
			)
			.map_err(Into::into)
		})
	};

	let enable_grandpa = !config.disable_grandpa;
	sc_service::spawn_tasks(sc_service::SpawnTasksParams {
		rpc_builder,
		client: client.clone(),
		transaction_pool: transaction_pool.clone(),
		task_manager: &mut task_manager,
		config,
		keystore: keystore_container.keystore(),
		backend: backend.clone(),
		network: network.clone(),
		sync_service: sync_service.clone(),
		system_rpc_tx,
		tx_handler_controller,
		telemetry: telemetry.as_mut(),
	})?;

	frontier::spawn_tasks(
		&task_manager,
		client.clone(),
		backend,
		frontier_backend,
		filter_pool,
		storage_override,
		fee_history_cache,
		fee_history_cache_limit,
		sync_service.clone(),
		pubsub_notification_sinks,
		eth_rpc_config.clone(),
		prometheus_registry.clone(),
	);

	if let Some(hwbench) = hwbench {
		sc_sysinfo::print_hwbench(&hwbench);

		// Here you can check whether the hardware meets your chains' requirements. Putting a link
		// in there and swapping out the requirements for your own are probably a good idea. The
		// requirements for a para-chain are dictated by its relay-chain.
		if let Err(e) =
			frame_benchmarking_cli::SUBSTRATE_REFERENCE_HARDWARE.check_hardware(&hwbench)
		{
			log::warn!(
				"⚠️  The hardware does not meet the minimal requirements {e} for role 'Validator'.",
			);
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
	if let Some(database_path) = database_path {
		sc_storage_monitor::StorageMonitorService::try_spawn(
			storage_monitor,
			database_path,
			&task_manager.spawn_essential_handle(),
		)
		.map_err(|e| sc_service::Error::Application(e.into()))?;
	}

	let (block_import, grandpa_link, babe_link, _) = import_setup;

	if validator {
		start_consensus(
			client.clone(),
			network.clone(),
			select_chain,
			(babe_link, block_import, force_authoring, backoff_authoring_blocks),
			prometheus_registry.as_ref(),
			telemetry.as_ref().map(|t| t.handle()),
			&task_manager,
			transaction_pool.clone(),
			sync_service.clone(),
			keystore_container.keystore(),
			auth_disc_publish_non_global_ips,
			auth_disc_public_addresses,
		)?;
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

	network_starter.start_network();

	Ok((task_manager, client))
}

/// Start a node.
pub async fn start_node<RuntimeApi>(
	config: sc_service::Configuration,
	no_hardware_benchmarks: bool,
	storage_monitor: sc_storage_monitor::StorageMonitorParams,
	eth_rpc_config: &crate::cli::EthRpcConfig,
) -> sc_service::error::Result<(sc_service::TaskManager, Arc<FullClient<RuntimeApi>>)>
where
	RuntimeApi: sp_api::ConstructRuntimeApi<Block, FullClient<RuntimeApi>> + Send + Sync + 'static,
	RuntimeApi::RuntimeApi: RuntimeApiCollection,
	RuntimeApi::RuntimeApi: sp_consensus_babe::BabeApi<Block>,
{
	match config.network.network_backend {
		sc_network::config::NetworkBackendType::Libp2p =>
			start_node_impl::<RuntimeApi, _, NetworkWorker<Block, Hash>>(
				config,
				|client,
				 network,
				 select_chain,
				 (babe_link, block_import, force_authoring, backoff_authoring_blocks),
				 prometheus_registry,
				 telemetry,
				 task_manager,
				 transaction_pool,
				 sync_oracle,
				 keystore,
				 publish_non_global_ips,
				 public_addresses| {
					let authority_discovery_role =
						sc_authority_discovery::Role::PublishAndDiscover(keystore.clone());
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
								publish_non_global_ips,
								public_addresses,
								..Default::default()
							},
							client.clone(),
							Arc::new(network.clone()),
							Box::pin(dht_event_stream),
							authority_discovery_role,
							prometheus_registry.cloned(),
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
						prometheus_registry,
						telemetry.clone(),
					);
					let client_clone = client.clone();
					let slot_duration = babe_link.config().slot_duration();
					let babe_config = sc_consensus_babe::BabeParams {
						keystore: keystore.clone(),
						client: client.clone(),
						select_chain,
						env: proposer,
						block_import,
						sync_oracle: sync_oracle.clone(),
						justification_sync_link: sync_oracle.clone(),
						create_inherent_data_providers: move |parent, ()| {
							let client_clone = client_clone.clone();
							async move {
								let timestamp =
									sp_timestamp::InherentDataProvider::from_system_time();

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
						telemetry,
					};

					let babe = sc_consensus_babe::start_babe(babe_config)?;
					task_manager.spawn_essential_handle().spawn_blocking(
						"babe-proposer",
						Some("block-authoring"),
						babe,
					);

					Ok(())
				},
				no_hardware_benchmarks,
				storage_monitor,
				eth_rpc_config,
			)
			.await,
		sc_network::config::NetworkBackendType::Litep2p => {
			todo!()
		},
	}
}
