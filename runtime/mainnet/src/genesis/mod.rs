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
use pallet_im_online::sr25519::AuthorityId as ImOnlineId;
use pallet_token_allocation::{EnvelopeConfig, EnvelopeId};
use shared_runtime::currency::AFT;
use sp_consensus_aura::sr25519::AuthorityId as AuraId;
use sp_consensus_grandpa::AuthorityId as GrandpaId;
use sp_genesis_builder::PresetId;
use sp_runtime::Percent;
use staging::staging_config_genesis;

use crate::{MONTHS, Runtime, RuntimeGenesisConfig, SessionKeys};

mod development;
mod local;
mod staging;

// Returns the genesis config template populated with given parameters.
pub fn genesis(
    initial_authorities: Vec<(
        // Validator
        AccountId,
        // Session Keys
        GrandpaId,
        AuraId,
        ImOnlineId,
    )>,
    root_key: AccountId,
) -> serde_json::Value {
    const TOTAL_SUPPLY: Balance = 1_000_000_000 * AFT;

    build_struct_json_patch!(RuntimeGenesisConfig {
        balances: pallet_balances::GenesisConfig {
            balances: vec![
                (EnvelopeId::Founders.account::<Runtime>(), 67_000_000),
                (EnvelopeId::KoL.account::<Runtime>(), 3_000_000),
                (EnvelopeId::Private1.account::<Runtime>(), 120_000_000),
                (EnvelopeId::Private2.account::<Runtime>(), 80_000_000),
                (EnvelopeId::ICO1.account::<Runtime>(), 30_000_000),
                (EnvelopeId::Seed.account::<Runtime>(), 75_000_000),
                (EnvelopeId::ICO2.account::<Runtime>(), 30_000_000),
                (EnvelopeId::SerieA.account::<Runtime>(), 80_000_000)
            ]
        },
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
                            im_online: x.3.clone(),
                        },
                    )
                })
                .collect::<Vec<_>>(),
            non_authority_keys: Default::default(),
        },
        sudo: pallet_sudo::GenesisConfig {
            key: Some(root_key)
        },
        token_allocation: pallet_token_allocation::GenesisConfig {
            envelopes: vec![
                (
                    EnvelopeId::Founders,
                    EnvelopeConfig {
                        total_cap: 67_000_000,
                        upfront_rate: Percent::from_percent(0),
                        cliff: 12 * MONTHS,
                        vesting_duration: 36 * MONTHS
                    }
                ),
                (
                    EnvelopeId::KoL,
                    EnvelopeConfig {
                        total_cap: 3_000_000,
                        upfront_rate: Percent::from_percent(0),
                        cliff: 9 * MONTHS,
                        vesting_duration: 9 * MONTHS
                    }
                ),
                (
                    EnvelopeId::Private1,
                    EnvelopeConfig {
                        total_cap: 120_000_000,
                        upfront_rate: Percent::from_percent(5),
                        cliff: 8 * MONTHS,
                        vesting_duration: 38 * MONTHS
                    }
                ),
                (
                    EnvelopeId::Private2,
                    EnvelopeConfig {
                        total_cap: 80_000_000,
                        upfront_rate: Percent::from_percent(5),
                        cliff: 3 * MONTHS,
                        vesting_duration: 36 * MONTHS
                    }
                ),
                (
                    EnvelopeId::ICO1,
                    EnvelopeConfig {
                        total_cap: 600_000,
                        upfront_rate: Percent::from_percent(0),
                        cliff: 0u32,
                        vesting_duration: 6 * MONTHS
                    }
                ),
                (
                    EnvelopeId::ICO2,
                    EnvelopeConfig {
                        total_cap: 1_200_000,
                        upfront_rate: Percent::from_percent(0),
                        cliff: 0u32,
                        vesting_duration: 6 * MONTHS
                    }
                ),
                (
                    EnvelopeId::Seed,
                    EnvelopeConfig {
                        total_cap: 1_500_000,
                        upfront_rate: Percent::from_percent(0),
                        cliff: 18 * MONTHS,
                        vesting_duration: 12 * MONTHS
                    }
                ),
                (
                    EnvelopeId::SerieA,
                    EnvelopeConfig {
                        total_cap: 4_000_000,
                        upfront_rate: Percent::from_percent(0),
                        cliff: 12 * MONTHS,
                        vesting_duration: 12 * MONTHS
                    }
                ),
            ]
        }
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
