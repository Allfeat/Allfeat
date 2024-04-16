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

//! # Artists Pallet v2
//!
//! If you're diving into the "Artists Pallet v2," here's a quick guide to help you
//! navigate and understand its core components and functionalities.
//!
//! ### Overview
//!
//! The "Artists Pallet v2" is a pallet implementation designed for the management of artists on
//! Allfeat blockchain. This module enables users to register as artists, associate details to their
//! profiles, and handle this information on-chain.
//!
//! ### Key Features
//!
//! 1. **Artist Registration**: Users can register themselves as artists, providing details like
//!    their main
//! name, an alias, music genres, a description, and related assets.
//!
//! 2. **Storage**: Artist data is securely stored on-chain. Artists can be retrieved by their
//!    account
//! ID.
//!
//! 3. **Asset Handling**: Artist assets undergo hashing to ensure data integrity.
//!
//! 4. **Error Management**: Several error cases are covered, like when an artist tries to
//!    unregister while verified.
//!
//! ### Configuration (`Config`)
//!
//! This pallet offers multiple configurable constants:
//! - `BaseDeposit`: The base deposit for registering as an artist.
//! - `ByteDeposit`: The per-byte deposit for hashing data on-chain.
//! - `UnregisterPeriod`: The time a registered artist must wait before being allowed to unregister.
//! - `MaxNameLen`: Maximum allowable length for an artist's name.
//! - `MaxGenres`: Maximum number of genres an artist can associate with.
//! - `MaxAssets`: Maximum assets an artist can have.
//! - `MaxContracts`: Maximum contracts an artist can have.
//!
//! ### Events
//!
//! - `ArtistRegistered`: Triggered when a new artist gets registered. Carries the artist's account
//!   ID and name.
//!
//! ### Errors
//!
//! A few of the potential errors include:
//! - `NotUniqueGenre`: Raised when a genre appears multiple times in an artist's data.
//! - `NameUnavailable`: Raised if the artist's name is already taken by a verified artist.
//! - `NotRegistered`: If an account isn't registered as an artist.
//! - `AlreadyRegistered`: If the account ID is already registered as an artist.
//! - `IsVerified`: If the artist is verified and therefore cannot unregister.
//! - `PeriodNotPassed`: If the unregister period isn't fully elapsed yet.
//!
//! ### Extrinsics
//!
//! - `register`: Allows a user to register as an artist by mapping the Account ID.
//!
//! ### Wrapping Up
//!
//! As you navigate through "Artists Pallet v2," you'll find it's a robust module for on-chain
//! artist profile management. If you have questions, the comments in the code should guide you, but
//! this overview should give you a head start

#![allow(missing_docs)]
#![cfg_attr(not(feature = "std"), no_std)]

mod benchmarking;
mod macros;
pub mod migration;
#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;
pub mod types;
pub mod weights;

use weights::WeightInfo;

use frame_support::{
	pallet_prelude::{DispatchResultWithPostInfo, Get, Weight},
	BoundedVec,
};
use genres_registry::MusicGenre;
pub use types::Artist;

use crate::{
	types::{AccountIdOf, BalanceOf, UpdatableAssets, UpdatableData, UpdatableGenres},
	Event::{ArtistForceUnregistered, ArtistRegistered, ArtistUnregistered, ArtistUpdated},
};
use frame_support::{
	traits::{
		fungible::{BalancedHold, Credit, Inspect, MutateHold},
		tokens::{fungible::hold::Inspect as InspectHold, Precision},
		Imbalance, OnUnbalanced,
	},
	PalletId,
};
use sp_runtime::{traits::Zero, SaturatedConversion};

use frame_system::EnsureSignedBy;
use sp_runtime::traits::AccountIdConversion;

use sp_std::prelude::*;

pub use pallet::*;

/// Artists Pallet
#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use crate::types::{ArtistType, ExtraArtistTypes};
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;

	/// The current storage version, we set to 1 our new version.
	const STORAGE_VERSION: StorageVersion = StorageVersion::new(1);

	#[pallet::pallet]
	#[pallet::storage_version(STORAGE_VERSION)]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// The Artists pallet id, used for deriving its sovereign account ID.
		#[pallet::constant]
		type PalletId: Get<PalletId>;

		/// The overarching event type.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		#[cfg(not(feature = "runtime-benchmarks"))]
		/// The way to handle the storage deposit cost of Artist creation
		type Currency: Inspect<Self::AccountId>
			+ MutateHold<Self::AccountId, Reason = Self::RuntimeHoldReason>
			+ BalancedHold<Self::AccountId>;

		#[cfg(feature = "runtime-benchmarks")]
		/// The way to handle the storage deposit cost of Artist creation
		/// Include Currency trait to have access to NegativeImbalance
		type Currency: frame_support::traits::fungible::Mutate<Self::AccountId>
			+ Inspect<Self::AccountId>
			+ MutateHold<Self::AccountId, Reason = Self::RuntimeHoldReason>
			+ BalancedHold<Self::AccountId>;

		/// The base deposit for registering as an artist on chain.
		type BaseDeposit: Get<BalanceOf<Self>>;

		/// The per-byte deposit for placing data hashes on chain.
		type ByteDeposit: Get<BalanceOf<Self>>;

		/// The overarching hold reason.
		type RuntimeHoldReason: From<HoldReason>;

		/// The Root Origin that allow force unregistering artists.
		type RootOrigin: EnsureOrigin<Self::RuntimeOrigin>;

		/// Handler for the unbalanced reduction when slashing an artists deposit.
		type Slash: OnUnbalanced<Credit<Self::AccountId, Self::Currency>>;

		/// How many time a registered artist have to wait to unregister himself.
		#[pallet::constant]
		type UnregisterPeriod: Get<u32>;

		/// The maximum length of the artist name.
		#[pallet::constant]
		type MaxNameLen: Get<u32>;

		/// The maximum amount of genres that an artist can have.
		#[pallet::constant]
		type MaxGenres: Get<u32>;

		/// The maximum amount of assets that an artist can have.
		#[pallet::constant]
		type MaxAssets: Get<u32>;

		/// Weight information for extrinsics in this pallet.
		type WeightInfo: WeightInfo;
	}

	/// A reason for the pallet contracts placing a hold on funds.
	#[pallet::composite_enum]
	pub enum HoldReason {
		/// The Pallet has reserved it for registering the base Artist data.
		ArtistRegistration,
		/// The Pallet has reserved it for storage assets deposit.
		ArtistAssets,
		/// The Pallet has reserved it for storage description  deposit.
		ArtistDescription,
		/// The Pallet has reserved it for storage main name deposit.
		ArtistName,
		/// The Pallet has reserved it for storage alias deposit.
		ArtistAlias,
	}

	#[pallet::type_value]
	pub fn DefaultAddress<T: Config>() -> T::AccountId {
		let id: T::AccountId = T::PalletId::get().into_account_truncating();
		Address::<T>::set(id.clone());
		id
	}

	#[pallet::storage]
	#[pallet::getter(fn get_artist_by_id)]
	pub(super) type ArtistOf<T: Config> = StorageMap<_, Twox64Concat, T::AccountId, Artist<T>>;

	/// Used to cache the account id of this pallet
	#[pallet::storage]
	pub type Address<T: Config> = StorageValue<_, T::AccountId, ValueQuery, DefaultAddress<T>>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// A new artist got registered.
		ArtistRegistered {
			/// The address of the new artist.
			id: T::AccountId,
			/// main name of the new artist.
			name: BoundedVec<u8, T::MaxNameLen>,
		},

		/// An Artist as been unregistered
		ArtistUnregistered { id: T::AccountId },

		/// An Artist as been unregistered from the `T::RootOrigin`
		ArtistForceUnregistered { id: T::AccountId },

		ArtistUpdated {
			/// The address of the updated artist.
			id: T::AccountId,
			/// The new data.
			new_data: UpdatableData,
		},
	}

	#[pallet::error]
	pub enum Error<T> {
		/// A genre appear multiple time in the artist data.
		NotUniqueGenre,
		/// An asset appear multiple time in the artist data.
		NotUniqueAsset,
		/// The artist name is already attributed to a verified artist.
		NameUnavailable,
		/// Account isn't registered as an Artist.
		NotRegistered,
		/// This account ID is already registered as an artist.
		AlreadyRegistered,
		/// Artist is verified and can't unregister.
		IsVerified,
		/// Unregister period isn't fully passed.
		PeriodNotPassed,
		/// The maximum value possible for this field for an artist has been violated.
		Full,
		/// Element wasn't found.
		NotFound,
		/// The extra type trying to be set is already the main type.
		IsMainType,
		/// The main type trying to be set is already an extra type.
		IsExtraType,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Register the caller as an Artist.
		#[pallet::weight(T::WeightInfo::register(
			T::MaxNameLen::get(),
			T::MaxGenres::get(),
			T::MaxAssets::get()
		))]
		#[pallet::call_index(0)]
		pub fn register(
			origin: OriginFor<T>,
			main_name: BoundedVec<u8, T::MaxNameLen>,
			main_type: ArtistType,
			extra_artist_types: ExtraArtistTypes,
			genres: BoundedVec<MusicGenre, T::MaxGenres>,
			description: Option<Vec<u8>>,
			assets: BoundedVec<Vec<u8>, T::MaxAssets>,
		) -> DispatchResultWithPostInfo {
			let origin = ensure_signed(origin)?;

			ensure!(!ArtistOf::<T>::contains_key(origin.clone()), Error::<T>::AlreadyRegistered);

			let new_artist = Artist::<T>::new(
				origin.clone(),
				main_name.clone(),
				genres,
				main_type.into(),
				extra_artist_types,
				description,
				assets,
			)?;

			// held amount for base artist data registration
			T::Currency::hold(
				&HoldReason::ArtistRegistration.into(),
				&origin,
				T::BaseDeposit::get(),
			)?;

			ArtistOf::insert(origin.clone(), new_artist);

			Self::deposit_event(ArtistRegistered { id: origin, name: main_name });
			Ok(().into())
		}

		/// Unregister the caller from being an artist,
		/// clearing associated artist data mapped to this account.
		///
		/// Enforced by `T::RootOrigin`, ignoring `T::UnregisterPeriod` and slash held balance of
		/// the artist.
		#[pallet::weight(T::WeightInfo::force_unregister(
			T::MaxNameLen::get(),
			T::MaxGenres::get(),
			T::MaxAssets::get()
		))]
		#[pallet::call_index(1)]
		pub fn force_unregister(
			origin: OriginFor<T>,
			id: T::AccountId,
		) -> DispatchResultWithPostInfo {
			T::RootOrigin::ensure_origin(origin)?;

			Self::slash_held_all(&id)?;

			ArtistOf::<T>::remove(id.clone());

			Self::deposit_event(ArtistForceUnregistered { id });
			Ok(().into())
		}

		/// Unregister the caller from being an artist,
		/// clearing associated artist data mapped to this account
		#[pallet::weight(T::WeightInfo::unregister(
			T::MaxNameLen::get(),
			T::MaxGenres::get(),
			T::MaxAssets::get()
		))]
		#[pallet::call_index(2)]
		pub fn unregister(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
			let origin = ensure_signed(origin)?;

			Self::can_unregister(&origin)?;

			Self::release_held_all(&origin)?;

			ArtistOf::<T>::remove(origin.clone());

			Self::deposit_event(ArtistUnregistered { id: origin });
			Ok(().into())
		}

		/// Update the passed caller artist data field with the passed data.
		#[pallet::weight({
            let weight_fn = Pallet::<T>::get_weight_update_fn(&data);
            weight_fn()
        })]
		#[pallet::call_index(3)]
		pub fn update(origin: OriginFor<T>, data: UpdatableData) -> DispatchResultWithPostInfo {
			let origin = ensure_signed(origin)?;

			ArtistOf::<T>::try_mutate(origin.clone(), |maybe_artist| {
				if let Some(artist) = maybe_artist {
					artist.update(data.clone())?;
					Self::deposit_event(ArtistUpdated { id: origin, new_data: data });
					Ok(().into())
				} else {
					return Err(Error::<T>::NotRegistered.into());
				}
			})
		}
	}
}

impl<T> Pallet<T>
where
	T: frame_system::Config + Config,
{
	/// Release the held deposit for all reasons handled by this pallet.
	fn release_held_all(account_id: &T::AccountId) -> DispatchResultWithPostInfo {
		// return all held deposits
		T::Currency::release(
			&HoldReason::ArtistRegistration.into(),
			&account_id,
			T::BaseDeposit::get(),
			Precision::BestEffort,
		)?;
		T::Currency::release(
			&HoldReason::ArtistAssets.into(),
			&account_id,
			T::Currency::balance_on_hold(&HoldReason::ArtistAssets.into(), &account_id),
			Precision::BestEffort,
		)?;
		T::Currency::release(
			&HoldReason::ArtistAlias.into(),
			&account_id,
			T::Currency::balance_on_hold(&HoldReason::ArtistAlias.into(), &account_id),
			Precision::BestEffort,
		)?;
		T::Currency::release(
			&HoldReason::ArtistDescription.into(),
			&account_id,
			T::Currency::balance_on_hold(&HoldReason::ArtistDescription.into(), &account_id),
			Precision::BestEffort,
		)?;
		T::Currency::release(
			&HoldReason::ArtistName.into(),
			&account_id,
			T::Currency::balance_on_hold(&HoldReason::ArtistName.into(), &account_id),
			Precision::BestEffort,
		)?;
		Ok(().into())
	}

	/// Slash the held deposit for all reasons handled by this pallet.
	fn slash_held_all(account_id: &T::AccountId) -> DispatchResultWithPostInfo {
		// slash and handle slash for all held deposits
		let imbalance = <<T as pallet::Config>::Currency as BalancedHold<AccountIdOf<T>>>::slash(
			&HoldReason::ArtistRegistration.into(),
			&account_id,
			T::BaseDeposit::get(),
		)
		.0
		.merge(
			<<T as pallet::Config>::Currency as BalancedHold<AccountIdOf<T>>>::slash(
				&HoldReason::ArtistAssets.into(),
				&account_id,
				T::Currency::balance_on_hold(&HoldReason::ArtistAssets.into(), &account_id),
			)
			.0,
		)
		.merge(
			<<T as pallet::Config>::Currency as BalancedHold<AccountIdOf<T>>>::slash(
				&HoldReason::ArtistAssets.into(),
				&account_id,
				T::Currency::balance_on_hold(&HoldReason::ArtistAssets.into(), &account_id),
			)
			.0,
		)
		.merge(
			<<T as pallet::Config>::Currency as BalancedHold<AccountIdOf<T>>>::slash(
				&HoldReason::ArtistAlias.into(),
				&account_id,
				T::Currency::balance_on_hold(&HoldReason::ArtistAlias.into(), &account_id),
			)
			.0,
		)
		.merge(
			<<T as pallet::Config>::Currency as BalancedHold<AccountIdOf<T>>>::slash(
				&HoldReason::ArtistDescription.into(),
				&account_id,
				T::Currency::balance_on_hold(&HoldReason::ArtistDescription.into(), &account_id),
			)
			.0,
		)
		.merge(
			<<T as pallet::Config>::Currency as BalancedHold<AccountIdOf<T>>>::slash(
				&HoldReason::ArtistName.into(),
				&account_id,
				T::Currency::balance_on_hold(&HoldReason::ArtistName.into(), &account_id),
			)
			.0,
		);

		if !imbalance.peek().is_zero() {
			T::Slash::on_unbalanced(imbalance);
		}

		Ok(().into())
	}

	/// Returns a closure that computes the weight of an update operation based on the provided
	/// data.
	///
	/// This function is part of Substrate's weight and benchmarking system for blockchain
	/// operations. It determines the computational and storage resources required for different
	/// update operations.
	///
	/// # Arguments
	///
	/// * `data` - A reference to `UpdatableData<ArtistAliasOf<T>>`, an enum representing the type
	///   of data to be updated. The generic `T` is typically a type associated with a specific
	///   blockchain implementation.
	///
	/// # Returns
	///
	/// A `Box<dyn FnOnce() -> Weight>` which is a boxed closure that can be called once to compute
	/// the weight of the specified update operation. `Weight` is a metric used to measure the
	/// resource consumption of the operation on the blockchain.
	///
	/// # Implementation Details
	///
	/// - The function uses a `match` expression to determine the type of the update operation from
	///   `UpdatableData`.
	/// - For `Genres` and `Assets`, a sub-match on `UpdatableDataVec` discriminates whether items
	///   are being added, removed, or if the list is cleared.
	/// - Each branch calls an appropriate method from the `WeightInfo` trait, which must be
	///   implemented by `T`. These methods provide weight estimations for different operations,
	///   such as `T::WeightInfo::update_add_genres(T::MaxGenres::get())` for adding genres.
	/// - Closures are used to encapsulate the specific logic for each update operation, ensuring
	///   the returned function conforms to `FnOnce() -> Weight`.
	///
	/// This approach allows dynamic determination of operation costs on the blockchain, adapting to
	/// the current context and specific parameters of each update operation.
	fn get_weight_update_fn(data: &UpdatableData) -> Box<dyn FnOnce() -> Weight> {
		match data {
			UpdatableData::MainType(_) => Box::new(move || T::WeightInfo::update_main_type()),
			UpdatableData::ExtraTypes(_) => Box::new(move || T::WeightInfo::update_extra_types()),
			UpdatableData::Genres(x) => match x {
				UpdatableGenres::Add(_) =>
					Box::new(move || T::WeightInfo::update_add_genres(T::MaxGenres::get())),
				UpdatableGenres::Remove(_) =>
					Box::new(move || T::WeightInfo::update_remove_genres(T::MaxGenres::get())),
				UpdatableGenres::Clear =>
					Box::new(move || T::WeightInfo::update_clear_genres(T::MaxGenres::get())),
			},
			UpdatableData::Assets(x) => match x {
				UpdatableAssets::Add(_) =>
					Box::new(move || T::WeightInfo::update_add_assets(T::MaxAssets::get())),
				UpdatableAssets::Remove(_) =>
					Box::new(move || T::WeightInfo::update_remove_assets(T::MaxAssets::get())),
				UpdatableAssets::Clear =>
					Box::new(move || T::WeightInfo::update_clear_assets(T::MaxAssets::get())),
			},
			UpdatableData::Description(_) => Box::new(move || T::WeightInfo::update_description()),
		}
	}

	/// Return if the actual account ID can unregister from being an Artist.
	fn can_unregister(who: &T::AccountId) -> DispatchResultWithPostInfo {
		let artist_data = Pallet::<T>::get_artist_by_id(&who);

		match artist_data {
			Some(data) => {
				let current_block = <frame_system::Pallet<T>>::block_number();
				let expected_passed_time: u32 = T::UnregisterPeriod::get();

				// Verify that we passed the Unregister Period
				if current_block - data.registered_at < expected_passed_time.saturated_into() {
					return Err(Error::<T>::PeriodNotPassed.into());
				}

				Ok(().into())
			},
			None => Err(Error::<T>::NotRegistered.into()),
		}
	}
}

pub type EnsureArtistsPallet<T> =
	EnsureSignedBy<Address<T>, <T as frame_system::Config>::AccountId>;
