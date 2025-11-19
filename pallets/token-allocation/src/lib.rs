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

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

pub mod weights;
pub use weights::WeightInfo;

use frame_support::pallet_prelude::*;
use frame_support::traits::fungible::Inspect;
use frame_support::{
    PalletId,
    traits::{
        fungible::{Mutate, MutateHold},
        tokens::{Fortitude, Precision, Preservation},
    },
};
use frame_system::pallet_prelude::OriginFor;
use frame_system::pallet_prelude::*;
use serde::{Deserialize, Serialize};
use sp_runtime::Percent;
use sp_runtime::traits::{AccountIdConversion, SaturatedConversion, Saturating, Zero};

type EnvConfigOf<T> =
    EnvelopeConfig<BalanceOf<T>, BlockNumberFor<T>, <T as frame_system::Config>::AccountId>;
type AllocationFor<T> = Allocation<AccountIdFor<T>, BalanceOf<T>, BlockNumberFor<T>>;
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
pub struct Allocation<AccountId, Balance, BlockNumber> {
    pub envelope: EnvelopeId,
    pub beneficiary: AccountId,
    pub total: Balance,
    pub upfront: Balance,
    pub vested_total: Balance,
    pub released: Balance,
    pub start: BlockNumber,
}

type AllocationId = u32;

#[frame_support::pallet]
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
        type EpochDuration: Get<BlockNumberFor<Self>>;

        #[pallet::constant]
        type MaxPayoutsPerBlock: Get<u32>;

        /// The overarching HoldReason type.
        type RuntimeHoldReason: From<HoldReason>;

        type WeightInfo: WeightInfo;
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
    pub type Allocations<T: Config> =
        StorageMap<_, Blake2_128Concat, AllocationId, AllocationFor<T>, OptionQuery>;

    #[pallet::storage]
    pub type NextAllocationId<T: Config> = StorageValue<_, AllocationId, ValueQuery>;

    #[pallet::storage]
    pub type NextPayoutAt<T: Config> = StorageValue<_, BlockNumberFor<T>, ValueQuery>;

    #[pallet::storage]
    pub type PayoutCursor<T: Config> = StorageValue<_, AllocationId, OptionQuery>;

    #[pallet::storage]
    pub type EpochIndex<T: Config> = StorageValue<_, u64, ValueQuery>;

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        fn on_initialize(now: BlockNumberFor<T>) -> Weight {
            // We ensure we are at a new epoch
            let next_epoch_at = NextPayoutAt::<T>::get();
            if now < next_epoch_at {
                return T::WeightInfo::on_initialize_noop();
            }

            let epoch = EpochIndex::<T>::get();
            let current_cursor = PayoutCursor::<T>::get();
            let iter = match current_cursor {
                Some(k) => Allocations::<T>::iter_keys_from_key(k),
                None => Allocations::<T>::iter_keys(),
            };

            let limit = T::MaxPayoutsPerBlock::get();

            let mut payout = 0u32;
            for id in iter {
                if payout < limit {
                    Self::payout_allocation(id, now);

                    // We did a new payout
                    payout += 1;
                } else {
                    // payout limit is enforced, and there is still allocations to
                    // process
                    PayoutCursor::<T>::set(Some(id));

                    Self::deposit_event(Event::EpochPayout {
                        epoch,
                        at: now,
                        cursor: Some(id),
                    });

                    return T::WeightInfo::on_initialize_partial();
                }
            }

            // If we come to the end, then we processed everything without enforcing the
            // limit. We can proceed to the next epoch correctly.
            let next = now.saturating_add(T::EpochDuration::get());
            NextPayoutAt::<T>::put(next);
            PayoutCursor::<T>::set(None);
            EpochIndex::<T>::set(epoch.saturating_add(1));

            Self::deposit_event(Event::EpochPayout {
                epoch,
                at: now,
                cursor: None,
            });

            T::WeightInfo::on_initialize_epoch_finished(payout)
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
        AllocationAdded(AllocationId),
        UpfrontPaid(AllocationId),
        VestedReleased(AllocationId, BalanceOf<T>),
        EpochPayout {
            epoch: u64,
            at: BlockNumberFor<T>,
            cursor: Option<AllocationId>,
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
        #[pallet::weight(T::WeightInfo::add_allocation())]
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

            Self::do_add_allocation(id, &who, total, start, &cfg, true)?;
            Ok(())
        }
    }

    impl<T: Config> Pallet<T> {
        pub(crate) fn do_add_allocation(
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
            let alloc: AllocationFor<T> = Allocation {
                envelope: id,
                beneficiary: who.clone(),
                total,
                upfront,
                vested_total,
                released: Zero::zero(),
                start: start_block,
            };
            let alloc_id = NextAllocationId::<T>::get();
            let reason: T::RuntimeHoldReason = HoldReason::TokenAllocation.into();

            frame_system::Pallet::<T>::inc_providers(who);
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

                if emit_events {
                    Self::deposit_event(Event::UpfrontPaid(alloc_id));
                }
            }
            if alloc.vested_total.is_zero() {
                let _ = frame_system::Pallet::<T>::dec_providers(who);
                EnvelopeDistributed::<T>::insert(id, new_distributed);
                return Ok(());
            }

            Allocations::<T>::insert(alloc_id, alloc);
            EnvelopeDistributed::<T>::insert(id, new_distributed);
            NextAllocationId::<T>::set(alloc_id.saturating_add(1));

            if emit_events {
                Self::deposit_event(Event::AllocationAdded(alloc_id));
            }

            Ok(())
        }

        pub fn claimable_amount(
            cfg: &EnvConfigOf<T>,
            alloc: &AllocationFor<T>,
            now: BlockNumberFor<T>,
        ) -> BalanceOf<T> {
            let effective_start = core::cmp::max(alloc.start, cfg.cliff);
            if now <= effective_start {
                return Zero::zero();
            }
            let elapsed = now.saturating_sub(effective_start);
            if elapsed >= cfg.vesting_duration {
                return alloc.vested_total.saturating_sub(alloc.released);
            }
            let vested = Self::mul_div(alloc.vested_total, elapsed, cfg.vesting_duration);
            vested.saturating_sub(alloc.released)
        }

        pub(crate) fn payout_allocation(id: AllocationId, now: BlockNumberFor<T>) {
            if let Some(mut alloc) = Allocations::<T>::get(id) {
                let cfg = match Envelopes::<T>::get(alloc.envelope) {
                    Some(cfg) => cfg,
                    None => return,
                };
                let claimable = Self::claimable_amount(&cfg, &alloc, now);

                if !claimable.is_zero() {
                    let reason: T::RuntimeHoldReason = HoldReason::TokenAllocation.into();
                    if T::Currency::release(
                        &reason,
                        &alloc.beneficiary,
                        claimable,
                        Precision::Exact,
                    )
                    .is_ok()
                    {
                        alloc.released = alloc.released.saturating_add(claimable);

                        // Allocation is fully vested, we can remove it
                        if alloc.vested_total.saturating_sub(alloc.released).is_zero() {
                            Allocations::<T>::remove(id);
                            let _ = frame_system::Pallet::<T>::dec_providers(&alloc.beneficiary);
                        } else {
                            Allocations::<T>::insert(id, alloc);
                        }

                        Self::deposit_event(Event::VestedReleased(id, claimable));
                    }
                }
            }
        }

        pub fn mul_div(
            a: BalanceOf<T>,
            b: frame_system::pallet_prelude::BlockNumberFor<T>,
            c: frame_system::pallet_prelude::BlockNumberFor<T>,
        ) -> BalanceOf<T> {
            let a128: u128 = a.saturated_into();
            let b128: u128 = b.saturated_into();
            let c128: u128 = c.saturated_into();
            a128.saturating_mul(b128)
                .saturating_div(c128)
                .saturated_into()
        }
    }
}
