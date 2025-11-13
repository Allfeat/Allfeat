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

mod mock;
mod types;
mod weights;
use allfeat_midds::MiddsId;
use types::{BalanceOf, MiddsInfo};
pub use weights::WeightInfo;

#[cfg(test)]
mod tests;

mod benchmarking;

extern crate alloc;

use alloc::boxed::Box;
use frame_support::{
    StorageHasher, pallet_prelude::*, sp_runtime::Saturating, traits::fungible::MutateHold,
};
use frame_system::pallet_prelude::*;
pub use pallet::*;

#[frame_support::pallet()]
pub mod pallet {
    use super::*;

    #[cfg(feature = "runtime-benchmarks")]
    use allfeat_midds::benchmarking::BenchmarkHelper;

    use allfeat_primitives::Moment;
    #[cfg(feature = "runtime-benchmarks")]
    use frame_support::traits::fungible::Mutate;
    use frame_support::{
        PalletId,
        traits::{Time, fungible::MutateHold, tokens::Precision},
    };
    use types::{BalanceOf, MiddsInfo, MomentOf};

    /// The in-code storage version.
    const STORAGE_VERSION: StorageVersion = StorageVersion::new(0);

    /// Default implementations of [`DefaultConfig`], which can be used to implement [`Config`].
    pub mod config_preludes {
        use super::*;
        use frame_support::{derive_impl, parameter_types, traits::ConstU64};

        pub struct TestDefaultConfig;

        parameter_types! {
            pub const UnregisterPeriod: Option<u64> = None;
        }

        #[derive_impl(frame_system::config_preludes::TestDefaultConfig, no_aggregated_types)]
        impl frame_system::DefaultConfig for TestDefaultConfig {}

        #[frame_support::register_default_impl(TestDefaultConfig)]
        impl DefaultConfig for TestDefaultConfig {
            #[inject_runtime_type]
            type RuntimeHoldReason = ();
            type ByteDepositCost = ConstU64<1>;
            type UnregisterPeriod = UnregisterPeriod;
            type WeightInfo = ();
        }
    }

    #[pallet::config(with_default)]
    pub trait Config<I: 'static = ()>: frame_system::Config {
        /// The MIDDS pallet instance pallet id
        #[pallet::no_default]
        #[pallet::constant]
        type PalletId: Get<PalletId>;

        #[pallet::no_default]
        #[cfg(not(feature = "runtime-benchmarks"))]
        /// The currency trait used to manage MIDDS payments.
        type Currency: MutateHold<Self::AccountId, Reason = Self::RuntimeHoldReason>;

        #[pallet::no_default]
        type Timestamp: Time<Moment = Moment>;

        #[pallet::no_default]
        #[cfg(feature = "runtime-benchmarks")]
        /// The way to handle the storage deposit cost of Artist creation
        /// Include Currency trait to have access to NegativeImbalance
        type Currency: Mutate<Self::AccountId>
            + MutateHold<Self::AccountId, Reason = Self::RuntimeHoldReason>;

        #[pallet::no_default_bounds]
        /// The overarching HoldReason type.
        type RuntimeHoldReason: From<HoldReason<I>>;

        #[pallet::no_default]
        /// The MIDDS actor that this pallet instance manage.
        type MIDDS: Parameter + Member + MaxEncodedLen;

        #[pallet::no_default]
        /// The origin which may provide new MIDDS to register on-chain for this instance.
        type ProviderOrigin: EnsureOrigin<Self::RuntimeOrigin, Success = Self::AccountId>;

        #[pallet::constant]
        #[pallet::no_default_bounds]
        /// The per-byte deposit cost when depositing MIDDS on-chain.
        type ByteDepositCost: Get<BalanceOf<Self, I>>;

        /// How many time the depositor have to wait to remove the MIDDS.
        #[pallet::constant]
        #[pallet::no_default_bounds]
        type UnregisterPeriod: Get<Option<MomentOf<Self, I>>>;

        type WeightInfo: WeightInfo;

        #[cfg(feature = "runtime-benchmarks")]
        #[pallet::no_default]
        type BenchmarkHelper: BenchmarkHelper<Self::MIDDS>;
    }

    /// A reason for the pallet MIDDS placing a hold on funds.
    #[pallet::composite_enum]
    pub enum HoldReason<I: 'static = ()> {
        /// A new MIDDS has been deposited and require colateral data value hold.
        MiddsRegistration,
    }

    #[pallet::pallet]
    #[pallet::storage_version(STORAGE_VERSION)]
    pub struct Pallet<T, I = ()>(PhantomData<(T, I)>);

    /// Storage of the next identifier to help identifying new MIDDS.
    #[pallet::storage]
    pub(super) type NextId<T: Config<I>, I: 'static = ()> = StorageValue<_, u64, ValueQuery>;

    #[pallet::storage]
    pub type MiddsOf<T: Config<I>, I: 'static = ()> =
        StorageMap<_, Blake2_128Concat, MiddsId, T::MIDDS>;

    #[pallet::storage]
    pub type MiddsInfoOf<T: Config<I>, I: 'static = ()> =
        StorageMap<_, Blake2_128Concat, MiddsId, MiddsInfo<T, I>>;

    /// Storage mapping Hashed MIDDS to the existing ID of that MIDDS for integrity and
    /// duplication check
    #[pallet::storage]
    pub type HashIndex<T: Config<I>, I: 'static = ()> =
        StorageMap<_, Blake2_128Concat, [u8; 32], MiddsId>;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config<I>, I: 'static = ()> {
        MIDDSRegistered {
            provider: T::AccountId,
            midds_id: MiddsId,
            data_colateral: BalanceOf<T, I>,
        },
        MIDDSUnregistered {
            midds_id: MiddsId,
        },
    }

    #[pallet::error]
    pub enum Error<T, I = ()> {
        /// A MIDDS with the same hash ID (so the same data) is already registered.
        MiddsDataAlreadyExist,
        /// The specified MIDDS ID is not related to any pending MIDDS.
        MiddsNotFound,
        UnvalidMiddsData,
        /// The lock-unregister period is still going.
        UnregisterLocked,
        /// The MIDDS can't be unregistered when pre-certifier/certified.
        UnregisterLockedNoVoting,
        /// The caller is not the provider of the MIDDS.
        NotProvider,
        /// Funds can't be released at this moment.
        CantReleaseFunds,
        /// Funds can't be held at this moment.
        CantHoldFunds,
    }

    #[pallet::call(weight(<T as Config<I>>::WeightInfo))]
    impl<T: Config<I>, I: 'static> Pallet<T, I> {
        #[pallet::call_index(0)]
        #[pallet::weight(T::WeightInfo::register(midds.encoded_size() as u32))]
        pub fn register(origin: OriginFor<T>, midds: Box<T::MIDDS>) -> DispatchResult {
            let provider = T::ProviderOrigin::ensure_origin(origin)?;
            let midds = *midds;

            let size = midds.encoded_size() as u32;
            let data_cost = Self::calculate_midds_colateral(size);

            let hash = Blake2_256::hash(&midds.encode());

            let info: MiddsInfo<T, I> = MiddsInfo {
                provider,
                registered_at: T::Timestamp::now(),
                hash,
                encoded_size: size,
                data_cost,
            };

            Self::inner_register(midds, info)
        }

        #[pallet::call_index(2)]
        #[pallet::weight(T::WeightInfo::unregister())]
        pub fn unregister(origin: OriginFor<T>, midds_id: MiddsId) -> DispatchResult {
            let caller = T::ProviderOrigin::ensure_origin(origin)?;

            if let Some(info) = MiddsInfoOf::<T, I>::get(midds_id) {
                ensure!(info.provider == caller, Error::<T, I>::NotProvider);

                if T::UnregisterPeriod::get().is_some() {
                    let now = T::Timestamp::now();
                    let spent = now - info.registered_at;
                    ensure!(
                        spent
                            > frame_support::sp_runtime::SaturatedConversion::saturated_into(
                                T::UnregisterPeriod::get().unwrap()
                            ),
                        Error::<T, I>::UnregisterLocked
                    );
                }

                T::Currency::release(
                    &HoldReason::MiddsRegistration.into(),
                    &caller,
                    info.data_cost,
                    Precision::BestEffort,
                )
                .map_err(|_| Error::<T, I>::CantReleaseFunds)?;

                MiddsOf::<T, I>::remove(midds_id);
                MiddsInfoOf::<T, I>::remove(midds_id);
                HashIndex::<T, I>::remove(info.hash);

                Self::deposit_event(Event::<T, I>::MIDDSUnregistered { midds_id });

                Ok(())
            } else {
                Err(Error::<T, I>::MiddsNotFound.into())
            }
        }
    }
}

impl<T: Config<I>, I: 'static> Pallet<T, I> {
    fn inner_register(midds: T::MIDDS, info: MiddsInfo<T, I>) -> DispatchResult {
        let midds_id = Self::get_next_id();

        // Verify that the same MIDDS isn't registered already by checking hash integrity.
        ensure!(
            !HashIndex::<T, I>::contains_key(info.hash),
            Error::<T, I>::MiddsDataAlreadyExist
        );

        T::Currency::hold(
            &HoldReason::MiddsRegistration.into(),
            &info.provider,
            info.data_cost,
        )?;

        MiddsOf::<T, I>::insert(midds_id, midds.clone());
        MiddsInfoOf::<T, I>::insert(midds_id, &info);
        HashIndex::<T, I>::insert(info.hash, midds_id);

        Self::increment_next_id();

        Self::deposit_event(Event::<T, I>::MIDDSRegistered {
            provider: info.provider,
            midds_id,
            data_colateral: info.data_cost,
        });

        Ok(())
    }

    fn get_next_id() -> MiddsId {
        NextId::<T, I>::get()
    }

    fn increment_next_id() {
        let current = Self::get_next_id();
        NextId::<T, I>::set(current.saturating_add(1))
    }

    fn calculate_midds_colateral(size: u32) -> BalanceOf<T, I> {
        T::ByteDepositCost::get().saturating_mul(BalanceOf::<T, I>::from(size))
    }
}
