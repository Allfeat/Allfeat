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
use sc_telemetry::TelemetryEndpoints;
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

const STAGING_TELEMETRY_URL: &str = "wss://telemetry.polkadot.io/submit/";
const DEFAULT_PROTOCOL_ID: &str = "aft";

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
		Some(
			TelemetryEndpoints::new(vec![(STAGING_TELEMETRY_URL.to_string(), 0)])
				.expect("Staging telemetry url is valid; qed"),
		),
		Some(DEFAULT_PROTOCOL_ID),
		None,
		Some(chain_properties()),
		Default::default(),
	)
}

fn symphonie_genesis() -> GenesisConfig {
	testnet_genesis(
		vec![
			(
				//5F6jrk6r15QvJPf7PvDESkZKpVnuc54oQsGKUFQvp2QKHWrQ
				hex!["8641ed9a03e0e4e679a2d36dfee489353a3a117a1830e6b4b5c4388ec7569e70"].into(),
				//5FNC5HnVzZwntjVJiNnA4999kAvjaym5F5E1uxGtCu2wULAL
				hex!["920adc764a443e5cd284de8c8cd3757539a801f1130e10412e94b2af7bc9492a"].into(),
				//5Eo8wbEB8zJb7wbGLMXcoAeYAZda5BLFyzzhgJkVMLAWDSgs
				hex!["78d537921b129fc335828edf3e50427c09087aea2231be87ec16dd10474817ae"]
					.unchecked_into(),
				//5G12PQX7EoQbmsPhMoDBeZpbKVUArk28Tf5iZ5PcESEW8wTN
				hex!["ae2254603f8c217c246f2af1cf1565caac39c3c2c37e5fc3360f03961607e230"]
					.unchecked_into(),
				//5DZLhGUd9MqMrGvx7GrwP4p5YagrqNDTMUJxTG7SVbufYhJz
				hex!["42131cbbd5420a035447ab28417eb783cdbf323c485cc9ba4b6a06b28f5cc464"]
					.unchecked_into(),
				//5Cns2WNjbAfHzJUZjsPYCXeqCPm8aXVrCtscAbYbJ3DRLVHC
				hex!["20272cb9b2b1629d1729bb6db5e702b36db50130059ef2005f444d62a6771260"]
					.unchecked_into(),
			),
			(
				//5HBURdeyRQoRAXqyJELrvwL4b6T4D3xfoSD52hERPfbLgy8j
				hex!["e256ab8cba4cf4c694057f846c203291dcb49ca105f906e2ea86a0d3a05f102a"].into(),
				//5ENqGczpMdfQMjbM1Loon8W9np9udhp5rc96K2iCJxhjMQKJ
				hex!["664bd6a760a9669d12662d569436c662e1f5ce0ad29ec7220ca47c4c72e75e17"].into(),
				//5FcWoNNkpa1GxQSc1HAnr3vRfbg22HVncQTcebpWnKzwKbsv
				hex!["9cf7551ed4f32981e562c4aeee0ae8a352bf4924b1b59497c2d380bee30018cc"]
					.unchecked_into(),
				//5HRC18W3EwxL8oWkPMvfNv2PADS7ajS5QTe1yXGvYZR3w3p1
				hex!["ecccd2a64fdfc10cc3a0d8aefea9cc4793dd11cf4fef0384c4d641a24e741c67"]
					.unchecked_into(),
				//5HEZAEdGzRCA6yhhyNj58dMSoD47W9nsbQ5kPvpWLAbgsEW7
				hex!["e4b0581cd0d33cae2d28855f1a44a7bef94060935d4aea13e2db37b02fd3ba69"]
					.unchecked_into(),
				//5CqJt7Bu2Vz7osbYxSxwV8HQReZ2sdr4fo7CAqeAWGSQxN8V
				hex!["2204b3a91d9c5c449a9d1d9305f1a031461c38205063571708f9ff613224c447"]
					.unchecked_into(),
			),
		],
		vec![],
		hex!["8a18186617e63cf6fb3e7aa1a5569c2964f7bf7b0fccbc684048bdbfc5260f5d"].into(),
		Some(vec![
			//5FBmd6CQsTo2KUhkw5KXfP9cFoYB57tPfBoA25WqZPmo26H7
			hex!["8a18186617e63cf6fb3e7aa1a5569c2964f7bf7b0fccbc684048bdbfc5260f5d"].into(),
			//5Fjztu41BFyuYbHjT5SvwJZC2TfWUArHeFmM6B8LqBGG7Vt2
			hex!["a2aca0ac60d20205b88827a78e2c36aac7897630df589e0901b2a272da71e75f"].into(),
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
