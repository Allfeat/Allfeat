// This file is part of Allfeat.

// Copyright (C) 2022-2024 Allfeat.
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

#[cfg(test)]
mod tests;

mod benchmarking;

extern crate alloc;

use allfeat_support::traits::Midds;
use alloc::boxed::Box;
use frame_support::{
	dispatch::DispatchResult,
	ensure,
	sp_runtime::Saturating,
	traits::{fungible::MutateHold, Get},
};
pub use pallet::*;

#[frame_support::pallet(dev_mode)]
pub mod pallet {
	use core::fmt::Debug;

	use super::*;
	use allfeat_support::traits::Midds;
	use frame_support::{
		pallet_prelude::*,
		traits::{fungible::Inspect, tokens::Precision},
		PalletId,
	};
	use frame_system::pallet_prelude::*;

	pub type BalanceOf<T, I = ()> =
		<<T as Config<I>>::Currency as Inspect<<T as frame_system::Config>::AccountId>>::Balance;

	pub type MiddsHashIdOf<T> = <T as frame_system::Config>::Hash;

	/// The in-code storage version.
	const STORAGE_VERSION: frame_support::traits::StorageVersion =
		frame_support::traits::StorageVersion::new(1);

	/// Default implementations of [`DefaultConfig`], which can be used to implement [`Config`].
	pub mod config_preludes {
		use super::*;
		use frame_support::{
			derive_impl,
			traits::{ConstU32, ConstU64},
		};

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
			type UnregisterPeriod = ConstU32<7>;
		}
	}

	#[pallet::config(with_default)]
	pub trait Config<I: 'static = ()>: frame_system::Config {
		/// The MIDDS pallet instance pallet id
		#[pallet::no_default]
		#[pallet::constant]
		type PalletId: Get<PalletId>;

		#[pallet::no_default_bounds]
		/// The overarching event type.
		type RuntimeEvent: From<Event<Self, I>>
			+ IsType<<Self as frame_system::Config>::RuntimeEvent>;

		#[pallet::no_default]
		#[cfg(not(feature = "runtime-benchmarks"))]
		/// The currency trait used to manage MIDDS payments.
		type Currency: MutateHold<Self::AccountId, Reason = Self::RuntimeHoldReason>;

		#[pallet::no_default]
		#[cfg(feature = "runtime-benchmarks")]
		/// The way to handle the storage deposit cost of Artist creation
		/// Include Currency trait to have access to NegativeImbalance
		type Currency: frame_support::traits::fungible::Mutate<Self::AccountId>
			+ MutateHold<Self::AccountId, Reason = Self::RuntimeHoldReason>;

		#[pallet::no_default_bounds]
		/// The overarching HoldReason type.
		type RuntimeHoldReason: From<HoldReason>;

		#[pallet::no_default]
		/// The MIDDS actor that this pallet instance manage.
		type MIDDS: Midds<Self::Hashing, Self::AccountId, EditableFields = Self::MIDDSEditableFields>
			+ Parameter
			+ Member;

		#[pallet::no_default]
		type MIDDSEditableFields: Default + Encode + Decode + Clone + PartialEq + TypeInfo + Debug;

		#[pallet::no_default]
		/// The origin which may provide new MIDDS to register on-chain for this instance.
		type ProviderOrigin: EnsureOrigin<Self::RuntimeOrigin, Success = Self::AccountId>;

		#[pallet::constant]
		#[pallet::no_default_bounds]
		/// The per-byte deposit cost when depositing MIDDS on-chain.
		type ByteDepositCost: Get<BalanceOf<Self, I>>;

		/// How many time a the depositor have to wait to remove the MIDDS.
		#[pallet::constant]
		type UnregisterPeriod: Get<u32>;
	}

	/// A reason for the pallet MIDDS placing a hold on funds.
	#[pallet::composite_enum]
	pub enum HoldReason {
		/// A new MIDDS has been deposited and require colateral data value hold.
		MiddsRegistration,
	}

	#[pallet::pallet]
	#[pallet::storage_version(STORAGE_VERSION)]
	pub struct Pallet<T, I = ()>(PhantomData<(T, I)>);

	#[pallet::storage]
	pub(super) type PendingMidds<T: Config<I>, I: 'static = ()> =
		StorageMap<_, Blake2_128Concat, MiddsHashIdOf<T>, T::MIDDS>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config<I>, I: 'static = ()> {
		MIDDSRegistered {
			provider: T::AccountId,
			hash_id: MiddsHashIdOf<T>,
			data_colateral: BalanceOf<T, I>,
		},
		MIDDSUpdated {
			hash_id: MiddsHashIdOf<T>,
		}
	}

	#[pallet::error]
	pub enum Error<T, I = ()> {
		/// A MIDDS with the same hash ID (so the same data) is already registered.
		MiddsDataAlreadyExist,
		/// The specified MIDDS ID is not related to any pending MIDDS.
		PendingMiddsNotFound,
		/// The caller is not the provider of the MIDDS.
		NotProvider,
		/// Funds can't be released at this moment.
		CantReleaseFunds,
		/// Funds can't be held at this moment.
		CantHoldFunds,
	}

	#[pallet::call]
	impl<T: Config<I>, I: 'static> Pallet<T, I> {
		#[pallet::call_index(0)]
		pub fn register(origin: OriginFor<T>, mut midds: Box<T::MIDDS>) -> DispatchResult {
			let provider = T::ProviderOrigin::ensure_origin(origin)?;

			midds.set_provider(provider.clone());
			Self::inner_register(&provider, *midds)
		}

		#[pallet::call_index(1)]
		pub fn update_field(
			origin: OriginFor<T>,
			midds_id: MiddsHashIdOf<T>,
			field_data: T::MIDDSEditableFields,
		) -> DispatchResult {
			let caller = T::ProviderOrigin::ensure_origin(origin)?;

			PendingMidds::<T, I>::try_mutate_exists(midds_id, |x| match x {
				Some(midds) => {
					ensure!(midds.provider() == caller, Error::<T, I>::NotProvider);

					let old_cost = Self::calculate_midds_colateral(midds);
					midds.update_field(field_data);
					let new_cost = Self::calculate_midds_colateral(midds);

					if old_cost > new_cost {
						T::Currency::release(
							&HoldReason::MiddsRegistration.into(),
							&caller,
							old_cost.saturating_sub(new_cost),
							Precision::BestEffort,
						)
						.map_err(|_| Error::<T, I>::CantReleaseFunds)?;
					} else if old_cost < new_cost {
						T::Currency::hold(
							&HoldReason::MiddsRegistration.into(),
							&caller,
							new_cost.saturating_sub(old_cost),
						)
						.map_err(|_| Error::<T, I>::CantHoldFunds)?;
					}

					Self::deposit_event(Event::<T, I>::MIDDSUpdated {
						hash_id: midds.hash(),
					});

					Ok(())
				},
				None => Err(Error::<T, I>::PendingMiddsNotFound),
			})?;

			Ok(())
		}
	}
}

impl<T: Config<I>, I: 'static> Pallet<T, I> {
	fn inner_register(provider: &T::AccountId, midds: T::MIDDS) -> DispatchResult {
		let midds_hash = midds.hash();

		// Verify that the same MIDDS hash isn't registered already.
		ensure!(
			!PendingMidds::<T, I>::contains_key(midds_hash),
			Error::<T, I>::MiddsDataAlreadyExist
		);

		// data colateral lock
		let data_lock = Self::calculate_midds_colateral(&midds);

		T::Currency::hold(&HoldReason::MiddsRegistration.into(), provider, data_lock)?;
		PendingMidds::<T, I>::insert(midds_hash, midds);

		Self::deposit_event(Event::<T, I>::MIDDSRegistered {
			provider: provider.clone(),
			hash_id: midds_hash,
			data_colateral: data_lock,
		});

		Ok(())
	}

	fn calculate_midds_colateral(midds: &T::MIDDS) -> BalanceOf<T, I> {
		let bytes = midds.total_bytes();
		T::ByteDepositCost::get().saturating_mul(BalanceOf::<T, I>::from(bytes))
	}
}