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

// Substrate
use allfeat_primitives::{AccountId, Balance, Nonce};
use sc_executor::{NativeElseWasmExecutor, NativeExecutionDispatch, NativeVersion};
// Local
use crate::eth::EthCompatRuntimeApiCollection;
use harmonie_runtime::{opaque::Block, RuntimeApi};

/// Full backend.
pub type FullBackend = sc_service::TFullBackend<Block>;
/// Full client.
pub type FullClient<RuntimeApi, Executor> =
	sc_service::TFullClient<Block, RuntimeApi, NativeElseWasmExecutor<Executor>>;

pub type Client = FullClient<RuntimeApi, HarmonieRuntimeExecutor>;

/// Only enable the benchmarking host functions when we actually want to benchmark.
#[cfg(feature = "runtime-benchmarks")]
pub type HostFunctions = frame_benchmarking::benchmarking::HostFunctions;
/// Otherwise we use empty host functions for ext host functions.
#[cfg(not(feature = "runtime-benchmarks"))]
pub type HostFunctions = ();

pub struct HarmonieRuntimeExecutor;
impl NativeExecutionDispatch for HarmonieRuntimeExecutor {
	type ExtendHostFunctions = HostFunctions;

	fn dispatch(method: &str, data: &[u8]) -> Option<Vec<u8>> {
		harmonie_runtime::api::dispatch(method, data)
	}

	fn native_version() -> NativeVersion {
		harmonie_runtime::native_version()
	}
}

/// A set of APIs that every runtimes must implement.
pub trait BaseRuntimeApiCollection:
	sp_api::ApiExt<Block>
	+ sp_api::Metadata<Block>
	+ sp_block_builder::BlockBuilder<Block>
	+ sp_offchain::OffchainWorkerApi<Block>
	+ sp_session::SessionKeys<Block>
	+ sp_consensus_babe::BabeApi<Block>
	+ sp_consensus_grandpa::GrandpaApi<Block>
	+ sp_transaction_pool::runtime_api::TaggedTransactionQueue<Block>
{
}

impl<Api> BaseRuntimeApiCollection for Api where
	Api: sp_api::ApiExt<Block>
		+ sp_api::Metadata<Block>
		+ sp_block_builder::BlockBuilder<Block>
		+ sp_offchain::OffchainWorkerApi<Block>
		+ sp_session::SessionKeys<Block>
		+ sp_consensus_babe::BabeApi<Block>
		+ sp_consensus_grandpa::GrandpaApi<Block>
		+ sp_transaction_pool::runtime_api::TaggedTransactionQueue<Block>
{
}

/// A set of APIs that template runtime must implement.
pub trait RuntimeApiCollection:
	BaseRuntimeApiCollection
	+ EthCompatRuntimeApiCollection
	+ frame_system_rpc_runtime_api::AccountNonceApi<Block, AccountId, Nonce>
	+ pallet_transaction_payment_rpc_runtime_api::TransactionPaymentApi<Block, Balance>
	+ sp_authority_discovery::AuthorityDiscoveryApi<Block>
{
}

impl<Api> RuntimeApiCollection for Api where
	Api: BaseRuntimeApiCollection
		+ EthCompatRuntimeApiCollection
		+ frame_system_rpc_runtime_api::AccountNonceApi<Block, AccountId, Nonce>
		+ pallet_transaction_payment_rpc_runtime_api::TransactionPaymentApi<Block, Balance>
		+ sp_authority_discovery::AuthorityDiscoveryApi<Block>
{
}
