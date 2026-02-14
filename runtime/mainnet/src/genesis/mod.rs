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

//! Genesis presets to build the runtime.

extern crate alloc;

use allfeat_primitives::{AccountId, Balance};
use alloc::{vec, vec::Vec};
use development::development_config_genesis;
use frame_support::build_struct_json_patch;
use local::local_config_genesis;
use shared_runtime::currency::AFT;
use sp_consensus_aura::sr25519::AuthorityId as AuraId;
use sp_consensus_grandpa::AuthorityId as GrandpaId;
use sp_genesis_builder::PresetId;
use staging::staging_config_genesis;
use token::{VALIDATOR_ENDOWMENT, tokenomics};

use crate::{RuntimeGenesisConfig, SessionKeys};

mod development;
mod local;
pub mod staging;

pub mod token;

const DEV_ENDOWMENT: Balance = 100_000_000 * AFT;

fn merge_balance(balances: &mut Vec<(AccountId, Balance)>, account: AccountId, amount: Balance) {
    if let Some((_, existing_amount)) = balances
        .iter_mut()
        .find(|(existing_account, _)| *existing_account == account)
    {
        *existing_amount = existing_amount.saturating_add(amount);
    } else {
        balances.push((account, amount));
    }
}

// Returns the genesis config template populated with given parameters.
pub fn genesis(
    initial_authorities: Vec<(
        // Validator
        AccountId,
        // Session Keys
        GrandpaId,
        AuraId,
    )>,
    dev_accounts: Vec<AccountId>,
    root_key: AccountId,
) -> serde_json::Value {
    let mut token_genesis = tokenomics(root_key.clone(), initial_authorities.len() as u128);

    // Give each validator an initial endowment (taken from R&D envelope)
    for (account, _, _) in &initial_authorities {
        merge_balance(
            &mut token_genesis.balances.balances,
            account.clone(),
            VALIDATOR_ENDOWMENT,
        );
    }

    for account in dev_accounts {
        merge_balance(&mut token_genesis.balances.balances, account, DEV_ENDOWMENT);
    }

    build_struct_json_patch!(RuntimeGenesisConfig {
        balances: token_genesis.balances,
        token_allocation: token_genesis.allocations,
        validators: pallet_validators::GenesisConfig {
            initial_validators: initial_authorities
                .iter()
                .map(|x| x.0.clone())
                .collect::<Vec<_>>(),
        },
        session: pallet_session::GenesisConfig {
            keys: initial_authorities
                .iter()
                .map(|x| {
                    (
                        x.0.clone(),
                        x.0.clone(),
                        SessionKeys {
                            grandpa: x.1.clone(),
                            aura: x.2.clone(),
                        },
                    )
                })
                .collect::<Vec<_>>(),
            non_authority_keys: Default::default(),
        },
        sudo: pallet_sudo::GenesisConfig {
            key: Some(root_key)
        },
    })
}

/// Provides the JSON representation of predefined genesis config for given `id`.
pub fn get_preset(id: &PresetId) -> Option<Vec<u8>> {
    let patch = match id.as_ref() {
        sp_genesis_builder::DEV_RUNTIME_PRESET => development_config_genesis(),
        sp_genesis_builder::LOCAL_TESTNET_RUNTIME_PRESET => local_config_genesis(),
        "staging" => staging_config_genesis(),
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
    vec![
        PresetId::from(sp_genesis_builder::DEV_RUNTIME_PRESET),
        PresetId::from(sp_genesis_builder::LOCAL_TESTNET_RUNTIME_PRESET),
        PresetId::from("staging"),
    ]
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use super::*;

    fn assert_no_duplicate_balances(patch: serde_json::Value) {
        let balances = patch
            .get("balances")
            .and_then(|b| b.get("balances"))
            .and_then(serde_json::Value::as_array)
            .expect("balances.balances should be present in genesis patch");

        let mut seen = BTreeSet::new();
        for entry in balances {
            let account = entry
                .get(0)
                .expect("balance entry should contain account id")
                .to_string();
            assert!(
                seen.insert(account.clone()),
                "duplicate account in balances genesis: {account}"
            );
        }
    }

    #[test]
    fn development_preset_has_unique_balances() {
        assert_no_duplicate_balances(development_config_genesis());
    }

    #[test]
    fn local_testnet_preset_has_unique_balances() {
        assert_no_duplicate_balances(local_config_genesis());
    }
}
