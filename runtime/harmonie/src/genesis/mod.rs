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

//! Genesis presets to build the runtime.

use allfeat_primitives::{AccountId, Balance};
use development::development_config_genesis;
use local::local_config_genesis;
pub use pallet_im_online::sr25519::AuthorityId as ImOnlineId;
use shared_runtime::{
	currency::AFT,
	genesis_utils::{get_account_id_from_seed, get_from_seed},
};
pub use sp_authority_discovery::AuthorityId as AuthorityDiscoveryId;
pub use sp_consensus_babe::AuthorityId as BabeId;
pub use sp_consensus_grandpa::AuthorityId as GrandpaId;
use sp_genesis_builder::PresetId;
use sp_std::{vec, vec::Vec};

use crate::{
	BabeConfig, BalancesConfig, RuntimeGenesisConfig, SessionConfig, SessionKeys, SudoConfig,
	ValidatorSetConfig, BABE_GENESIS_EPOCH_CONFIG,
};

mod development;
mod local;

// Returns the genesis config template populated with given parameters.
pub fn genesis(
	initial_authorities: Vec<(
		// Validator
		AccountId,
		// Session Keys
		GrandpaId,
		BabeId,
		ImOnlineId,
		AuthorityDiscoveryId,
	)>,
	root_key: AccountId,
	mut endowed_accounts: Vec<AccountId>,
) -> serde_json::Value {
	// endow all authorities and nominators.
	initial_authorities.iter().map(|x| &x.0).for_each(|x| {
		if !endowed_accounts.contains(x) {
			endowed_accounts.push(x.clone())
		}
	});

	const ENDOWMENT: Balance = 10_000_000 * AFT;

	let config = RuntimeGenesisConfig {
		balances: BalancesConfig {
			balances: endowed_accounts.iter().cloned().map(|x| (x, ENDOWMENT)).collect::<Vec<_>>(),
		},
		validator_set: ValidatorSetConfig {
			initial_validators: initial_authorities.iter().map(|x| x.0.clone()).collect::<Vec<_>>(),
		},
		session: SessionConfig {
			keys: initial_authorities
				.iter()
				.map(|x| {
					(
						x.0.clone(),
						x.0.clone(),
						SessionKeys {
							grandpa: x.1.clone(),
							babe: x.2.clone(),
							im_online: x.3.clone(),
							authority_discovery: x.4.clone(),
						},
					)
				})
				.collect::<Vec<_>>(),
		},
		babe: BabeConfig { epoch_config: BABE_GENESIS_EPOCH_CONFIG, ..Default::default() },
		sudo: SudoConfig { key: Some(root_key) },
		..Default::default()
	};

	serde_json::to_value(config).expect("Could not build genesis config.")
}

/// Helper function to generate stash, controller and session key from seed
fn authority_keys_from_seed(
	seed: &str,
) -> (AccountId, GrandpaId, BabeId, ImOnlineId, AuthorityDiscoveryId) {
	(
		get_account_id_from_seed::<sp_core::ecdsa::Public>(seed),
		get_from_seed::<GrandpaId>(seed),
		get_from_seed::<BabeId>(seed),
		get_from_seed::<ImOnlineId>(seed),
		get_from_seed::<AuthorityDiscoveryId>(seed),
	)
}

/// Provides the JSON representation of predefined genesis config for given `id`.
pub fn get_preset(id: &PresetId) -> Option<Vec<u8>> {
	let patch = match id.try_into() {
		Ok("development") => development_config_genesis(),
		Ok("local_testnet") => local_config_genesis(),
		_ => return None,
	};
	Some(
		serde_json::to_string(&patch)
			.expect("serialization to json is expected to work. qed.")
			.into_bytes(),
	)
}

/// List of supported presets.
pub fn preset_names() -> Vec<PresetId> {
	vec![PresetId::from("development"), PresetId::from("local_testnet")]
}
