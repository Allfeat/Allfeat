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

#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

use frame_support::pallet_prelude::*;
use frame_support::traits::fungible::Inspect;
use frame_support::{
    PalletId,
    traits::{
        fungible::{Mutate, MutateHold, Unbalanced},
        tokens::{Fortitude, Precision, Preservation},
    },
};
use frame_system::pallet_prelude::OriginFor;
use frame_system::pallet_prelude::*;
use serde::{Deserialize, Serialize};
use sp_runtime::Percent;
use sp_runtime::traits::{AccountIdConversion, Saturating, Zero};

type EnvConfigOf<T> =
    EnvelopeConfig<BalanceOf<T>, BlockNumberFor<T>, <T as frame_system::Config>::AccountId>;
type AllocationOf<T> = Allocation<BalanceOf<T>, BlockNumberFor<T>>;
type InitialAllocation<T> = (
    EnvelopeId,
    <T as frame_system::Config>::AccountId,
    BalanceOf<T>,
    Option<BlockNumberFor<T>>,
);

pub type BalanceOf<T> =
    <<T as Config>::Currency as Inspect<<T as frame_system::Config>::AccountId>>::Balance;

#[derive(
    Encode,
    Decode,
    DecodeWithMemTracking,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Debug,
    TypeInfo,
    MaxEncodedLen,
    Serialize,
    Deserialize,
)]
pub enum EnvelopeId {
    Founders,
    KoL,
    Private1,
    Private2,
    ICO1,
    Seed,
    ICO2,
    SerieA,
    Airdrop,
    CommunityRewards,
    Exchanges,
    ResearchDevelopment,
    Reserve,
}

impl EnvelopeId {
    pub fn account<T: pallet::Config>(&self) -> T::AccountId {
        let pid = <T as pallet::Config>::PalletId::get();
        pid.into_sub_account_truncating(*self as u8)
    }
}

#[derive(
    Encode,
    Decode,
    Clone,
    PartialEq,
    Eq,
    RuntimeDebug,
    TypeInfo,
    MaxEncodedLen,
    Serialize,
    Deserialize,
)]
pub struct EnvelopeConfig<Balance, BlockNumber, AccountId> {
    pub total_cap: Balance,
    pub upfront_rate: Percent,
    pub cliff: BlockNumber,
    pub vesting_duration: BlockNumber,
    pub unique_beneficiary: Option<AccountId>,
}

#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub struct Allocation<Balance, BlockNumber> {
    pub envelope: EnvelopeId,
    pub total: Balance,
    pub upfront: Balance,
    pub vested_total: Balance,
    pub released: Balance,
    pub start: BlockNumber,
}

#[frame_support::pallet(dev_mode)]
pub mod pallet {
    use super::*;

    #[pallet::config]
    pub trait Config: frame_system::Config {
        type Currency: MutateHold<Self::AccountId, Reason = Self::RuntimeHoldReason>
            + Mutate<Self::AccountId>;

        type AdminOrigin: EnsureOrigin<Self::RuntimeOrigin>;

        #[pallet::constant]
        type PalletId: Get<PalletId>;

        #[pallet::constant]
        type MaxAllocations: Get<u32>;

        #[pallet::constant]
        type EpochDuration: Get<BlockNumberFor<Self>>;

        #[pallet::constant]
        type MaxPayoutsPerBlock: Get<u32>;

        /// The overarching HoldReason type.
        type RuntimeHoldReason: From<HoldReason>;
    }

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    #[pallet::composite_enum]
    pub enum HoldReason {
        TokenAllocation,
    }

    #[pallet::storage]
    pub type Envelopes<T: Config> =
        StorageMap<_, Blake2_128Concat, EnvelopeId, EnvConfigOf<T>, OptionQuery>;

    #[pallet::storage]
    pub type EnvelopeDistributed<T: Config> =
        StorageMap<_, Blake2_128Concat, EnvelopeId, BalanceOf<T>, ValueQuery>;

    #[pallet::storage]
    pub type AllocationsOf<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        BoundedVec<Allocation<BalanceOf<T>, BlockNumberFor<T>>, T::MaxAllocations>,
        ValueQuery,
    >;

    #[pallet::storage]
    pub type ProviderRefs<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, u32, ValueQuery>;

    #[pallet::storage]
    pub type NextPayoutAt<T: Config> = StorageValue<_, BlockNumberFor<T>, ValueQuery>;

    #[pallet::storage]
    pub type PayoutCursor<T: Config> = StorageValue<_, (T::AccountId, u32), OptionQuery>;

    #[pallet::storage]
    pub type EpochIndex<T: Config> = StorageValue<_, u64, ValueQuery>;

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        fn on_initialize(now: BlockNumberFor<T>) -> Weight {
            if now < NextPayoutAt::<T>::get() {
                return Weight::zero();
            }

            let mut remaining = T::MaxPayoutsPerBlock::get();
            let mut db_reads: u64 = 0;
            let mut db_writes: u64 = 0;
            let mut total_released = BalanceOf::<T>::zero();

            if let Some((cursor_who, cursor_idx)) = PayoutCursor::<T>::get() {
                db_reads += 1;

                let (rem_after, next_idx_opt, released_delta, writes) =
                    Self::process_account(&cursor_who, cursor_idx, remaining, now);

                remaining = rem_after;
                total_released = total_released.saturating_add(released_delta);
                db_writes = db_writes.saturating_add(writes);

                if remaining == 0 {
                    let idx = next_idx_opt.unwrap_or(0);
                    PayoutCursor::<T>::put((cursor_who, idx));
                    db_writes += 1;
                    return T::DbWeight::get().reads_writes(db_reads, db_writes);
                }

                let mut iter =
                    AllocationsOf::<T>::iter_from(AllocationsOf::<T>::hashed_key_for(&cursor_who));

                if iter.next().is_some() {
                    db_reads += 1;
                }

                for (who, _vec) in iter {
                    db_reads += 1;
                    let (rem_after, _next_idx_opt, released_delta, writes) =
                        Self::process_account(&who, 0u32, remaining, now);
                    remaining = rem_after;
                    total_released = total_released.saturating_add(released_delta);
                    db_writes = db_writes.saturating_add(writes);

                    if remaining == 0 {
                        PayoutCursor::<T>::put((who, 0u32));
                        db_writes += 1;
                        return T::DbWeight::get().reads_writes(db_reads, db_writes);
                    }
                }

                PayoutCursor::<T>::kill();
                let next = now.saturating_add(T::EpochDuration::get());
                NextPayoutAt::<T>::put(next);
                EpochIndex::<T>::mutate(|e| *e = e.saturating_add(1));
                db_writes += 3;

                Self::deposit_event(Event::EpochPayout {
                    epoch: EpochIndex::<T>::get(),
                    at: now,
                    total_released,
                });

                return T::DbWeight::get().reads_writes(db_reads, db_writes);
            }

            for (who, _vec) in AllocationsOf::<T>::iter() {
                db_reads += 1;
                let (rem_after, _next_idx_opt, released_delta, writes) =
                    Self::process_account(&who, 0u32, remaining, now);
                remaining = rem_after;
                total_released = total_released.saturating_add(released_delta);
                db_writes = db_writes.saturating_add(writes);

                if remaining == 0 {
                    PayoutCursor::<T>::put((who, 0u32));
                    db_writes += 1;
                    return T::DbWeight::get().reads_writes(db_reads, db_writes);
                }
            }

            PayoutCursor::<T>::kill();
            let next = now.saturating_add(T::EpochDuration::get());
            NextPayoutAt::<T>::put(next);
            EpochIndex::<T>::mutate(|e| *e = e.saturating_add(1));
            db_writes += 3;

            Self::deposit_event(Event::EpochPayout {
                epoch: EpochIndex::<T>::get(),
                at: now,
                total_released,
            });

            T::DbWeight::get().reads_writes(db_reads, db_writes)
        }
    }

    #[pallet::genesis_config]
    pub struct GenesisConfig<T: Config> {
        pub envelopes: sp_runtime::Vec<(EnvelopeId, EnvConfigOf<T>)>,
        pub initial_allocations: sp_runtime::Vec<InitialAllocation<T>>,
    }

    impl<T: Config> Default for GenesisConfig<T> {
        fn default() -> Self {
            Self {
                envelopes: sp_runtime::Vec::new(),
                initial_allocations: sp_runtime::Vec::new(),
            }
        }
    }

    #[pallet::genesis_build]
    impl<T: Config> BuildGenesisConfig for GenesisConfig<T> {
        fn build(&self) {
            for (id, cfg_in) in &self.envelopes {
                assert!(
                    !Envelopes::<T>::contains_key(id),
                    "duplicate envelope in genesis"
                );
                let cfg: EnvConfigOf<T> =
                    EnvelopeConfig::<BalanceOf<T>, BlockNumberFor<T>, T::AccountId> {
                        total_cap: cfg_in.total_cap,
                        upfront_rate: cfg_in.upfront_rate,
                        cliff: cfg_in.cliff,
                        vesting_duration: cfg_in.vesting_duration,
                        unique_beneficiary: cfg_in.unique_beneficiary.clone(),
                    };

                let envelope_acc = id.account::<T>();
                if !cfg.total_cap.is_zero() {
                    <T as Config>::Currency::mint_into(&envelope_acc, cfg.total_cap)
                        .expect("mint_into should succeed at genesis");
                }

                <T as Config>::Currency::deactivate(cfg.total_cap);

                Envelopes::<T>::insert(id, cfg);
                EnvelopeDistributed::<T>::insert(id, BalanceOf::<T>::zero());
            }

            for (id, who, total, start) in &self.initial_allocations {
                let cfg =
                    Envelopes::<T>::get(id).expect("envelope must be defined before allocations");
                if let Some(enforced) = cfg.unique_beneficiary.clone() {
                    assert!(
                        *who == enforced,
                        "allocation beneficiary must match unique beneficiary"
                    );
                }
                assert!(
                    !AllocationsOf::<T>::contains_key(who),
                    "duplicate allocation in genesis"
                );

                Pallet::<T>::do_add_allocation(*id, who, *total, *start, &cfg, false)
                    .expect("genesis allocation must succeed");
            }

            for (id, _) in &self.envelopes {
                let cfg = Envelopes::<T>::get(id).expect("envelope must exist");
                if let Some(benef) = cfg.unique_beneficiary.clone() {
                    let already = EnvelopeDistributed::<T>::get(id);
                    if already < cfg.total_cap {
                        let remaining = cfg.total_cap.saturating_sub(already);
                        Pallet::<T>::do_add_allocation(*id, &benef, remaining, None, &cfg, false)
                            .expect("auto unique beneficiary allocation must succeed");
                    }
                }
            }

            // Epoch payout init
            NextPayoutAt::<T>::put(T::EpochDuration::get());
            EpochIndex::<T>::put(0u64);
            PayoutCursor::<T>::kill();
        }
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        AllocationAdded(EnvelopeId, T::AccountId, BalanceOf<T>),
        UpfrontPaid(EnvelopeId, T::AccountId, BalanceOf<T>),
        VestedReleased(EnvelopeId, T::AccountId, BalanceOf<T>),
        EpochPayout {
            epoch: u64,
            at: BlockNumberFor<T>,
            total_released: BalanceOf<T>,
        },
    }

    #[pallet::error]
    pub enum Error<T> {
        EnvelopeUnknown,
        AllocationExists,
        EnvelopeCapExceeded,
        ArithmeticOverflow,
        TooMuchAllocations,
        AllocationDisabled,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::call_index(0)]
        pub fn add_allocation(
            origin: OriginFor<T>,
            id: EnvelopeId,
            who: T::AccountId,
            total: BalanceOf<T>,
            start: Option<frame_system::pallet_prelude::BlockNumberFor<T>>,
        ) -> DispatchResult {
            T::AdminOrigin::ensure_origin(origin)?;

            let cfg = Envelopes::<T>::get(id).ok_or(Error::<T>::EnvelopeUnknown)?;

            if cfg.unique_beneficiary.is_some() {
                return Err(Error::<T>::AllocationDisabled.into());
            }

            ensure!(
                !AllocationsOf::<T>::contains_key(&who),
                Error::<T>::AllocationExists
            );

            Self::do_add_allocation(id, &who, total, start, &cfg, true)?;
            Self::deposit_event(Event::AllocationAdded(id, who, total));
            Ok(())
        }
    }

    impl<T: Config> Pallet<T> {
        fn ensure_provider(who: &T::AccountId) {
            if ProviderRefs::<T>::get(who) == 0 {
                frame_system::Pallet::<T>::inc_providers(who);
            }
            ProviderRefs::<T>::mutate(who, |c| *c = c.saturating_add(1));
        }

        pub fn release_provider(who: &T::AccountId) {
            ProviderRefs::<T>::mutate(who, |c| {
                if *c != 0 {
                    let _ = frame_system::Pallet::<T>::dec_providers(who);
                    *c = 0u32;
                }
            });
        }

        fn do_add_allocation(
            id: EnvelopeId,
            who: &T::AccountId,
            total: BalanceOf<T>,
            start: Option<BlockNumberFor<T>>,
            cfg: &EnvConfigOf<T>,
            emit_events: bool,
        ) -> DispatchResult {
            let distributed = EnvelopeDistributed::<T>::get(id);
            let new_distributed = distributed.saturating_add(total);
            ensure!(
                new_distributed <= cfg.total_cap,
                Error::<T>::EnvelopeCapExceeded
            );

            let source = id.account::<T>();
            let upfront = cfg.upfront_rate.mul_floor(total);
            let vested_total = total.saturating_sub(upfront);
            let start_block = start.unwrap_or(cfg.cliff);
            let alloc = Allocation {
                envelope: id,
                total,
                upfront,
                vested_total,
                released: Zero::zero(),
                start: start_block,
            };
            let reason: T::RuntimeHoldReason = HoldReason::TokenAllocation.into();

            Self::ensure_provider(who);
            <T as Config>::Currency::transfer_and_hold(
                &reason,
                &source,
                who,
                total,
                Precision::Exact,
                Preservation::Expendable,
                Fortitude::Polite,
            )?;

            if !upfront.is_zero() {
                <T as Config>::Currency::release(&reason, who, upfront, Precision::Exact)?;

                <T as Config>::Currency::reactivate(upfront);

                if emit_events {
                    Self::deposit_event(Event::UpfrontPaid(id, who.clone(), upfront));
                }
            }
            if alloc.vested_total.is_zero() {
                Self::release_provider(who);
                EnvelopeDistributed::<T>::insert(id, new_distributed);
                return Ok(());
            }

            AllocationsOf::<T>::try_mutate(who, |vec| -> DispatchResult {
                vec.try_push(alloc.clone())
                    .map_err(|_| Error::<T>::TooMuchAllocations)?;

                Ok(())
            })?;

            EnvelopeDistributed::<T>::insert(id, new_distributed);

            Ok(())
        }

        pub fn claimable_amount(
            cfg: &EnvConfigOf<T>,
            alloc: &AllocationOf<T>,
            now: BlockNumberFor<T>,
        ) -> Option<BalanceOf<T>> {
            let effective_start = core::cmp::max(alloc.start, cfg.cliff);
            if now <= effective_start {
                return Some(Zero::zero());
            }
            let elapsed = now.saturating_sub(effective_start);
            if elapsed >= cfg.vesting_duration {
                return alloc.vested_total.saturating_sub(alloc.released).into();
            }
            let vested = Self::mul_div(alloc.vested_total, elapsed, cfg.vesting_duration)?;
            let available = vested.saturating_sub(alloc.released);
            Some(available)
        }

        fn process_account(
            who: &T::AccountId,
            start_idx: u32,
            mut remaining: u32,
            now: BlockNumberFor<T>,
        ) -> (u32, Option<u32>, BalanceOf<T>, u64) {
            let mut writes: u64 = 0;
            let mut total_released = BalanceOf::<T>::zero();

            AllocationsOf::<T>::mutate(who, |vec| {
                if (start_idx as usize) > vec.len() {
                    return;
                }

                let mut i = start_idx as usize;
                while remaining > 0 && i < vec.len() {
                    let alloc = &mut vec[i];

                    let id = alloc.envelope;

                    let Some(cfg) = Envelopes::<T>::get(id) else {
                        vec.swap_remove(i);
                        writes += 1;
                        continue;
                    };

                    if let Some(claimable) = Self::claimable_amount(&cfg, alloc, now) {
                        if !claimable.is_zero() {
                            let reason: T::RuntimeHoldReason = HoldReason::TokenAllocation.into();
                            if T::Currency::release(&reason, who, claimable, Precision::Exact)
                                .is_ok()
                            {
                                alloc.released = alloc.released.saturating_add(claimable);
                                total_released = total_released.saturating_add(claimable);

                                <T as Config>::Currency::reactivate(claimable);

                                let done =
                                    alloc.vested_total.saturating_sub(alloc.released).is_zero();
                                if done {
                                    vec.swap_remove(i);
                                    writes += 1;
                                    remaining = remaining.saturating_sub(1);
                                    continue;
                                } else {
                                    writes += 1;
                                }
                            }
                        }
                    }

                    i += 1;
                    remaining = remaining.saturating_sub(1);
                }
            });

            let mut next_idx: Option<u32> = None;
            if !AllocationsOf::<T>::get(who).is_empty() {
                if remaining == 0 {
                    next_idx = Some(0u32);
                }
            } else {
                Self::release_provider(who);
                writes += 1;
            }

            (remaining, next_idx, total_released, writes)
        }

        pub fn mul_div(
            a: BalanceOf<T>,
            b: frame_system::pallet_prelude::BlockNumberFor<T>,
            c: frame_system::pallet_prelude::BlockNumberFor<T>,
        ) -> Option<BalanceOf<T>> {
            let a128: u128 = a.try_into().ok()?;
            let b128: u128 = b.try_into().ok()?;
            let c128: u128 = c.try_into().ok()?;
            if c128 == 0 {
                return None;
            }
            let res = a128.checked_mul(b128)?.checked_div(c128)?;
            res.try_into().ok()
        }
    }
}
