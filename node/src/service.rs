// This file is part of Allfeat.

// Copyright (C) 2022-2025 Allfeat.
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

pub use allfeat_runtime::RuntimeApi as AllfeatRuntimeApi;
pub use melodie_runtime::RuntimeApi as MelodieRuntimeApi;

// std
use sc_consensus_aura::{ImportQueueParams, StartAuraParams};
use std::{sync::Arc, time::Duration};
// crates.io
use futures::FutureExt;
// allfeat
use allfeat_primitives::*;
// polkadot-sdk
use sc_client_api::{BlockBackend, backend::Backend};
use sc_consensus_slots::SlotProportion;
use sc_rpc_spec_v2::SubscriptionTaskExecutor;
use sc_service::{Configuration, TaskManager, WarpSyncConfig, error::Error as ServiceError};
use sc_telemetry::TelemetryWorker;
use sc_transaction_pool_api::OffchainTransactionPoolFactory;
use sp_api::ConstructRuntimeApi;
use sp_consensus_aura::sr25519::{AuthorityId as AuraId, AuthorityPair as AuraPair};

/// The minimum period of blocks on which justifications will be
/// imported and generated.
const GRANDPA_JUSTIFICATION_PERIOD: u32 = 512;

type HostFunctions = sp_io::SubstrateHostFunctions;

/// Full client backend type.
type FullBackend = sc_service::TFullBackend<Block>;
/// Full client type.
pub(crate) type FullClient<RuntimeApi> =
    sc_service::TFullClient<Block, RuntimeApi, sc_executor::WasmExecutor<HostFunctions>>;
type FullSelectChain = sc_consensus::LongestChain<FullBackend, Block>;
type FullGrandpaBlockImport<RA> =
    sc_consensus_grandpa::GrandpaBlockImport<FullBackend, Block, FullClient<RA>, FullSelectChain>;
type GrandpaLinkHalf<RA> = sc_consensus_grandpa::LinkHalf<Block, FullClient<RA>, FullSelectChain>;

pub(crate) struct ConsensusParts<RuntimeApi> {
    pub grandpa_block_import: FullGrandpaBlockImport<RuntimeApi>,
    pub grandpa_link: GrandpaLinkHalf<RuntimeApi>,
}
pub(crate) struct ExtraParts<RuntimeApi> {
    pub consensus_parts: ConsensusParts<RuntimeApi>,
    pub telemetry: Option<sc_telemetry::Telemetry>,
}

type Service<RuntimeApi> = sc_service::PartialComponents<
    FullClient<RuntimeApi>,
    FullBackend,
    FullSelectChain,
    sc_consensus::DefaultImportQueue<Block>,
    sc_transaction_pool::TransactionPoolHandle<Block, FullClient<RuntimeApi>>,
    ExtraParts<RuntimeApi>,
>;

pub use crate::chain_specs::IdentifyVariant;

/// A set of APIs that allfeat-like runtimes must implement.
pub trait RuntimeApiCollection:
    pallet_transaction_payment_rpc::TransactionPaymentRuntimeApi<Block, Balance>
    + sp_api::ApiExt<Block>
    + sp_api::Metadata<Block>
    + sp_block_builder::BlockBuilder<Block>
    + sp_consensus_grandpa::GrandpaApi<Block>
    + sp_consensus_aura::AuraApi<Block, AuraId>
    + sp_offchain::OffchainWorkerApi<Block>
    + sp_session::SessionKeys<Block>
    + sp_transaction_pool::runtime_api::TaggedTransactionQueue<Block>
    + substrate_frame_rpc_system::AccountNonceApi<Block, AccountId, Nonce>
{
}
impl<Api> RuntimeApiCollection for Api where
    Api: pallet_transaction_payment_rpc::TransactionPaymentRuntimeApi<Block, Balance>
        + sp_api::ApiExt<Block>
        + sp_api::Metadata<Block>
        + sp_block_builder::BlockBuilder<Block>
        + sp_consensus_grandpa::GrandpaApi<Block>
        + sp_consensus_aura::AuraApi<Block, AuraId>
        + sp_offchain::OffchainWorkerApi<Block>
        + sp_session::SessionKeys<Block>
        + sp_transaction_pool::runtime_api::TaggedTransactionQueue<Block>
        + substrate_frame_rpc_system::AccountNonceApi<Block, AccountId, Nonce>
{
}

pub fn new_partial<RuntimeApi>(
    config: &Configuration,
) -> Result<Service<RuntimeApi>, Box<ServiceError>>
where
    RuntimeApi: ConstructRuntimeApi<Block, FullClient<RuntimeApi>>,
    RuntimeApi: Send + Sync + 'static,
    RuntimeApi::RuntimeApi: RuntimeApiCollection,
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
        .transpose()
        .map_err(|e| Box::new(sc_service::Error::Application(e.into())))?;

    let executor = sc_service::new_wasm_executor::<sp_io::SubstrateHostFunctions>(&config.executor);
    let (client, backend, keystore_container, task_manager) =
        sc_service::new_full_parts::<Block, RuntimeApi, _>(
            config,
            telemetry.as_ref().map(|(_, telemetry)| telemetry.handle()),
            executor,
        )?;
    let client = Arc::new(client);

    let telemetry = telemetry.map(|(worker, telemetry)| {
        task_manager
            .spawn_handle()
            .spawn("telemetry", None, worker.run());
        telemetry
    });

    let select_chain = sc_consensus::LongestChain::new(backend.clone());

    let transaction_pool = Arc::from(
        sc_transaction_pool::Builder::new(
            task_manager.spawn_essential_handle(),
            client.clone(),
            config.role.is_authority().into(),
        )
        .with_options(config.transaction_pool.clone())
        .with_prometheus(config.prometheus_registry())
        .build(),
    );

    let (grandpa_block_import, grandpa_link) = sc_consensus_grandpa::block_import(
        client.clone(),
        GRANDPA_JUSTIFICATION_PERIOD,
        &client,
        select_chain.clone(),
        telemetry.as_ref().map(|x| x.handle()),
    )
    .map_err(|e| Box::new(sc_service::Error::Application(e.into())))?;

    let cidp_client = client.clone();
    let import_queue =
        sc_consensus_aura::import_queue::<AuraPair, _, _, _, _, _>(ImportQueueParams {
            block_import: grandpa_block_import.clone(),
            justification_import: Some(Box::new(grandpa_block_import.clone())),
            client: client.clone(),
            create_inherent_data_providers: move |parent_hash, _| {
                let cidp_client = cidp_client.clone();
                async move {
                    let slot_duration =
                        sc_consensus_aura::standalone::slot_duration_at(&*cidp_client, parent_hash)
                            .map_err(|e| {
                                Box::new(sp_consensus::error::Error::ClientImport(format!("{e:?}")))
                            })?;
                    let timestamp = sp_timestamp::InherentDataProvider::from_system_time();

                    let slot =
						sp_consensus_aura::inherents::InherentDataProvider::from_timestamp_and_slot_duration(
							*timestamp,
							slot_duration,
						);

                    Ok((slot, timestamp))
                }
            },
            spawner: &task_manager.spawn_essential_handle(),
            registry: config.prometheus_registry(),
            check_for_equivocation: Default::default(),
            telemetry: telemetry.as_ref().map(|x| x.handle()),
            compatibility_mode: Default::default(),
        })
        .map_err(|e| Box::new(sc_service::Error::Application(e.into())))?;
    let consensus_parts = ConsensusParts {
        grandpa_block_import,
        grandpa_link,
    };

    Ok(sc_service::PartialComponents {
        client,
        backend,
        task_manager,
        import_queue,
        keystore_container,
        select_chain,
        transaction_pool,
        other: ExtraParts::<RuntimeApi> {
            consensus_parts,
            telemetry,
        },
    })
}

/// Builds a new service for a full client.
fn new_full<RuntimeApi, N>(config: Configuration) -> Result<TaskManager, Box<ServiceError>>
where
    N: sc_network::NetworkBackend<Block, <Block as sp_runtime::traits::Block>::Hash>,
    RuntimeApi: ConstructRuntimeApi<Block, FullClient<RuntimeApi>>,
    RuntimeApi: Send + Sync + 'static,
    RuntimeApi::RuntimeApi: RuntimeApiCollection,
{
    let sc_service::PartialComponents {
        client,
        backend,
        mut task_manager,
        import_queue,
        keystore_container,
        select_chain,
        transaction_pool,
        other: mut extra_parts,
    } = new_partial::<RuntimeApi>(&config)?;

    let mut net_config = sc_network::config::FullNetworkConfiguration::<
        Block,
        <Block as sp_runtime::traits::Block>::Hash,
        N,
    >::new(&config.network, config.prometheus_registry().cloned());

    let metrics = N::register_notification_metrics(config.prometheus_registry());

    let peer_store_handle = net_config.peer_store_handle();
    let grandpa_protocol_name = sc_consensus_grandpa::protocol_standard_name(
        &client
            .block_hash(0)
            .ok()
            .flatten()
            .expect("Genesis block exists; qed"),
        &config.chain_spec,
    );
    let (grandpa_protocol_config, grandpa_notification_service) =
        sc_consensus_grandpa::grandpa_peers_set_config::<_, N>(
            grandpa_protocol_name.clone(),
            metrics.clone(),
            peer_store_handle,
        );
    net_config.add_notification_protocol(grandpa_protocol_config);

    let warp_sync = Arc::new(sc_consensus_grandpa::warp_proof::NetworkProvider::new(
        backend.clone(),
        extra_parts
            .consensus_parts
            .grandpa_link
            .shared_authority_set()
            .clone(),
        Vec::default(),
    ));

    let (network, system_rpc_tx, tx_handler_controller, sync_service) =
        sc_service::build_network(sc_service::BuildNetworkParams {
            config: &config,
            net_config,
            client: client.clone(),
            transaction_pool: transaction_pool.clone(),
            spawn_handle: task_manager.spawn_handle(),
            import_queue,
            block_announce_validator_builder: None,
            warp_sync_config: Some(WarpSyncConfig::WithProvider(warp_sync)),
            block_relay: None,
            metrics,
        })?;

    if config.offchain_worker.enabled {
        task_manager.spawn_handle().spawn(
            "offchain-workers-runner",
            "offchain-worker",
            sc_offchain::OffchainWorkers::new(sc_offchain::OffchainWorkerOptions {
                runtime_api_provider: client.clone(),
                is_validator: config.role.is_authority(),
                keystore: Some(keystore_container.keystore()),
                offchain_db: backend.offchain_storage(),
                transaction_pool: Some(OffchainTransactionPoolFactory::new(
                    transaction_pool.clone(),
                )),
                network_provider: Arc::new(network.clone()),
                enable_http_requests: true,
                custom_extensions: |_| vec![],
            })
            .map_err(|e| Box::new(sc_service::Error::Application(e.into())))?
            .run(client.clone(), task_manager.spawn_handle())
            .boxed(),
        );
    }

    let role = config.role;
    let force_authoring = config.force_authoring;
    let backoff_authoring_blocks: Option<()> = None;
    let name = config.network.node_name.clone();
    let enable_grandpa = !config.disable_grandpa;
    let prometheus_registry = config.prometheus_registry().cloned();

    let rpc_extensions_builder = {
        let grandpa_link = &extra_parts.consensus_parts.grandpa_link;

        let client = client.clone();
        let pool = transaction_pool.clone();
        let justification_stream = grandpa_link.justification_stream();
        let shared_authority_set = grandpa_link.shared_authority_set().clone();
        let shared_voter_state = sc_consensus_grandpa::SharedVoterState::empty();
        let finality_proof_provider = sc_consensus_grandpa::FinalityProofProvider::new_for_service(
            backend.clone(),
            Some(shared_authority_set.clone()),
        );

        Box::new(move |subscription_executor: SubscriptionTaskExecutor| {
            let deps = crate::rpc::FullDeps {
                client: client.clone(),
                pool: pool.clone(),
                grandpa: crate::rpc::GrandpaDeps {
                    shared_voter_state: shared_voter_state.clone(),
                    shared_authority_set: shared_authority_set.clone(),
                    justification_stream: justification_stream.clone(),
                    subscription_executor: subscription_executor.clone(),
                    finality_provider: finality_proof_provider.clone(),
                },
            };
            crate::rpc::create_full(deps).map_err(Into::into)
        })
    };

    sc_service::spawn_tasks(sc_service::SpawnTasksParams {
        network: Arc::new(network.clone()),
        client: client.clone(),
        keystore: keystore_container.keystore(),
        task_manager: &mut task_manager,
        transaction_pool: transaction_pool.clone(),
        rpc_builder: rpc_extensions_builder,
        backend,
        system_rpc_tx,
        tx_handler_controller,
        sync_service: sync_service.clone(),
        config,
        telemetry: extra_parts.telemetry.as_mut(),
        tracing_execute_block: None,
    })?;

    if role.is_authority() {
        let proposer_factory = sc_basic_authorship::ProposerFactory::new(
            task_manager.spawn_handle(),
            client.clone(),
            transaction_pool.clone(),
            prometheus_registry.as_ref(),
            extra_parts.telemetry.as_ref().map(|x| x.handle()),
        );

        let slot_duration = sc_consensus_aura::slot_duration(&*client)
            .map_err(|e| Box::new(sc_service::Error::Application(e.into())))?;

        let aura = sc_consensus_aura::start_aura::<AuraPair, _, _, _, _, _, _, _, _, _, _>(
            StartAuraParams {
                slot_duration,
                client,
                select_chain,
                block_import: extra_parts.consensus_parts.grandpa_block_import,
                proposer_factory,
                create_inherent_data_providers: move |_, ()| async move {
                    let timestamp = sp_timestamp::InherentDataProvider::from_system_time();

                    let slot =
						sp_consensus_aura::inherents::InherentDataProvider::from_timestamp_and_slot_duration(
							*timestamp,
							slot_duration,
						);

                    Ok((slot, timestamp))
                },
                force_authoring,
                backoff_authoring_blocks,
                keystore: keystore_container.keystore(),
                sync_oracle: sync_service.clone(),
                justification_sync_link: sync_service.clone(),
                block_proposal_slot_portion: SlotProportion::new(2f32 / 3f32),
                max_block_proposal_slot_portion: None,
                telemetry: extra_parts.telemetry.as_ref().map(|x| x.handle()),
                compatibility_mode: Default::default(),
            },
        )
        .map_err(|e| Box::new(sc_service::Error::Application(e.into())))?;

        // the AURA authoring task is considered essential, i.e. if it
        // fails we take down the service with it.
        task_manager
            .spawn_essential_handle()
            .spawn_blocking("aura", Some("block-authoring"), aura);
    }

    if enable_grandpa {
        // if the node isn't actively participating in consensus then it doesn't
        // need a keystore, regardless of which protocol we use below.
        let keystore = if role.is_authority() {
            Some(keystore_container.keystore())
        } else {
            None
        };

        let grandpa_config = sc_consensus_grandpa::Config {
            gossip_duration: Duration::from_millis(333),
            justification_generation_period: GRANDPA_JUSTIFICATION_PERIOD,
            name: Some(name),
            observer_enabled: false,
            keystore,
            local_role: role,
            telemetry: extra_parts.telemetry.as_ref().map(|x| x.handle()),
            protocol_name: grandpa_protocol_name,
        };

        // start the full GRANDPA voter
        // NOTE: non-authorities could run the GRANDPA observer protocol, but at
        // this point the full voter should provide better guarantees of block
        // and vote data availability than the observer. The observer has not
        // been tested extensively yet and having most nodes in a network run it
        // could lead to finality stalls.
        let grandpa_config = sc_consensus_grandpa::GrandpaParams {
            config: grandpa_config,
            link: extra_parts.consensus_parts.grandpa_link,
            network,
            sync: Arc::new(sync_service),
            notification_service: grandpa_notification_service,
            voting_rule: sc_consensus_grandpa::VotingRulesBuilder::default().build(),
            prometheus_registry,
            shared_voter_state: sc_consensus_grandpa::SharedVoterState::empty(),
            telemetry: extra_parts.telemetry.as_ref().map(|x| x.handle()),
            offchain_tx_pool_factory: OffchainTransactionPoolFactory::new(transaction_pool),
        };

        // the GRANDPA voter task is considered infallible, i.e.
        // if it fails we take down the service with it.
        task_manager.spawn_essential_handle().spawn_blocking(
            "grandpa-voter",
            None,
            sc_consensus_grandpa::run_grandpa_voter(grandpa_config)
                .map_err(|e| Box::new(sc_service::Error::Application(e.into())))?,
        );
    }

    Ok(task_manager)
}

pub fn new_full_from_network_cfg<RuntimeApi>(
    config: Configuration,
) -> Result<TaskManager, Box<ServiceError>>
where
    RuntimeApi: ConstructRuntimeApi<Block, FullClient<RuntimeApi>>,
    RuntimeApi: Send + Sync + 'static,
    RuntimeApi::RuntimeApi: RuntimeApiCollection,
{
    match config.network.network_backend {
        sc_network::config::NetworkBackendType::Libp2p => new_full::<
            RuntimeApi,
            sc_network::NetworkWorker<Block, <Block as sp_runtime::traits::Block>::Hash>,
        >(config),
        sc_network::config::NetworkBackendType::Litep2p => {
            new_full::<RuntimeApi, sc_network::Litep2pNetworkBackend>(config)
        }
    }
}
