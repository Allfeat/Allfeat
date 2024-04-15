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

use super::*;
use derive_getters::Getters;
use frame_support::{
	dispatch::DispatchErrorWithPostInfo, traits::tokens::fungible::hold::Inspect as InspectHold,
};
use frame_system::pallet_prelude::BlockNumberFor;
use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use sp_runtime::{traits::Hash, RuntimeDebug, Saturating};
use sp_std::collections::btree_set::BTreeSet;

pub(super) type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
pub(super) type BalanceOf<T> =
	<<T as Config>::Currency as Inspect<<T as frame_system::Config>::AccountId>>::Balance;
pub(super) type ArtistAliasOf<T> = BoundedVec<u8, <T as Config>::MaxNameLen>;

#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo)]
pub enum UpdatableData<ArtistAlias> {
	Alias(Option<ArtistAlias>),
	Genres(UpdatableGenres),
	Description(Option<Vec<u8>>),
	Assets(UpdatableAssets),
}

#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo)]
pub enum UpdatableAssets {
	Add(Vec<u8>),
	/// lookup into the existing value if the content exist and try to remove it
	Remove(Vec<u8>),
	Clear,
}

#[derive(Encode, MaxEncodedLen, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo)]
pub enum UpdatableGenres {
	Add(MusicGenre),
	/// lookup into the existing value if the content exist and try to remove it
	Remove(MusicGenre),
	Clear,
}

/// How an Artist is designed to be stored on-chain.
#[derive(Encode, MaxEncodedLen, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, Getters)]
#[scale_info(skip_type_params(T))]
pub struct Artist<T>
where
	T: frame_system::Config + Config,
{
	// Main data
	/// The artist's identifier. While the predominant mapping employs AccountId => Artist,
	/// it's essential to include this in the artist's data since verified artists can be retrieved
	/// by their name as well.
	pub(crate) owner: AccountIdOf<T>,
	/// When the artist got registered on-chain.
	pub(crate) registered_at: BlockNumberFor<T>,
	/// When the artist got verified.
	verified_at: Option<BlockNumberFor<T>>,
	// Metadata
	/// The name of the artist.
	/// This is generally the main name of how we usually call the artist (e.g: 'The Weeknd')
	/// This is fixed and can't be changed after the registration.
	pub(crate) main_name: BoundedVec<u8, T::MaxNameLen>,
	/// An alias to the main name.
	/// This name can be changed compared to the 'nickname'
	pub(crate) alias: Option<ArtistAliasOf<T>>,
	/// The main music genres of the artists.
	genres: BoundedVec<MusicGenre, T::MaxGenres>,
	// Metadata Fingerprint
	// Given the significant size of certain data associated with an artist,
	// we choose to store a digital fingerprint (hash) of this data rather than
	// the raw data itself. This fingerprint acts as a unique digital reference,
	// and services can use it to compare and validate the artist's data, ensuring
	// that it has been approved and recorded on the blockchain by the artist themselves.
	/// The digital fingerprint (hash) of the artist's description.
	pub(crate) description: Option<T::Hash>,
	/// Digital assets (such as photos, profile pictures, banners, videos, etc.)
	/// that officially represent the artist. These fingerprints allow for the
	/// verification of the authenticity of these assets.
	assets: BoundedVec<T::Hash, T::MaxAssets>,
	// Linked chain logic data
	/// Associated smart-contracts deployed by dApps for the artist (e.g: royalties contracts)
	contracts: BoundedVec<AccountIdOf<T>, T::MaxContracts>,
}

impl<T> Artist<T>
where
	T: frame_system::Config + Config,
{
	pub(super) fn new(
		owner: AccountIdOf<T>,
		main_name: BoundedVec<u8, T::MaxNameLen>,
		alias: Option<ArtistAliasOf<T>>,
		genres: BoundedVec<MusicGenre, T::MaxGenres>,
		description: Option<Vec<u8>>,
		assets: BoundedVec<Vec<u8>, T::MaxAssets>,
	) -> Result<Self, DispatchErrorWithPostInfo> {
		let current_block = <frame_system::Pallet<T>>::block_number();

		let mut new_artist = Artist {
			owner,
			registered_at: current_block,
			verified_at: None,
			main_name: main_name.clone(),
			alias: Default::default(),
			// need to set later with the checked fn
			genres: Default::default(),
			description: Default::default(),
			assets: Default::default(),
			contracts: Default::default(),
		};

		let name_len: BalanceOf<T> = main_name.encoded_size().saturated_into();
		T::Currency::hold(
			&HoldReason::ArtistName.into(),
			&new_artist.owner,
			T::ByteDeposit::get().saturating_mul(name_len),
		)?;

		new_artist.set_alias(alias)?;
		new_artist.set_checked_genres(genres)?;
		new_artist.set_description(description)?;
		assets
			.iter()
			.try_for_each(|asset| new_artist.add_checked_asset(asset).map(|_| ()))?;

		Ok(new_artist)
	}

	/// Set the genres of the artist while verifying that there is not the same genre multiple
	/// times.
	pub(super) fn set_checked_genres(
		&mut self,
		genres: BoundedVec<MusicGenre, T::MaxGenres>,
	) -> DispatchResultWithPostInfo {
		let mut seen = BTreeSet::new();

		for genre in genres.clone() {
			if !seen.insert(genre.clone()) {
				return Err(Error::<T>::NotUniqueGenre.into());
			}
		}

		self.genres = genres;

		Ok(().into())
	}

	fn add_checked_genres(&mut self, genre: MusicGenre) -> DispatchResultWithPostInfo {
		let mut actual_genres = self.genres.clone();
		actual_genres.try_push(genre).map_err(|_| Error::<T>::Full)?;

		self.set_checked_genres(actual_genres)
	}

	pub(super) fn update(
		&mut self,
		field: UpdatableData<BoundedVec<u8, T::MaxNameLen>>,
	) -> DispatchResultWithPostInfo {
		match field {
			UpdatableData::Alias(x) => self.set_alias(x)?,
			UpdatableData::Genres(UpdatableGenres::Add(x)) => return self.add_checked_genres(x),
			UpdatableData::Genres(UpdatableGenres::Remove(x)) => return self.remove_genre(x),
			UpdatableData::Genres(UpdatableGenres::Clear) => self.genres = Default::default(),
			UpdatableData::Description(x) => self.set_description(x)?,
			UpdatableData::Assets(UpdatableAssets::Add(x)) => return self.add_checked_asset(&x),
			UpdatableData::Assets(UpdatableAssets::Remove(x)) => return self.remove_asset(&x),
			UpdatableData::Assets(UpdatableAssets::Clear) => self.clear_assets()?,
		}

		Ok(().into())
	}
	/// Return true if the artist have a 'verified_at" timestamp which mean he's verified
	pub(super) fn is_verified(&self) -> bool {
		self.verified_at.is_some()
	}

	fn set_alias(
		&mut self,
		alias: Option<BoundedVec<u8, T::MaxNameLen>>,
	) -> Result<(), DispatchErrorWithPostInfo> {
		let alias_len = alias.encoded_size();
		let alias_cost = T::ByteDeposit::get().saturating_mul(alias_len.saturated_into());

		let old_deposit =
			T::Currency::balance_on_hold(&HoldReason::ArtistAlias.into(), &self.owner);

		if alias_cost > old_deposit {
			T::Currency::hold(
				&HoldReason::ArtistAlias.into(),
				&self.owner,
				alias_cost - old_deposit,
			)?;
		}
		if alias_cost < old_deposit {
			T::Currency::release(
				&HoldReason::ArtistAlias.into(),
				&self.owner,
				old_deposit - alias_cost,
				Precision::Exact,
			)?;
		}

		self.alias = alias;

		Ok(())
	}

	fn set_description(
		&mut self,
		raw_description: Option<Vec<u8>>,
	) -> Result<(), DispatchErrorWithPostInfo> {
		// Clean any existent deposit
		self.unreserve_deposit_hash(HoldReason::ArtistDescription)?;

		match raw_description {
			Some(x) => {
				self.reserve_deposit_hash(HoldReason::ArtistDescription)?;
				self.description = Some(T::Hashing::hash(&x));
			},
			None => self.description = None,
		}

		Ok(())
	}

	fn add_checked_asset(&mut self, asset: &Vec<u8>) -> DispatchResultWithPostInfo {
		let hash = T::Hashing::hash(asset);

		match self.assets.contains(&hash) {
			false => {
				self.assets.try_push(hash).map_err(|_| Error::<T>::Full)?;

				// hold storage deposit
				self.reserve_deposit_hash(HoldReason::ArtistAssets)?;

				Ok(().into())
			},
			true => Err(Error::<T>::NotUniqueAsset.into()),
		}
	}

	fn remove_asset(&mut self, asset: &Vec<u8>) -> DispatchResultWithPostInfo {
		let hash = T::Hashing::hash(asset);

		if let Some(pos) = self.assets.iter().position(|&x| x == hash) {
			// refund storage deposit
			self.unreserve_deposit_hash(HoldReason::ArtistAssets)?;

			self.assets.remove(pos);

			Ok(().into())
		} else {
			Err(Error::<T>::NotFound.into())
		}
	}

	fn clear_assets(&mut self) -> Result<(), DispatchErrorWithPostInfo> {
		let actual_deposit =
			T::Currency::balance_on_hold(&HoldReason::ArtistAssets.into(), &self.owner);
		T::Currency::release(
			&HoldReason::ArtistAssets.into(),
			&self.owner,
			actual_deposit,
			Precision::BestEffort,
		)?;

		self.assets = Default::default();

		Ok(())
	}

	fn remove_genre(&mut self, genre: MusicGenre) -> DispatchResultWithPostInfo {
		if let Some(pos) = self.genres.iter().position(|&x| x == genre) {
			self.genres.remove(pos);
			Ok(().into())
		} else {
			Err(Error::<T>::NotFound.into())
		}
	}

	fn reserve_deposit_hash(&self, reason: HoldReason) -> Result<(), DispatchErrorWithPostInfo> {
		let hash_size = T::Hash::max_encoded_len();
		let hash_cost = T::ByteDeposit::get().saturating_mul(hash_size.saturated_into());

		T::Currency::hold(&reason.into(), &self.owner, hash_cost).map_err(|e| e.into())
	}

	fn unreserve_deposit_hash(
		&self,
		reason: HoldReason,
	) -> Result<BalanceOf<T>, DispatchErrorWithPostInfo> {
		let hash_size = T::Hash::max_encoded_len();
		let hash_cost = T::ByteDeposit::get().saturating_mul(hash_size.saturated_into());

		T::Currency::release(&reason.into(), &self.owner, hash_cost, Precision::BestEffort)
			.map_err(|e| e.into())
	}
}
