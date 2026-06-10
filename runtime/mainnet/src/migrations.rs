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

//! One-shot solo→parachain transition of the live `allfeat` chain (network
//! id `allfeat_staging`), following the Melodie playbook: the solo chain's
//! final act is a `sudo.setCode(<this wasm>)` batched with
//! `setStorage(AuraExt::Authorities)` enacted at block `H`, the para is
//! registered with `genesis_head = header(H)` and collators take over the
//! solo database; `Executive` then runs [`Migrations`] inside `H+1`.
//!
//! Unlike Melodie there is no MIDDS state here, and every hold-bearing
//! pallet keeps its solo index (Preimage 15, TokenAllocation 18, Ats 105),
//! so `Balances::Holds` decodes unchanged and no hold is touched.
//!
//! [`FixPublic2Cliff`] is the solo-era tokenomics data fix, carried over
//! defensively: it is guarded and idempotent, a no-op if the live chain has
//! already enacted it.

use alloc::vec::Vec;
use codec::{Decode, Encode};
use polkadot_sdk::{
	cumulus_primitives_core::ParaId,
	frame_support::{
		BoundedVec, Twox64Concat,
		pallet_prelude::ValueQuery,
		storage::{
			migration::{get_storage_value, have_storage_value, storage_key_iter},
			unhashed,
		},
		storage_alias,
		traits::{ConstU32, OnRuntimeUpgrade},
		weights::Weight,
	},
	frame_system, pallet_aura, pallet_collator_selection, pallet_session,
	sp_consensus_aura::sr25519::AuthorityId as AuraId,
	sp_core::crypto::KeyTypeId,
	sp_io::hashing::twox_128,
};

use crate::{AccountId, BlockNumber, EXISTENTIAL_DEPOSIT, MONTHS, PARA_ID, Runtime, SessionKeys};
use pallet_token_allocation::{Allocations, EnvelopeId, Envelopes};

/// No ATS migration here: the live chain runs the same `pallet-ats` 0.4.0
/// lineage as this runtime (storage version 2) — to be re-verified against
/// the live RPC during the phase-0 try-runtime, as was done for Melodie.
pub type Migrations = (FixPublic2Cliff, TransitionToParachain);

// ---------------------------------------------------------------------------
// FixPublic2Cliff (inherited from the solo runtime)
// ---------------------------------------------------------------------------

/// The Public2 envelope was historically configured with an erroneous
/// 18-month cliff, contradicting the signed contributor contracts (12
/// months). Lowers the envelope cliff and resets the `start` of every
/// Public2 allocation so the corrected envelope cliff applies.
pub struct FixPublic2Cliff;

const WRONG_CLIFF: BlockNumber = 18 * MONTHS;
const CORRECT_CLIFF: BlockNumber = 12 * MONTHS;

#[cfg(feature = "try-runtime")]
type SnapAlloc = pallet_token_allocation::Allocation<AccountId, crate::Balance, BlockNumber>;
#[cfg(feature = "try-runtime")]
type SnapEnv = pallet_token_allocation::EnvelopeConfig<crate::Balance, BlockNumber, AccountId>;

impl OnRuntimeUpgrade for FixPublic2Cliff {
	fn on_runtime_upgrade() -> Weight {
		let db = <Runtime as frame_system::Config>::DbWeight::get();

		// Guard: read the current Public2 envelope. Bail out (no-op) unless
		// the erroneous 18-month cliff is still in place. Makes the migration
		// idempotent if it is run more than once or left in the tuple.
		let cfg = match Envelopes::<Runtime>::get(EnvelopeId::Public2) {
			Some(cfg) if cfg.cliff == WRONG_CLIFF => cfg,
			_ => return db.reads(1),
		};

		// 1. Collect the ids of every Public2 allocation.
		//
		// We collect first to avoid mutating the map while a lazy iterator
		// over it is live. `iter()` is a full scan, so we also record exactly
		// how many entries were read for accurate weight accounting.
		// Allocation count is small (one entry per contributor), so an
		// in-memory Vec is fine for a one-shot upgrade.
		let mut total_iterated: u64 = 0;
		let mut ids: Vec<u32> = Vec::new();
		for (id, alloc) in Allocations::<Runtime>::iter() {
			total_iterated = total_iterated.saturating_add(1);
			if alloc.envelope == EnvelopeId::Public2 {
				ids.push(id);
			}
		}

		let touched = ids.len() as u64;

		// 2. Reset `start` to 0 for every Public2 allocation.
		for id in ids {
			Allocations::<Runtime>::mutate(id, |maybe_alloc| {
				if let Some(alloc) = maybe_alloc {
					alloc.start = 0;
				}
			});
		}

		// 3. Lower the envelope cliff itself.
		Envelopes::<Runtime>::insert(
			EnvelopeId::Public2,
			pallet_token_allocation::EnvelopeConfig { cliff: CORRECT_CLIFF, ..cfg },
		);

		log::info!(
			target: "runtime::migration",
			"FixPublic2Cliff: cliff {WRONG_CLIFF} -> {CORRECT_CLIFF}, reset start on {touched} allocation(s)",
		);

		db.reads_writes(
			total_iterated.saturating_add(touched).saturating_add(1),
			touched.saturating_add(1),
		)
	}

	#[cfg(feature = "try-runtime")]
	fn pre_upgrade() -> Result<Vec<u8>, polkadot_sdk::sp_runtime::TryRuntimeError> {
		let cfg = Envelopes::<Runtime>::get(EnvelopeId::Public2)
			.ok_or("pre_upgrade: Public2 envelope missing")?;
		polkadot_sdk::frame_support::ensure!(
			cfg.cliff == WRONG_CLIFF || cfg.cliff == CORRECT_CLIFF,
			"pre_upgrade: Public2 cliff is neither the erroneous 18*MONTHS nor the corrected 12*MONTHS value"
		);

		let mut public2: Vec<(u32, SnapAlloc)> = Vec::new();
		let mut others: Vec<(u32, SnapAlloc)> = Vec::new();
		for (id, a) in Allocations::<Runtime>::iter() {
			if a.envelope == EnvelopeId::Public2 {
				public2.push((id, a));
			} else {
				others.push((id, a));
			}
		}
		let envelopes: Vec<(EnvelopeId, SnapEnv)> = Envelopes::<Runtime>::iter().collect();

		Ok((public2, others, envelopes).encode())
	}

	#[cfg(feature = "try-runtime")]
	fn post_upgrade(state: Vec<u8>) -> Result<(), polkadot_sdk::sp_runtime::TryRuntimeError> {
		type Snapshot = (Vec<(u32, SnapAlloc)>, Vec<(u32, SnapAlloc)>, Vec<(EnvelopeId, SnapEnv)>);
		let (pre_public2, pre_others, pre_envelopes): Snapshot = Decode::decode(&mut &state[..])
			.map_err(|_| "post_upgrade: failed to decode pre-state")?;

		// 1. The Public2 envelope: only `cliff` changed, and it is correct.
		let new_cfg = Envelopes::<Runtime>::get(EnvelopeId::Public2)
			.ok_or("post_upgrade: Public2 envelope missing")?;
		polkadot_sdk::frame_support::ensure!(
			new_cfg.cliff == CORRECT_CLIFF,
			"post_upgrade: Public2 cliff was not lowered to 12*MONTHS"
		);
		let pre_public2_env = pre_envelopes
			.iter()
			.find_map(|(eid, c)| (*eid == EnvelopeId::Public2).then(|| c.clone()))
			.ok_or("post_upgrade: Public2 missing from pre-state envelopes")?;
		polkadot_sdk::frame_support::ensure!(
			new_cfg == SnapEnv { cliff: CORRECT_CLIFF, ..pre_public2_env },
			"post_upgrade: a Public2 envelope field other than `cliff` changed"
		);

		// 2. Every other envelope is byte-for-byte unchanged.
		for (eid, pre) in pre_envelopes.iter().filter(|(e, _)| *e != EnvelopeId::Public2) {
			let now = Envelopes::<Runtime>::get(*eid)
				.ok_or("post_upgrade: a non-Public2 envelope disappeared")?;
			polkadot_sdk::frame_support::ensure!(
				now == *pre,
				"post_upgrade: a non-Public2 envelope was modified"
			);
		}
		polkadot_sdk::frame_support::ensure!(
			Envelopes::<Runtime>::iter().count() == pre_envelopes.len(),
			"post_upgrade: the number of envelopes changed"
		);

		// 3. Every Public2 allocation: only `start` changed, and it is now 0.
		for (id, pre) in &pre_public2 {
			let now = Allocations::<Runtime>::get(*id)
				.ok_or("post_upgrade: a Public2 allocation disappeared")?;
			polkadot_sdk::frame_support::ensure!(
				now.start == 0,
				"post_upgrade: a Public2 allocation still has a non-zero start"
			);
			polkadot_sdk::frame_support::ensure!(
				now == SnapAlloc { start: 0, ..pre.clone() },
				"post_upgrade: a Public2 allocation field other than `start` changed"
			);
		}

		// 4. Every non-Public2 allocation is byte-for-byte unchanged.
		for (id, pre) in &pre_others {
			let now = Allocations::<Runtime>::get(*id)
				.ok_or("post_upgrade: a non-Public2 allocation disappeared")?;
			polkadot_sdk::frame_support::ensure!(
				now == *pre,
				"post_upgrade: a non-Public2 allocation was modified"
			);
		}

		// 5. No allocation was added or removed.
		polkadot_sdk::frame_support::ensure!(
			Allocations::<Runtime>::iter().count() == pre_public2.len() + pre_others.len(),
			"post_upgrade: the total allocation count changed"
		);

		Ok(())
	}
}

// ---------------------------------------------------------------------------
// TransitionToParachain
// ---------------------------------------------------------------------------

/// Solo-chain `SessionKeys` layout: `{ grandpa: ed25519, aura: sr25519 }`.
#[derive(Encode, Decode)]
struct OldSessionKeys {
	grandpa: [u8; 32],
	aura: AuraId,
}

/// Mirror of `pallet_balances`' hold entry (`IdAmount`) encoding.
#[cfg(feature = "try-runtime")]
#[derive(Encode, Decode)]
struct HoldEntry<Reason> {
	id: Reason,
	amount: crate::Balance,
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
/// solo validators → `CollatorSelection::Invulnerables` (no Aura key
/// rotation, `Aura::Authorities` untouched); dead solo pallet prefixes
/// killed; session keys re-encoded `{grandpa, aura}` → `{aura}`;
/// parachain-only pallets seeded. Balances holds and every other pallet's
/// state (TokenAllocation, Treasury, ATS, …) carry over byte-identical.
///
/// Guarded by the presence of the solo-only `Validators` pallet storage:
/// inert on fresh chains (dev/local/staging presets), non-reentrant. The
/// module is removable once the migration has been enacted on-chain.
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
			.unwrap_or_else(pallet_session::Validators::<Runtime>::get);
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

		log::info!(
			target: "runtime::solo-to-para",
			"solo→para transition done: ~{ops} storage ops"
		);

		<Runtime as frame_system::Config>::DbWeight::get().reads_writes(ops, ops)
	}

	#[cfg(feature = "try-runtime")]
	fn pre_upgrade() -> Result<Vec<u8>, polkadot_sdk::sp_runtime::TryRuntimeError> {
		Ok(is_solo_state().encode())
	}

	#[cfg(feature = "try-runtime")]
	fn post_upgrade(state: Vec<u8>) -> Result<(), polkadot_sdk::sp_runtime::TryRuntimeError> {
		use polkadot_sdk::pallet_balances;

		let was_solo = bool::decode(&mut &state[..]).map_err(|_| {
			polkadot_sdk::sp_runtime::TryRuntimeError::Other("bad pre_upgrade state")
		})?;
		if !was_solo {
			return Ok(());
		}
		polkadot_sdk::frame_support::ensure!(
			!is_solo_state(),
			"solo `Validators` storage must be gone"
		);
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
		// Every hold-bearing pallet keeps its solo index, so every existing
		// `Holds` entry must decode under the current `RuntimeHoldReason`.
		for who in pallet_balances::Holds::<Runtime>::iter_keys() {
			let key = pallet_balances::Holds::<Runtime>::hashed_key_for(&who);
			let raw = unhashed::get_raw(&key).ok_or(
				polkadot_sdk::sp_runtime::TryRuntimeError::Other("hold key without value"),
			)?;
			<Vec<HoldEntry<crate::RuntimeHoldReason>>>::decode(&mut &raw[..]).map_err(|_| {
				polkadot_sdk::sp_runtime::TryRuntimeError::Other(
					"a hold entry no longer decodes under the new RuntimeHoldReason",
				)
			})?;
		}
		Ok(())
	}
}
