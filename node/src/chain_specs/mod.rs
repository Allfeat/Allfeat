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

use crate::chain_specs::{
	genesis::testnet_genesis,
	helpers::{authority_keys_from_seed, chain_properties, get_account_id_from_seed},
};
pub use allfeat_primitives::{AccountId, Balance, Signature};
use hex_literal::hex;
use pallet_im_online::sr25519::AuthorityId as ImOnlineId;
use sc_chain_spec::{ChainSpecExtension, ChainType};
use serde::{Deserialize, Serialize};
use sp_authority_discovery::AuthorityId as AuthorityDiscoveryId;
use sp_consensus_babe::AuthorityId as BabeId;
use sp_core::{crypto::UncheckedInto, sr25519};
use sp_finality_grandpa::AuthorityId as GrandpaId;
use sp_runtime::traits::{IdentifyAccount, Verify};
pub use symphonie_runtime::{Block, GenesisConfig, SessionKeys};

type AccountPublic = <Signature as Verify>::Signer;
pub type ChainSpec = sc_service::GenericChainSpec<GenesisConfig, Extensions>;

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

fn development_config_genesis() -> GenesisConfig {
	testnet_genesis(
		vec![authority_keys_from_seed("Alice")],
		vec![],
		get_account_id_from_seed::<sr25519::Public>("Alice"),
		None,
	)
}

pub fn symphonie_testnet_config() -> ChainSpec {
	ChainSpec::from_genesis(
		"Symphonie Testnet",
		"symphonie_testnet",
		ChainType::Live,
		symphonie_genesis,
		vec![],
		None,
		Some("allfeat"),
		None,
		Some(chain_properties()),
		Default::default(),
	)
}

fn symphonie_genesis() -> GenesisConfig {
	testnet_genesis(
		vec![
			(
				//5E1ddnrhNQog14eq8Ydb8AhHcHkQcMi5qN9o2zqpTd7YnLME
				hex!["562084a21a731deda557ea459e62a0ebfde8dab9d6808c6278f80c966c77bb3e"].into(),
				//5GR9Z189CSvXjzDZmVwrRpPvy3rVV5ygicTprY7n3wLVK3Di
				hex!["c08857235e75f1bf3665568fd7ee75a4bfc6867ca70b4bfda9b2c2f6ebc5e91e"].into(),
				//5DaQ3zrWxpTM3Mu95yvWLxCLyZXBadzUGEifMrscg8vAeLJk
				hex!["42e1a8e510e2049b925401a877a618f3c18a487f0203746bfcc7d106fa56c630"]
					.unchecked_into(),
				//5FW4DJxWw3okS9H27CSZYafatCstvQXbiUr5UFtMLu4R3rSx
				hex!["980a5c0885142fb8e4517273ddf6a8bbb8551caaf0452090e5ff92fc6676beb0"]
					.unchecked_into(),
				//5Et26aurfK1uZPUp31gEuNjxyMt937wYrE72QP3xSeNmdmPQ
				hex!["7c8e66ebc8cbf64e50cf668cb6de7d48272ad5243fa85186c6312a64c07d3334"]
					.unchecked_into(),
				//5D4QLJzNAWWLDV9yAgxirun14NgLJxYJgE4qyKsRy1yED5Hp
				hex!["2c013e90c7282366854ea3ba0815a7f19fc461f921810911f5c0f782816a3328"]
					.unchecked_into(),
			),
			(
				//5Dy5t4f1egBTxBLZq8zJerX43nKKtNdSsuPDpDzevbri8CfV
				hex!["542f225139b0f0c1ed69b90844825f23b2dbf159b48cb9a215ea1fc3e975736e"].into(),
				//5Demd2q77PNyzB5KS6aZPzK1tFb8kMCQKdxdcVBaTo8HJFMt
				hex!["4637403e41fa998e3841ab94abcec51f013d33a613bbd63fc1ea0676202bca3b"].into(),
				//5GLERfburZZkmbEvkriyQ6tar7AwaH1uXeG3189xBmtGqinh
				hex!["bcc884e0738864594b6fa61437b772651d64f0192dea2522bd7140ae2e22dc52"]
					.unchecked_into(),
				//5FcunhH8quD1zF1cYm9Ec5kWDPBPR6NcgN9y7XtK2LQSpQKo
				hex!["9d44b7e12f65ea3869390b510fc1ca09e3eabb1cd0822e1106926cf40b6df873"]
					.unchecked_into(),
				//5EbeeK7qbKcHTH5FtJsYea3CbR4VEo5jkxeVWNpebK6kSmqn
				hex!["7012422860ef516a603744cbfe23ab9f67b023f787ec058441029430b2c56861"]
					.unchecked_into(),
				//5GzuE2C18VsAmKCveQusKSsAvfo3E1yBV8H93SDENFBqLZtn
				hex!["da4674495e358dbbb1c41c4bf70b061f580e5506694f2a388cfd68e73f354b36"]
					.unchecked_into(),
			),
			(
				//5CLfLyVi96EwZR774vSApaVJsm12BQxmxz3nqhxizLbxrXzu
				hex!["0c2b786f3437f33aa82a69e76d5051286badb503313b44bff58eb1616136f104"].into(),
				//5F9A2qrjHMFFqz5meN2BDiRpNw6GgyyMSAQCdqrgtu6KAiaz
				hex!["8819cbda401a83ece26ae910da0ba8d92d820be1aa6a5e2a77577d822cc67042"].into(),
				//5EX6oCS5Fk5x6KMg7mawmd3qeLvirFdjpeSz66fb1sv3hHDR
				hex!["6c9a1224e5e55510838213f1474cdadfd3de7c2fd200458f937bd460056bee22"]
					.unchecked_into(),
				//5FG3sALFTJ46qZA5AyfEBnQLmhS79aDvddfmB5YnyeftDiFB
				hex!["8d5bc121a4eea2037760f51ebde38f53acadc49f9659565f290ce2863ea83192"]
					.unchecked_into(),
				//5HHZr24CBTcFxAtZbk7bBGRECBefYYfcu8xaUUQNCXotFixh
				hex!["e6fc63b0607839b2692eddaa91b5c6735c19a2c82eaaf583aa781788995cfa5a"]
					.unchecked_into(),
				//5G9ShAdpXqXiG2NPuRo6KDkoNAznbQCwJrez1xZGiqw4Bqpp
				hex!["b48e1d66d5946135e8e6301af39c5e6053ae4020074441ca56e8d426eadd9e28"]
					.unchecked_into(),
			),
		],
		vec![],
		//5GRB8FhizYEkp18o9HurU1TaHyeyJAZ29BfBmnf4fbSbdSPh
		hex!["eed6fe7cfd6d51c3e67def8218078c5e1e4d5dfc2352b772da18a6014baf0a1e"].into(),
		Some(vec![
			//5GRB8FhizYEkp18o9HurU1TaHyeyJAZ29BfBmnf4fbSbdSPh
			hex!["eed6fe7cfd6d51c3e67def8218078c5e1e4d5dfc2352b772da18a6014baf0a1e"].into(),
		]),
	)
}

/// Development config (single validator Alice)
pub fn development_config() -> ChainSpec {
	ChainSpec::from_genesis(
		"Allfeat Development",
		"allfeat_dev",
		ChainType::Development,
		development_config_genesis,
		vec![],
		None,
		None,
		None,
		Some(chain_properties()),
		Default::default(),
	)
}

fn local_testnet_genesis() -> GenesisConfig {
	testnet_genesis(
		vec![authority_keys_from_seed("Alice"), authority_keys_from_seed("Bob")],
		vec![],
		get_account_id_from_seed::<sr25519::Public>("Alice"),
		None,
	)
}

/// Local testnet config (multivalidator Alice + Bob)
pub fn local_testnet_config() -> ChainSpec {
	ChainSpec::from_genesis(
		"Allfeat Local Testnet",
		"allfeat_local_testnet",
		ChainType::Local,
		local_testnet_genesis,
		vec![],
		None,
		None,
		None,
		Some(chain_properties()),
		Default::default(),
	)
}

#[cfg(test)]
pub(crate) mod tests {
	use super::*;
	use crate::service::{new_full_base, NewFullBase};
	use sc_service_test;
	use sp_runtime::BuildStorage;

	fn local_testnet_genesis_instant_single() -> GenesisConfig {
		testnet_genesis(
			vec![authority_keys_from_seed("Alice")],
			vec![],
			get_account_id_from_seed::<sr25519::Public>("Alice"),
			None,
		)
	}

	/// Local testnet config (single validator - Alice)
	pub fn integration_test_config_with_single_authority() -> ChainSpec {
		ChainSpec::from_genesis(
			"Integration Test",
			"test",
			ChainType::Development,
			local_testnet_genesis_instant_single,
			vec![],
			None,
			None,
			None,
			None,
			Default::default(),
		)
	}

	/// Local testnet config (multivalidator Alice + Bob)
	pub fn integration_test_config_with_two_authorities() -> ChainSpec {
		ChainSpec::from_genesis(
			"Integration Test",
			"test",
			ChainType::Development,
			local_testnet_genesis,
			vec![],
			None,
			None,
			None,
			None,
			Default::default(),
		)
	}

	#[test]
	#[ignore]
	fn test_connectivity() {
		sp_tracing::try_init_simple();

		sc_service_test::connectivity(integration_test_config_with_two_authorities(), |config| {
			let NewFullBase { task_manager, client, network, transaction_pool, .. } =
				new_full_base(config, false, |_, _| ())?;
			Ok(sc_service_test::TestNetComponents::new(
				task_manager,
				client,
				network,
				transaction_pool,
			))
		});
	}

	#[test]
	fn test_create_development_chain_spec() {
		development_config().build_storage().unwrap();
	}

	#[test]
	fn test_create_local_testnet_chain_spec() {
		local_testnet_config().build_storage().unwrap();
	}
}
