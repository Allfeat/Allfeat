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

use crate::types::MiddsWrapper;
use allfeat_support::traits::Midds;
use alloc::boxed::Box;
use frame_support::{pallet_prelude::*, sp_runtime::Saturating, traits::fungible::MutateHold};
use frame_system::pallet_prelude::*;

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use allfeat_primitives::Moment;
	use allfeat_support::traits::Midds;
	#[cfg(feature = "runtime-benchmarks")]
	use frame_support::traits::fungible::Mutate;
	use frame_support::{
		sp_runtime::traits::Saturating,
		traits::{
			fungible::{Inspect, MutateHold},
			tokens::Precision,
			Time,
		},
		PalletId,
	};

	pub type MomentOf<T, I> = <<T as Config<I>>::Timestamp as Time>::Moment;
	pub type BalanceOf<T, I = ()> =
		<<T as Config<I>>::Currency as Inspect<<T as frame_system::Config>::AccountId>>::Balance;

	pub type MiddsHashIdOf<T> = <T as frame_system::Config>::Hash;
	pub type MiddsWrapperOf<T, I> = MiddsWrapper<
		<T as frame_system::Config>::AccountId,
		MomentOf<T, I>,
		<T as Config<I>>::MIDDS,
	>;

	/// The in-code storage version.
	const STORAGE_VERSION: StorageVersion = StorageVersion::new(1);

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
			type RuntimeEvent = ();
			#[inject_runtime_type]
			type RuntimeHoldReason = ();
			type ByteDepositCost = ConstU64<1>;
			type MaxDepositCost = ConstU64<10000>;
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

		#[pallet::no_default_bounds]
		/// The overarching event type.
		type RuntimeEvent: From<Event<Self, I>>
			+ IsType<<Self as frame_system::Config>::RuntimeEvent>;

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
		type MIDDS: Midds<Hash = <Self as frame_system::Config>::Hashing>
			+ Parameter
			+ Member
			+ MaxEncodedLen;

		#[pallet::no_default]
		/// The origin which may provide new MIDDS to register on-chain for this instance.
		type ProviderOrigin: EnsureOrigin<Self::RuntimeOrigin, Success = Self::AccountId>;

		#[pallet::constant]
		#[pallet::no_default_bounds]
		/// The per-byte deposit cost when depositing MIDDS on-chain.
		type ByteDepositCost: Get<BalanceOf<Self, I>>;

		#[pallet::constant]
		#[pallet::no_default_bounds]
		/// The maximum cost a user can lock in collateral for this MIDDS entity.
		/// This help to ensure we don't go higher than the max of the balance type, in such case
		/// the user would be able to don't pay any fees higher than this value.
		type MaxDepositCost: Get<BalanceOf<Self, I>>;

		/// How many time the depositor have to wait to remove the MIDDS.
		#[pallet::constant]
		#[pallet::no_default_bounds]
		type UnregisterPeriod: Get<Option<MomentOf<Self, I>>>;

		type WeightInfo: WeightInfo;
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

	#[pallet::storage]
	pub(super) type PendingMidds<T: Config<I>, I: 'static = ()> =
		StorageMap<_, Blake2_128Concat, MiddsHashIdOf<T>, MiddsWrapperOf<T, I>>;

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
		},
		MIDDSUnregistered {
			hash_id: MiddsHashIdOf<T>,
		},
	}

	#[pallet::error]
	pub enum Error<T, I = ()> {
		/// A MIDDS with the same hash ID (so the same data) is already registered.
		MiddsDataAlreadyExist,
		/// The specified MIDDS ID is not related to any pending MIDDS.
		PendingMiddsNotFound,
		/// Some data in the MIDDS aren't valid.
		UnvalidMiddsData,
		/// The lock-unregister period is still going.
		UnregisterLocked,
		/// The caller is not the provider of the MIDDS.
		NotProvider,
		/// Funds can't be released at this moment.
		CantReleaseFunds,
		/// Funds can't be held at this moment.
		CantHoldFunds,
		/// The provider tried to register/update a MIDDS that exceed data size cost maximum
		/// authorized.
		OverflowedAuthorizedDataCost,
	}

	#[pallet::call(weight(<T as Config<I>>::WeightInfo))]
	impl<T: Config<I>, I: 'static> Pallet<T, I> {
		#[pallet::call_index(0)]
		pub fn register(origin: OriginFor<T>, midds: Box<T::MIDDS>) -> DispatchResult {
			let provider = T::ProviderOrigin::ensure_origin(origin)?;
			let midds = *midds;
			ensure!(midds.is_valid(), Error::<T, I>::UnvalidMiddsData);
			let midds = MiddsWrapper::new(provider, T::Timestamp::now(), midds);

			Self::inner_register(midds)
		}

		#[pallet::call_index(1)]
		pub fn update_field(
			origin: OriginFor<T>,
			midds_id: MiddsHashIdOf<T>,
			field_data: <T::MIDDS as Midds>::EditableFields,
		) -> DispatchResult {
			let caller = T::ProviderOrigin::ensure_origin(origin)?;

			PendingMidds::<T, I>::try_mutate_exists(midds_id, |x| -> DispatchResult {
				match x {
					Some(midds) => {
						ensure!(midds.provider() == caller, Error::<T, I>::NotProvider);

						let old_hash = midds.midds.hash();
						let old_cost = Self::calculate_midds_colateral(midds);
						midds.midds.update_field(field_data)?;
						ensure!(midds.midds.is_valid(), Error::<T, I>::UnvalidMiddsData);
						let new_cost = Self::calculate_midds_colateral(midds);

						ensure!(
							new_cost <= T::MaxDepositCost::get(),
							Error::<T, I>::OverflowedAuthorizedDataCost
						);

						match old_cost.cmp(&new_cost) {
							core::cmp::Ordering::Greater => {
								T::Currency::release(
									&HoldReason::MiddsRegistration.into(),
									&caller,
									old_cost.saturating_sub(new_cost),
									Precision::BestEffort,
								)
								.map_err(|_| Error::<T, I>::CantReleaseFunds)?;
							},
							core::cmp::Ordering::Less => {
								T::Currency::hold(
									&HoldReason::MiddsRegistration.into(),
									&caller,
									new_cost.saturating_sub(old_cost),
								)
								.map_err(|_| Error::<T, I>::CantHoldFunds)?;
							},
							core::cmp::Ordering::Equal => {},
						};

						let new_hash = midds.midds.hash();

						PendingMidds::<T, I>::remove(old_hash);
						PendingMidds::<T, I>::insert(new_hash, midds);

						Self::deposit_event(Event::<T, I>::MIDDSUpdated { hash_id: new_hash });

						Ok(())
					},
					None => Err(Error::<T, I>::PendingMiddsNotFound.into()),
				}
			})?;

			Ok(())
		}

		#[pallet::call_index(2)]
		pub fn unregister(origin: OriginFor<T>, midds_id: MiddsHashIdOf<T>) -> DispatchResult {
			let caller = T::ProviderOrigin::ensure_origin(origin)?;

			if let Some(midds) = PendingMidds::<T, I>::get(midds_id) {
				ensure!(midds.provider() == caller, Error::<T, I>::NotProvider);

				if T::UnregisterPeriod::get().is_some() {
					let now = T::Timestamp::now();
					let spent = now - midds.registered_at();
					ensure!(
						spent >
							frame_support::sp_runtime::SaturatedConversion::saturated_into(
								T::UnregisterPeriod::get().unwrap()
							),
						Error::<T, I>::UnregisterLocked
					);
				}

				let actual_cost = Self::calculate_midds_colateral(&midds);
				T::Currency::release(
					&HoldReason::MiddsRegistration.into(),
					&caller,
					actual_cost,
					Precision::BestEffort,
				)
				.map_err(|_| Error::<T, I>::CantReleaseFunds)?;

				PendingMidds::<T, I>::remove(midds_id);

				Self::deposit_event(Event::<T, I>::MIDDSUnregistered { hash_id: midds_id });

				Ok(())
			} else {
				Err(Error::<T, I>::PendingMiddsNotFound.into())
			}
		}
	}
}

impl<T: Config<I>, I: 'static> Pallet<T, I> {
	fn inner_register(midds: MiddsWrapperOf<T, I>) -> DispatchResult {
		let midds_hash = midds.midds.hash();

		// Verify that the same MIDDS hash isn't registered already.
		ensure!(
			!PendingMidds::<T, I>::contains_key(midds_hash),
			Error::<T, I>::MiddsDataAlreadyExist
		);

		// data colateral lock
		let data_lock = Self::calculate_midds_colateral(&midds);
		ensure!(data_lock <= T::MaxDepositCost::get(), Error::<T, I>::OverflowedAuthorizedDataCost);

		T::Currency::hold(&HoldReason::MiddsRegistration.into(), &midds.provider(), data_lock)?;
		PendingMidds::<T, I>::insert(midds_hash, midds.clone());

		Self::deposit_event(Event::<T, I>::MIDDSRegistered {
			provider: midds.provider(),
			hash_id: midds_hash,
			data_colateral: data_lock,
		});

		Ok(())
	}

	fn calculate_midds_colateral(midds: &MiddsWrapperOf<T, I>) -> BalanceOf<T, I> {
		let bytes = midds.midds.total_bytes();
		T::ByteDepositCost::get().saturating_mul(BalanceOf::<T, I>::from(bytes))
	}
}
