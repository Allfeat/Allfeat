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
use std::{collections::BTreeMap, sync::Arc};
// Allfeat
use allfeat_primitives::*;

/// A type representing all RPC extensions.
pub type RpcExtension = jsonrpsee::RpcModule<()>;

/// Extra dependencies for BABE.
pub struct BabeDeps {
	/// A handle to the BABE worker for issuing requests.
	pub babe_worker_handle: sc_consensus_babe::BabeWorkerHandle<Block>,
	/// The keystore that manages the keys of the node.
	pub keystore: sp_keystore::KeystorePtr,
}

/// Extra dependencies for GRANDPA
pub struct GrandpaDeps<BE> {
	/// Voting round info.
	pub shared_voter_state: grandpa::SharedVoterState,
	/// Authority set info.
	pub shared_authority_set:
		grandpa::SharedAuthoritySet<Hash, sp_runtime::traits::NumberFor<Block>>,
	/// Receives notifications about justification events from Grandpa.
	pub justification_stream: grandpa::GrandpaJustificationStream<Block>,
	/// Executor to drive the subscription manager in the Grandpa RPC handler.
	pub subscription_executor: sc_rpc_spec_v2::SubscriptionTaskExecutor,
	/// Finality proof provider.
	pub finality_provider: Arc<grandpa::FinalityProofProvider<BE, Block>>,
}

/// Full client dependencies
pub struct FullDeps<C, P, SC, BE, A: sc_transaction_pool::ChainApi, CIDP> {
	/// The client instance to use.
	pub client: Arc<C>,
	/// Transaction pool instance.
	pub pool: Arc<P>,
	/// Graph pool instance.
	pub graph: Arc<sc_transaction_pool::Pool<A>>,
	/// The SelectChain Strategy
	pub select_chain: SC,
	/// Whether to deny unsafe calls
	pub deny_unsafe: sc_rpc::DenyUnsafe,
	/// BABE specific dependencies.
	pub babe: BabeDeps,
	/// GRANDPA specific dependencies.
	pub grandpa: GrandpaDeps<BE>,
	/// A copy of the chain spec.
	pub chain_spec: Box<dyn sc_chain_spec::ChainSpec>,
	/// The Node authority flag
	pub is_authority: bool,
	/// Network service
	pub network: Arc<dyn sc_network::service::traits::NetworkService>,
	/// Chain syncing service
	pub sync: Arc<sc_network_sync::SyncingService<Block>>,
	/// EthFilterApi pool.
	pub filter_pool: Option<fc_rpc_core::types::FilterPool>,
	/// Backend.
	pub frontier_backend: Arc<dyn fc_api::Backend<Block> + Send + Sync>,
	/// Maximum number of logs in a query.
	pub max_past_logs: u32,
	/// Fee history cache.
	pub fee_history_cache: fc_rpc_core::types::FeeHistoryCache,
	/// Maximum fee history cache size.
	pub fee_history_cache_limit: fc_rpc_core::types::FeeHistoryCacheLimit,
	/// Ethereum data access overrides.
	pub storage_override: Arc<dyn fc_rpc_v2::StorageOverride<Block>>,
	/// Cache for Ethereum block data.
	pub block_data_cache: Arc<fc_rpc_v2::EthBlockDataCacheTask<Block>>,
	/// Mandated parent hashes for a given block hash.
	pub forced_parent_hashes: Option<BTreeMap<sp_core::H256, sp_core::H256>>,
	/// Something that can create the inherent data providers for pending state
	pub pending_create_inherent_data_providers: CIDP,
}

/// Default Ethereum RPC config
pub struct DefaultEthConfig<C, BE>(std::marker::PhantomData<(C, BE)>);
impl<C, BE> fc_rpc_v2::EthConfig<Block, C> for DefaultEthConfig<C, BE>
where
	C: 'static + Sync + Send + sc_client_api::StorageProvider<Block, BE>,
	BE: 'static + sc_client_api::Backend<Block>,
{
	type EstimateGasAdapter = ();
	type RuntimeStorageOverride =
		fc_rpc_v2::frontier_backend_client::SystemAccountId20StorageOverride<Block, C, BE>;
}

/// Instantiate all RPC extensions.
pub fn create_full<C, P, SC, BE, A, EC, CIDP>(
	deps: FullDeps<C, P, SC, BE, A, CIDP>,
	subscription_task_executor: sc_rpc::SubscriptionTaskExecutor,
	pubsub_notification_sinks: Arc<
		fc_mapping_sync::EthereumBlockNotificationSinks<
			fc_mapping_sync::EthereumBlockNotification<Block>,
		>,
	>,
	// maybe_tracing_config: Option<TracingConfig>,
) -> Result<RpcExtension, Box<dyn std::error::Error + Send + Sync>>
where
	BE: 'static + sc_client_api::backend::Backend<Block>,
	BE::State: sc_client_api::backend::StateBackend<Hashing>,
	C: 'static
		+ Send
		+ Sync
		+ sc_client_api::AuxStore
		+ sc_client_api::backend::StorageProvider<Block, BE>
		+ sc_client_api::BlockchainEvents<Block>
		+ sc_client_api::UsageProvider<Block>
		+ sc_client_api::BlockBackend<Block>
		+ sp_api::CallApiAt<Block>
		+ sp_api::ProvideRuntimeApi<Block>
		+ sp_blockchain::HeaderBackend<Block>
		+ sp_blockchain::HeaderMetadata<Block, Error = sp_blockchain::Error>,
	C::Api: fp_rpc::ConvertTransactionRuntimeApi<Block>
		+ fp_rpc::EthereumRuntimeRPCApi<Block>
		+ pallet_transaction_payment_rpc::TransactionPaymentRuntimeApi<Block, Balance>
		+ sp_block_builder::BlockBuilder<Block>
		+ sp_consensus_babe::BabeApi<Block>
		+ substrate_frame_rpc_system::AccountNonceApi<Block, AccountId, Nonce>,
	CIDP: sp_inherents::CreateInherentDataProviders<Block, ()> + Send + 'static,
	SC: sp_consensus::SelectChain<Block> + 'static,
	P: 'static + Sync + Send + sc_transaction_pool_api::TransactionPool<Block = Block>,
	A: 'static + sc_transaction_pool::ChainApi<Block = Block>,
	EC: ,
{
	// frontier
	use fp_rpc::NoTransactionConverter;
	// polkadot-sdk
	use pallet_transaction_payment_rpc::{TransactionPayment, TransactionPaymentApiServer};
	use sc_consensus_babe_rpc::{Babe, BabeApiServer};
	use sc_consensus_grandpa_rpc::{Grandpa, GrandpaApiServer};
	use sc_rpc_spec_v2::chain_spec::{ChainSpec, ChainSpecApiServer};
	use sc_sync_state_rpc::{SyncState, SyncStateApiServer};
	use substrate_frame_rpc_system::{System, SystemApiServer};

	let mut module = RpcExtension::new(());
	let FullDeps {
		client,
		pool,
		babe,
		grandpa,
		select_chain,
		graph,
		deny_unsafe,
		chain_spec,
		is_authority,
		network,
		sync,
		filter_pool,
		frontier_backend,
		max_past_logs,
		fee_history_cache,
		fee_history_cache_limit,
		storage_override,
		block_data_cache,
		forced_parent_hashes,
		pending_create_inherent_data_providers,
	} = deps;
	let BabeDeps { keystore, babe_worker_handle } = babe;
	let GrandpaDeps {
		shared_voter_state,
		shared_authority_set,
		justification_stream,
		subscription_executor,
		finality_provider,
	} = grandpa;

	let chain_name = chain_spec.name().to_string();
	let genesis_hash = client
		.block_hash(0u32.into())
		.ok()
		.flatten()
		.expect("Genesis block exists; qed");
	let properties = chain_spec.properties();
	module.merge(ChainSpec::new(chain_name, genesis_hash, properties).into_rpc())?;
	module.merge(System::new(client.clone(), pool.clone(), deny_unsafe).into_rpc())?;
	module.merge(TransactionPayment::new(client.clone()).into_rpc())?;
	module.merge(
		Babe::new(client.clone(), babe_worker_handle.clone(), keystore, select_chain, deny_unsafe)
			.into_rpc(),
	)?;
	module.merge(
		Grandpa::new(
			subscription_executor,
			shared_authority_set.clone(),
			shared_voter_state,
			justification_stream,
			finality_provider,
		)
		.into_rpc(),
	)?;
	module.merge(
		SyncState::new(chain_spec, client.clone(), shared_authority_set, babe_worker_handle)?
			.into_rpc(),
	)?;
	module.merge(
		Eth::<Block, C, P, _, BE, A, CIDP, EC>::new(
			client.clone(),
			pool.clone(),
			graph.clone(),
			<Option<NoTransactionConverter>>::None,
			sync.clone(),
			Vec::new(),
			storage_override.clone(),
			frontier_backend.clone(),
			is_authority,
			block_data_cache.clone(),
			fee_history_cache,
			fee_history_cache_limit,
			10,
			forced_parent_hashes,
			pending_create_inherent_data_providers,
			None,
		)
		.replace_config::<EC>()
		.into_rpc(),
	)?;

	let tx_pool = TxPool::new(client.clone(), graph.clone());
	if let Some(filter_pool) = filter_pool {
		module.merge(
			EthFilter::new(
				client.clone(),
				frontier_backend,
				graph,
				filter_pool,
				500_usize, // max stored filters
				max_past_logs,
				block_data_cache,
			)
			.into_rpc(),
		)?;
	}

	module.merge(
		EthPubSub::new(
			pool,
			client.clone(),
			sync,
			subscription_task_executor,
			storage_override,
			pubsub_notification_sinks,
		)
		.into_rpc(),
	)?;
	module.merge(
		Net::new(
			client.clone(),
			network,
			// Whether to format the `peer_count` response as Hex (default) or not.
			true,
		)
		.into_rpc(),
	)?;
	module.merge(Web3::new(client.clone()).into_rpc())?;
	module.merge(tx_pool.into_rpc())?;

	/* tracing impl todo
	if let Some(tracing_config) = maybe_tracing_config {
		if let Some(trace_filter_requester) = tracing_config.tracing_requesters.trace {
			module.merge(
				Trace::new(client, trace_filter_requester, tracing_config.trace_filter_max_count)
					.into_rpc(),
			)?;
		}

		if let Some(debug_requester) = tracing_config.tracing_requesters.debug {
			module.merge(Debug::new(debug_requester).into_rpc())?;
		}
	}
	*/

	Ok(module)
}
