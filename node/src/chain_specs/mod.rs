// This file is part of Substrate.

// Copyright (C) 2018-2022 Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: GPL-3.0-or-later WITH Classpath-exception-2.0

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

//! Substrate chain configurations.

pub mod genesis;
pub mod helpers;

use crate::chain_specs::helpers::{authority_keys_from_seed, chain_properties};
pub use allfeat_primitives::{AccountId, Balance, Signature};
use hex_literal::hex;
use pallet_im_online::sr25519::AuthorityId as ImOnlineId;
use sc_chain_spec::{ChainSpecExtension, ChainType, GenericChainSpec};
use serde::{Deserialize, Serialize};
use sp_authority_discovery::AuthorityId as AuthorityDiscoveryId;
use sp_consensus_babe::AuthorityId as BabeId;
use sp_core::{crypto::UncheckedInto, sr25519};
use sp_finality_grandpa::AuthorityId as GrandpaId;
use sp_runtime::traits::{IdentifyAccount, Verify};
pub use symphonie_runtime::{Block, SessionKeys};

type AccountPublic = <Signature as Verify>::Signer;

/// Node `ChainSpec` extensions.
///
/// Additional parameters for some Substrate core modules,
/// customizable from the chain spec.
#[derive(Default, Clone, Serialize, Deserialize, ChainSpecExtension)]
#[serde(rename_all = "camelCase")]
pub struct Extensions {
	/// Block numbers with known hashes.
	pub fork_blocks: sc_client_api::ForkBlocks<Block>,
	/// Known bad block hashes.
	pub bad_blocks: sc_client_api::BadBlocks<Block>,
	/// The light sync state extension used by the sync-state rpc.
	pub light_sync_state: sc_sync_state_rpc::LightSyncStateExtension,
}

#[allow(unused)]
// Dummy chain spec, in case when we don't have the native runtime.
pub type DummyChainSpec = GenericChainSpec<(), Extensions>;

#[cfg(feature = "symphonie-native")]
pub type SymphonieChainSpec = GenericChainSpec<symphonie_runtime::GenesisConfig, Extensions>;
#[cfg(not(feature = "symphonie-native"))]
pub type SymphonieChainSpec = GenericChainSpec<DummyChainSpec, Extensions>;

pub fn symphonie_config() -> Result<SymphonieChainSpec, String> {
	SymphonieChainSpec::from_json_bytes(&include_bytes!("../../genesis/symphonie_raw.json")[..])
}

/// Development config (single validator Alice)
pub fn development_config() -> SymphonieChainSpec {
	SymphonieChainSpec::from_genesis(
		"Symphonie Development",
		"symphonie_dev",
		ChainType::Development,
		genesis::symphonie_dev_genesis,
		vec![],
		None,
		None,
		None,
		Some(chain_properties()),
		Default::default(),
	)
}
