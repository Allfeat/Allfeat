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
pub use weights::WeightInfo;

#[cfg(test)]
mod tests;

mod benchmarking;

extern crate alloc;

use alloc::boxed::Box;
use frame_support::pallet_prelude::*;
use frame_system::pallet_prelude::*;
pub use pallet::*;
use sp_core::U256;

#[frame_support::pallet()]
pub mod pallet {
    use super::*;
    use allfeat_primitives::Moment;
    #[cfg(feature = "runtime-benchmarks")]
    use frame_support::traits::fungible::Mutate;
    use frame_support::{
        PalletId,
        traits::{Time, fungible::MutateHold},
    };
    use types::BalanceOf;

    pub type AtsId = u64;

    /// The in-code storage version.
    const STORAGE_VERSION: StorageVersion = StorageVersion::new(1);

    /// Default implementations of [`DefaultConfig`], which can be used to implement [`Config`].
    pub mod config_preludes {
        use super::*;
        use frame_support::{derive_impl, traits::ConstU64};

        pub struct TestDefaultConfig;

        #[derive_impl(frame_system::config_preludes::TestDefaultConfig, no_aggregated_types)]
        impl frame_system::DefaultConfig for TestDefaultConfig {}

        #[frame_support::register_default_impl(TestDefaultConfig)]
        impl DefaultConfig for TestDefaultConfig {
            #[inject_runtime_type]
            type RuntimeEvent = ();
            #[inject_runtime_type]
            type RuntimeHoldReason = ();
            type ByteDepositCost = ConstU64<1>;
            type WeightInfo = ();
        }
    }

    #[pallet::config(with_default)]
    pub trait Config: frame_system::Config {
        /// The ATS pallet instance pallet id
        #[pallet::no_default]
        #[pallet::constant]
        type PalletId: Get<PalletId>;

        #[pallet::no_default_bounds]
        /// The overarching event type.
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        #[pallet::no_default]
        #[cfg(not(feature = "runtime-benchmarks"))]
        /// The currency trait used to manage ATS payments.
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
        type RuntimeHoldReason: From<HoldReason>;

        #[pallet::no_default]
        /// The origin which may provide new ATS to register on-chain for this instance.
        type ProviderOrigin: EnsureOrigin<Self::RuntimeOrigin, Success = Self::AccountId>;

        #[pallet::constant]
        #[pallet::no_default_bounds]
        /// The per-byte deposit cost when depositing ATS on-chain.
        type ByteDepositCost: Get<BalanceOf<Self>>;

        type WeightInfo: WeightInfo;
    }

    /// A reason for the pallet ATS placing a hold on funds.
    #[pallet::composite_enum]
    pub enum HoldReason {
        /// A new ATS has been deposited and require colateral data value hold.
        AtsRegistration,
    }

    #[pallet::pallet]
    #[pallet::storage_version(STORAGE_VERSION)]
    pub struct Pallet<T>(PhantomData<T>);

    #[derive(
        Encode, Decode, MaxEncodedLen, TypeInfo, Clone, PartialEq, Eq, Debug, DecodeWithMemTracking,
    )]
    pub struct AtsData {
        pub a: U256,
        pub b: U256,
    }

    /// Storage of the next identifier to help identifying new ATS.
    #[pallet::storage]
    pub(super) type NextId<T: Config> = StorageValue<_, u64, ValueQuery>;

    #[pallet::storage]
    pub type AtsOf<T: Config> = StorageMap<_, Blake2_128Concat, AtsId, AtsData>;

    /// Storage mapping Hashed ATS to the existing ID of that ATS for integrity and
    /// duplication check
    #[pallet::storage]
    pub type HashIndex<T: Config> = StorageMap<_, Blake2_128Concat, [u8; 32], AtsId>;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        ATSRegistered {
            provider: T::AccountId,
            ats_id: AtsId,
            data_colateral: BalanceOf<T>,
        },
    }

    #[pallet::error]
    pub enum Error<T> {
        /// A ATS with the same hash ID (so the same data) is already registered.
        AtsDataAlreadyExist,
        /// The specified ATS ID is not related to any pending ATS.
        AtsNotFound,
        UnvalidAtsData,
        /// The caller is not the provider of the ATS.
        NotProvider,
        /// Funds can't be held at this moment.
        CantHoldFunds,
    }

    #[pallet::call(weight(<T as Config>::WeightInfo))]
    impl<T: Config> Pallet<T> {
        #[pallet::call_index(0)]
        #[pallet::weight(T::WeightInfo::register(ats_data.encoded_size() as u32))]
        pub fn register(origin: OriginFor<T>, ats_data: Box<AtsData>) -> DispatchResult {
            let _sender = T::ProviderOrigin::ensure_origin(origin)?;

            Ok(())
        }

        #[pallet::call_index(2)]
        #[pallet::weight(T::WeightInfo::unregister())]
        pub fn unregister(origin: OriginFor<T>, ats_id: AtsId) -> DispatchResult {
            let _sender = T::ProviderOrigin::ensure_origin(origin)?;

            Ok(())
        }
    }
}
