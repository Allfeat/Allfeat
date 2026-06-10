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

use crate::{
	AccountId, Balance, CollatorSelectionConfig, EXISTENTIAL_DEPOSIT, PARA_ID, ParachainInfoConfig,
	PolkadotXcmConfig, RuntimeGenesisConfig, SessionConfig, SessionKeys, SudoConfig, UNIT,
	genesis_token::{VALIDATOR_ENDOWMENT, tokenomics},
};
use alloc::{vec, vec::Vec};
use array_bytes::Dehexify;
use polkadot_sdk::{
	cumulus_primitives_core::ParaId, frame_support::build_struct_json_patch,
	parachains_common::AuraId, sp_core::crypto::UncheckedInto, sp_genesis_builder,
	sp_genesis_builder::PresetId, sp_keyring::Sr25519Keyring,
};
use serde_json::Value;

const SAFE_XCM_VERSION: u32 = polkadot_sdk::staging_xcm::prelude::XCM_VERSION;

/// Endowment of the convenience dev accounts on dev/local chains, taken on
/// top of the tokenomics (which only a real network should preserve).
const DEV_ENDOWMENT: Balance = 100_000_000 * UNIT;

pub fn session_keys(keys: AuraId) -> SessionKeys {
	SessionKeys { aura: keys }
}

fn merge_balance(balances: &mut Vec<(AccountId, Balance)>, account: AccountId, amount: Balance) {
	if let Some((_, existing_amount)) =
		balances.iter_mut().find(|(existing_account, _)| *existing_account == account)
	{
		*existing_amount = existing_amount.saturating_add(amount);
	} else {
		balances.push((account, amount));
	}
}

fn genesis(
	invulnerables: Vec<(AccountId, AuraId)>,
	dev_accounts: Vec<AccountId>,
	root: AccountId,
	id: ParaId,
) -> Value {
	let mut token_genesis = tokenomics(root.clone(), invulnerables.len() as u128);
	// Give each collator an initial endowment (taken from the R&D envelope).
	for (account, _) in &invulnerables {
		merge_balance(&mut token_genesis.balances.balances, account.clone(), VALIDATOR_ENDOWMENT);
	}
	for account in dev_accounts {
		merge_balance(&mut token_genesis.balances.balances, account, DEV_ENDOWMENT);
	}
	build_struct_json_patch!(RuntimeGenesisConfig {
		balances: token_genesis.balances,
		token_allocation: token_genesis.allocations,
		parachain_info: ParachainInfoConfig { parachain_id: id },
		collator_selection: CollatorSelectionConfig {
			invulnerables: invulnerables.iter().cloned().map(|(acc, _)| acc).collect::<Vec<_>>(),
			candidacy_bond: EXISTENTIAL_DEPOSIT * 16
		},
		session: SessionConfig {
			keys: invulnerables
				.into_iter()
				.map(|(acc, aura)| { (acc.clone(), acc, session_keys(aura),) })
				.collect::<Vec<_>>()
		},
		polkadot_xcm: PolkadotXcmConfig { safe_xcm_version: Some(SAFE_XCM_VERSION) },
		sudo: SudoConfig { key: Some(root) },
	})
}

fn development_config_genesis() -> Value {
	genesis(
		vec![(Sr25519Keyring::Alice.to_account_id(), Sr25519Keyring::Alice.public().into())],
		vec![
			Sr25519Keyring::Bob.to_account_id(),
			Sr25519Keyring::Charlie.to_account_id(),
			Sr25519Keyring::Dave.to_account_id(),
			Sr25519Keyring::Eve.to_account_id(),
			Sr25519Keyring::Ferdie.to_account_id(),
		],
		Sr25519Keyring::Alice.to_account_id(),
		PARA_ID.into(),
	)
}

fn local_testnet_genesis() -> Value {
	genesis(
		vec![
			(Sr25519Keyring::Alice.to_account_id(), Sr25519Keyring::Alice.public().into()),
			(Sr25519Keyring::Bob.to_account_id(), Sr25519Keyring::Bob.public().into()),
		],
		vec![Sr25519Keyring::Bob.to_account_id()],
		Sr25519Keyring::Alice.to_account_id(),
		PARA_ID.into(),
	)
}

/// Helper for the hex-encoded staging keys below (the SS58 forms are kept in
/// comments; `Ss58Codec` is not available in a `no_std` runtime).
fn account_from_hex(hex: &str) -> AccountId {
	AccountId::new(<[u8; 32]>::dehexify(hex).expect("static hex key is valid; qed"))
}

fn aura_from_hex(hex: &str) -> AuraId {
	<[u8; 32]>::dehexify(hex)
		.expect("static hex key is valid; qed")
		.unchecked_into()
}

fn staging_config_genesis() -> Value {
	genesis(
		vec![
			// Collator 1 (qSuo1LcUoi7JFNQbar8r8N7JN9JSMggsFEUKcPWc6sEyPjiFa)
			(
				account_from_hex(
					"14bd31575f7cb6148d57cf6c9028c4b0937c0e38516e7639746852f19e12cb33",
				),
				aura_from_hex("2edd6141c37e37a90b7bb8398346d6689e7ccda12c6b8bf9bba124549a7e626f"),
			),
			// Collator 2 (qSv3xY3t1rFkhxvpSBdqChhwjTCqJ1qJjNoq5ZKFo8vvTgms4)
			(
				account_from_hex(
					"202470527888387cd90f557e02780829b4827d8a606c30f734cb82dfdd01d319",
				),
				aura_from_hex("e050792174140b0d17097c7cf837ab6e07a79f9a8c3682574bccf30ffe7c1b2f"),
			),
		],
		vec![],
		// qSysBTZC3yQRKNroife4djUTQwnfVxHQ19PpxgHKcRFJszHRA
		account_from_hex("c8dd2c957dc60b384cc5cf901f6b980bf1a9acf90b4ac7a45e7c408f3436a317"),
		PARA_ID.into(),
	)
}

pub fn get_preset(id: &PresetId) -> Option<Vec<u8>> {
	let patch = match id.as_ref() {
		sp_genesis_builder::LOCAL_TESTNET_RUNTIME_PRESET => local_testnet_genesis(),
		sp_genesis_builder::DEV_RUNTIME_PRESET => development_config_genesis(),
		"staging" => staging_config_genesis(),
		_ => return None,
	};
	Some(
		serde_json::to_string(&patch)
			.expect("serialization to json is expected to work. qed.")
			.into_bytes(),
	)
}

pub fn preset_names() -> Vec<PresetId> {
	vec![
		PresetId::from(sp_genesis_builder::DEV_RUNTIME_PRESET),
		PresetId::from(sp_genesis_builder::LOCAL_TESTNET_RUNTIME_PRESET),
		PresetId::from("staging"),
	]
}
