use crate::{
	configs::{MiddsDepositBase, MiddsDepositPerByte},
	AccountId, BalancesConfig, CollatorSelectionConfig, MusicalWorksConfig, ParachainInfoConfig,
	PolkadotXcmConfig, RecordingsConfig, ReleasesConfig, RuntimeGenesisConfig, SessionConfig,
	SessionKeys, SudoConfig, EXISTENTIAL_DEPOSIT, PARA_ID,
};
use alloc::{vec, vec::Vec};
use polkadot_sdk::{
	cumulus_primitives_core::ParaId, frame_support::build_struct_json_patch,
	parachains_common::AuraId, sp_genesis_builder, sp_genesis_builder::PresetId,
	sp_keyring::Sr25519Keyring,
};
use serde_json::Value;

const SAFE_XCM_VERSION: u32 = polkadot_sdk::staging_xcm::prelude::XCM_VERSION;

pub fn template_session_keys(keys: AuraId) -> SessionKeys {
	SessionKeys { aura: keys }
}

fn testnet_genesis(
	invulnerables: Vec<(AccountId, AuraId)>,
	endowed_accounts: Vec<AccountId>,
	root: AccountId,
	id: ParaId,
) -> Value {
	build_struct_json_patch!(RuntimeGenesisConfig {
		balances: BalancesConfig {
			balances: endowed_accounts
				.iter()
				.cloned()
				.map(|k| (k, 1u128 << 60))
				.collect::<Vec<_>>()
		},
		parachain_info: ParachainInfoConfig { parachain_id: id },
		collator_selection: CollatorSelectionConfig {
			invulnerables: invulnerables.iter().cloned().map(|(acc, _)| acc).collect::<Vec<_>>(),
			candidacy_bond: EXISTENTIAL_DEPOSIT * 16
		},
		session: SessionConfig {
			keys: invulnerables
				.into_iter()
				.map(|(acc, aura)| {
					(
						acc.clone(),
						acc,
						template_session_keys(aura),
					)
				})
				.collect::<Vec<_>>()
		},
		polkadot_xcm: PolkadotXcmConfig { safe_xcm_version: Some(SAFE_XCM_VERSION) },
		sudo: SudoConfig { key: Some(root) },
		musical_works: MusicalWorksConfig {
			deposit_base: MiddsDepositBase::get(),
			deposit_per_byte: MiddsDepositPerByte::get(),
		},
		recordings: RecordingsConfig {
			deposit_base: MiddsDepositBase::get(),
			deposit_per_byte: MiddsDepositPerByte::get(),
		},
		releases: ReleasesConfig {
			deposit_base: MiddsDepositBase::get(),
			deposit_per_byte: MiddsDepositPerByte::get(),
		},
	})
}

fn local_testnet_genesis() -> Value {
	testnet_genesis(
		vec![
			(Sr25519Keyring::Alice.to_account_id(), Sr25519Keyring::Alice.public().into()),
			(Sr25519Keyring::Bob.to_account_id(), Sr25519Keyring::Bob.public().into()),
		],
		Sr25519Keyring::well_known().map(|k| k.to_account_id()).collect(),
		Sr25519Keyring::Alice.to_account_id(),
		PARA_ID.into(),
	)
}

fn development_config_genesis() -> Value {
	testnet_genesis(
		vec![
			(Sr25519Keyring::Alice.to_account_id(), Sr25519Keyring::Alice.public().into()),
			(Sr25519Keyring::Bob.to_account_id(), Sr25519Keyring::Bob.public().into()),
		],
		Sr25519Keyring::well_known().map(|k| k.to_account_id()).collect(),
		Sr25519Keyring::Alice.public().into(),
		PARA_ID.into(),
	)
}

pub fn get_preset(id: &PresetId) -> Option<Vec<u8>> {
	let patch = match id.as_ref() {
		sp_genesis_builder::LOCAL_TESTNET_RUNTIME_PRESET => local_testnet_genesis(),
		sp_genesis_builder::DEV_RUNTIME_PRESET => development_config_genesis(),
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
	]
}
