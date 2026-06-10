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

//! One-shot solo→parachain transition of the live `allfeat-melodie-3` chain.
//!
//! The solo chain's final act is a `sudo.setCode(<this wasm>)` enacted at
//! block `H` (mandatory: on-chain `:code` must equal the PVF registered on the
//! relay; it also halts the solo node, which cannot author under a runtime
//! requiring the `set_validation_data` inherent). The para is registered with
//! `genesis_head = header(H)` and collators take over the solo database;
//! `Executive` then runs [`Migrations`] inside `H+1` like any runtime upgrade.
//!
//! [`TransitionToParachain`] is guarded by the presence of the solo-only
//! `Validators` pallet storage: inert on fresh chains, non-reentrant. The
//! module is removable once the migration has been enacted on-chain.

use alloc::vec::Vec;
use codec::{Decode, Encode};
use polkadot_sdk::{
    cumulus_primitives_core::ParaId,
    frame_support::{
        pallet_prelude::ValueQuery,
        storage::{
            migration::{get_storage_value, have_storage_value, storage_key_iter},
            unhashed,
        },
        storage_alias,
        traits::{fungible::MutateHold, tokens::Precision, ConstU32, OnRuntimeUpgrade},
        weights::Weight,
        BoundedVec, Twox64Concat,
    },
    frame_system, pallet_aura, pallet_balances, pallet_collator_selection, pallet_preimage,
    pallet_safe_mode, pallet_session,
    sp_consensus_aura::sr25519::AuthorityId as AuraId,
    sp_core::crypto::KeyTypeId,
    sp_io::hashing::twox_128,
};

use crate::{
    configs::{MiddsDepositBase, MiddsDepositPerByte},
    AccountId, Balance, Balances, Runtime, RuntimeHoldReason, SessionKeys, EXISTENTIAL_DEPOSIT,
    PARA_ID,
};

/// No ATS migration here: the live chain is already at ATS storage version 2,
/// the layout this runtime expects.
pub type Migrations = (TransitionToParachain,);

/// Solo-chain `SessionKeys` layout: `{ grandpa: ed25519, aura: sr25519 }`.
#[derive(Encode, Decode)]
struct OldSessionKeys {
    grandpa: [u8; 32],
    aura: AuraId,
}

/// Mirror of `pallet_balances`' hold entry (`IdAmount`) encoding.
#[derive(Encode, Decode)]
struct HoldEntry<Reason> {
    id: Reason,
    amount: Balance,
}

/// Inner hold reason of the in-repo `pallet_midds` running live at @201.
#[derive(Encode, Decode, Clone, Copy, PartialEq, Eq)]
enum LegacyMiddsHoldReason {
    #[codec(index = 0)]
    MiddsRegistration,
}

/// `RuntimeHoldReason` as laid out by the live @201 runtime (hold-bearing
/// pallets only). 102/103/104 are the OLD in-repo MIDDS instances, undecodable
/// under the current enum (their replacements live at 106-108); the others
/// carry over byte-identical.
#[derive(Encode, Decode, Clone, Copy, PartialEq, Eq)]
enum LegacyHoldReason {
    #[codec(index = 15)]
    Preimage(pallet_preimage::HoldReason),
    #[codec(index = 18)]
    SafeMode(pallet_safe_mode::HoldReason),
    #[codec(index = 102)]
    MusicalWorks(LegacyMiddsHoldReason),
    #[codec(index = 103)]
    Recordings(LegacyMiddsHoldReason),
    #[codec(index = 104)]
    Releases(LegacyMiddsHoldReason),
    #[codec(index = 105)]
    Ats(pallet_ats::HoldReason),
}

impl LegacyHoldReason {
    fn into_current(self) -> Option<RuntimeHoldReason> {
        match self {
            Self::Preimage(r) => Some(RuntimeHoldReason::Preimage(r)),
            Self::SafeMode(r) => Some(RuntimeHoldReason::SafeMode(r)),
            Self::Ats(r) => Some(RuntimeHoldReason::Ats(r)),
            Self::MusicalWorks(_) | Self::Recordings(_) | Self::Releases(_) => None,
        }
    }
}

/// `cumulus_pallet_aura_ext::Authorities` is `pub(crate)` — the alias NAME
/// must match the storage item name.
#[storage_alias]
type Authorities = StorageValue<AuraExt, BoundedVec<AuraId, ConstU32<100_000>>, ValueQuery>;

/// `staging_parachain_info::ParachainId` is `pub(super)`.
#[storage_alias]
type ParachainId = StorageValue<ParachainInfo, ParaId, ValueQuery>;

const GRANDPA_KEY_TYPE: KeyTypeId = KeyTypeId(*b"gran");

fn is_solo_state() -> bool {
    have_storage_value(b"Validators", b"Validators", &[])
}

fn clear_pallet_prefix(pallet_name: &[u8]) -> u64 {
    let res = unhashed::clear_prefix(&twox_128(pallet_name), None, None);
    res.backend as u64
}

/// One-shot transition of the imported solo state to the parachain layout:
/// solo validators → `CollatorSelection::Invulnerables` (no Aura key rotation,
/// `Aura::Authorities` untouched); dead solo pallet prefixes killed; session
/// keys re-encoded `{grandpa, aura}` → `{aura}`; parachain-only pallets
/// seeded; the three MIDDS instances reset (pre-202 layout with no migration
/// path) with every MIDDS bond released back to its payer. ATS state is NOT
/// touched.
pub struct TransitionToParachain;

impl OnRuntimeUpgrade for TransitionToParachain {
    fn on_runtime_upgrade() -> Weight {
        if !is_solo_state() {
            log::info!(
                target: "runtime::solo-to-para",
                "no solo-chain state detected, TransitionToParachain skipped"
            );
            return <Runtime as frame_system::Config>::DbWeight::get().reads(1);
        }

        let mut ops: u64 = 0;

        // 1. Solo validator set → initial collator set.
        let validators = get_storage_value::<Vec<AccountId>>(b"Validators", b"Validators", &[])
            .unwrap_or_else(|| pallet_session::Validators::<Runtime>::get());
        ops += 2;

        // 2. Dead solo pallets. Their indices are reused by CollatorSelection /
        // AuraExt / WeightReclaim, but prefixes derive from pallet NAMES, so
        // this cannot touch the new pallets' state.
        for pallet in [&b"Validators"[..], &b"Grandpa"[..], &b"Historical"[..]] {
            ops += clear_pallet_prefix(pallet);
        }

        // 3a. Session: re-encode `NextKeys` in place, {grandpa, aura} → {aura}.
        let next_keys: Vec<(AccountId, OldSessionKeys)> =
            storage_key_iter::<AccountId, OldSessionKeys, Twox64Concat>(b"Session", b"NextKeys")
                .collect();
        for (who, old) in next_keys {
            pallet_session::NextKeys::<Runtime>::insert(&who, SessionKeys { aura: old.aura });
            ops += 2;
        }

        // 3b. Same for the queued key schedule.
        if let Some(queued) =
            get_storage_value::<Vec<(AccountId, OldSessionKeys)>>(b"Session", b"QueuedKeys", &[])
        {
            let queued: Vec<(AccountId, SessionKeys)> = queued
                .into_iter()
                .map(|(who, old)| (who, SessionKeys { aura: old.aura }))
                .collect();
            pallet_session::QueuedKeys::<Runtime>::put(queued);
            ops += 2;
        }

        // 3c. Drop the grandpa `KeyOwner` entries (aura ones stay valid).
        let gran_owners: Vec<(KeyTypeId, Vec<u8>)> = pallet_session::KeyOwner::<Runtime>::iter()
            .map(|(key, _owner)| key)
            .filter(|(key_type, _)| *key_type == GRANDPA_KEY_TYPE)
            .collect();
        for key in gran_owners {
            pallet_session::KeyOwner::<Runtime>::remove(&key);
            ops += 1;
        }
        pallet_session::DisabledValidators::<Runtime>::kill();
        ops += 1;

        // 4a. CollatorSelection: candidacy stays closed until governance opens it.
        let mut invulnerables = validators;
        invulnerables.sort();
        let count = invulnerables.len();
        let invulnerables: BoundedVec<
            AccountId,
            <Runtime as pallet_collator_selection::Config>::MaxInvulnerables,
        > = BoundedVec::truncate_from(invulnerables);
        if invulnerables.len() < count {
            log::warn!(
                target: "runtime::solo-to-para",
                "invulnerable set truncated from {count} to {}",
                invulnerables.len()
            );
        }
        pallet_collator_selection::Invulnerables::<Runtime>::put(invulnerables);
        pallet_collator_selection::DesiredCandidates::<Runtime>::put(0);
        pallet_collator_selection::CandidacyBond::<Runtime>::put(EXISTENTIAL_DEPOSIT * 16);
        ops += 3;

        // 4b. AuraExt. NOTE: the LOAD-BEARING copy is written by the solo
        // chain's final block (`cutover.mjs set-code` batches
        // `setStorage(AuraExt::Authorities)` with the `setCode`) because the
        // PVF seal check reads it from the PARENT state, BEFORE this migration
        // runs. Seeding here is idempotent reinforcement only.
        Authorities::put(pallet_aura::Authorities::<Runtime>::get());
        ops += 2;

        // 4c. Must match the ID the para is registered under.
        ParachainId::put(ParaId::from(PARA_ID));
        ops += 1;

        // 5a. MIDDS: release every bond. The live @201 chain encodes holds
        // under the OLD pallet indices, undecodable as the current enum —
        // `Holds` is walked RAW (`iter()` would silently skip those entries):
        // current-format entries take the typed release path, legacy entries
        // are rebuilt by hand mirroring `pallet_balances`' bookkeeping.
        let hold_accounts: Vec<AccountId> =
            pallet_balances::Holds::<Runtime>::iter_keys().collect();
        let mut released_bonds: u64 = 0;
        let mut legacy_rewrites: u64 = 0;
        for who in hold_accounts {
            let key = pallet_balances::Holds::<Runtime>::hashed_key_for(&who);
            let Some(raw) = unhashed::get_raw(&key) else { continue };
            ops += 1;

            if let Ok(holds) = <Vec<HoldEntry<RuntimeHoldReason>>>::decode(&mut &raw[..]) {
                for hold in holds {
                    if matches!(
                        hold.id,
                        RuntimeHoldReason::MusicalWorks(_)
                            | RuntimeHoldReason::Recordings(_)
                            | RuntimeHoldReason::Releases(_)
                    ) {
                        let _ = <Balances as MutateHold<AccountId>>::release(
                            &hold.id,
                            &who,
                            hold.amount,
                            Precision::BestEffort,
                        );
                        released_bonds += 1;
                        ops += 4;
                    }
                }
                continue;
            }

            let Ok(legacy) = <Vec<HoldEntry<LegacyHoldReason>>>::decode(&mut &raw[..]) else {
                log::error!(
                    target: "runtime::solo-to-para",
                    "Holds entry neither current nor @201 format, left untouched"
                );
                continue;
            };
            let mut kept: Vec<HoldEntry<RuntimeHoldReason>> = Vec::new();
            let mut released: Balance = 0;
            for entry in legacy {
                match entry.id.into_current() {
                    Some(id) => kept.push(HoldEntry { id, amount: entry.amount }),
                    None => {
                        released = released.saturating_add(entry.amount);
                        released_bonds += 1;
                    }
                }
            }
            if released == 0 {
                continue;
            }
            legacy_rewrites += 1;
            if kept.is_empty() {
                unhashed::kill(&key);
            } else {
                unhashed::put_raw(&key, &kept.encode());
            }

            // `try_mutate_account` semantics (pallet-balances): a consumer ref
            // is held while `reserved != 0 || frozen != 0`; a provider ref
            // while `free >= ED`.
            let account = frame_system::Account::<Runtime>::get(&who);
            let did_provide = account.data.free >= EXISTENTIAL_DEPOSIT;
            let did_consume = account.data.reserved != 0 || account.data.frozen != 0;
            let new_free = account.data.free.saturating_add(released);
            let new_reserved = account.data.reserved.saturating_sub(released);
            frame_system::Account::<Runtime>::mutate(&who, |a| {
                a.data.free = new_free;
                a.data.reserved = new_reserved;
            });
            if !did_provide && new_free >= EXISTENTIAL_DEPOSIT {
                frame_system::Pallet::<Runtime>::inc_providers(&who);
            }
            if did_consume && new_reserved == 0 && account.data.frozen == 0 {
                frame_system::Pallet::<Runtime>::dec_consumers(&who);
            }
            ops += 4;
        }

        // 5b. Clear the three instances and re-seed the bond calibration; the
        // dynamic multipliers fall back to their `OnEmpty` (1.0) values.
        for pallet in [&b"MusicalWorks"[..], &b"Recordings"[..], &b"Releases"[..]] {
            ops += clear_pallet_prefix(pallet);
        }
        pallet_midds::DepositBase::<Runtime, pallet_midds::Instance1>::put(MiddsDepositBase::get());
        pallet_midds::DepositPerByte::<Runtime, pallet_midds::Instance1>::put(
            MiddsDepositPerByte::get(),
        );
        pallet_midds::DepositBase::<Runtime, pallet_midds::Instance2>::put(MiddsDepositBase::get());
        pallet_midds::DepositPerByte::<Runtime, pallet_midds::Instance2>::put(
            MiddsDepositPerByte::get(),
        );
        pallet_midds::DepositBase::<Runtime, pallet_midds::Instance3>::put(MiddsDepositBase::get());
        pallet_midds::DepositPerByte::<Runtime, pallet_midds::Instance3>::put(
            MiddsDepositPerByte::get(),
        );
        ops += 6;

        log::info!(
            target: "runtime::solo-to-para",
            "solo→para transition done: {released_bonds} MIDDS bonds released \
             ({legacy_rewrites} legacy hold entries rewritten), ~{ops} storage ops"
        );

        // Rough accounting; the real cost (15.1 KiB PoV compressed) was
        // measured in rehearsal.
        <Runtime as frame_system::Config>::DbWeight::get().reads_writes(ops, ops)
    }

    #[cfg(feature = "try-runtime")]
    fn pre_upgrade() -> Result<Vec<u8>, polkadot_sdk::sp_runtime::TryRuntimeError> {
        Ok(is_solo_state().encode())
    }

    #[cfg(feature = "try-runtime")]
    fn post_upgrade(state: Vec<u8>) -> Result<(), polkadot_sdk::sp_runtime::TryRuntimeError> {
        let was_solo = bool::decode(&mut &state[..])
            .map_err(|_| polkadot_sdk::sp_runtime::TryRuntimeError::Other("bad pre_upgrade state"))?;
        if !was_solo {
            return Ok(());
        }
        polkadot_sdk::frame_support::ensure!(!is_solo_state(), "solo `Validators` storage must be gone");
        polkadot_sdk::frame_support::ensure!(
            ParachainId::get() == ParaId::from(PARA_ID),
            "ParachainInfo must be seeded"
        );
        polkadot_sdk::frame_support::ensure!(
            !pallet_collator_selection::Invulnerables::<Runtime>::get().is_empty(),
            "invulnerable collators must be seeded"
        );
        polkadot_sdk::frame_support::ensure!(
            !pallet_session::QueuedKeys::<Runtime>::get().is_empty(),
            "queued session keys must be re-encoded"
        );
        for who in pallet_balances::Holds::<Runtime>::iter_keys() {
            let key = pallet_balances::Holds::<Runtime>::hashed_key_for(&who);
            let raw = unhashed::get_raw(&key).ok_or(
                polkadot_sdk::sp_runtime::TryRuntimeError::Other("hold key without value"),
            )?;
            let holds =
                <Vec<HoldEntry<RuntimeHoldReason>>>::decode(&mut &raw[..]).map_err(|_| {
                    polkadot_sdk::sp_runtime::TryRuntimeError::Other(
                        "legacy hold entry survived the transition",
                    )
                })?;
            for hold in holds {
                polkadot_sdk::frame_support::ensure!(
                    !matches!(
                        hold.id,
                        RuntimeHoldReason::MusicalWorks(_)
                            | RuntimeHoldReason::Recordings(_)
                            | RuntimeHoldReason::Releases(_)
                    ),
                    "all MIDDS holds must be released"
                );
            }
        }
        Ok(())
    }
}
