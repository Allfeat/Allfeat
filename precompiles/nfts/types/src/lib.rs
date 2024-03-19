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

pub mod solidity;

pub mod alias {
	use frame_support::traits::Currency;
	use frame_system::pallet_prelude::BlockNumberFor;
	use sp_runtime::traits::StaticLookup;

	/// Alias for the Balance type for the provided Runtime.
	pub type BalanceOf<Runtime> = <<Runtime as pallet_nfts::Config>::Currency as Currency<
		<Runtime as frame_system::Config>::AccountId,
	>>::Balance;

	/// Alias for the Collection Id type for the provided Runtime.
	pub type CollectionIdOf<Runtime> = <Runtime as pallet_nfts::Config>::CollectionId;

	/// Alias for the Item Id type for the provided Runtime.
	pub type ItemIdOf<Runtime> = <Runtime as pallet_nfts::Config>::ItemId;

	/// A type alias representing the details of a collection.
	pub type CollectionDetailsOf<T> =
		pallet_nfts::CollectionDetails<<T as frame_system::Config>::AccountId, BalanceOf<T>>;

	/// Alias for the pallet nfts MintWitness type for the provided Runtime.
	pub type MintWitnessFor<Runtime> =
		pallet_nfts::MintWitness<ItemIdOf<Runtime>, BalanceOf<Runtime>>;

	/// Alias for the pallet nfts MintSettings type for the provided Runtime.
	pub type MintSettingsFor<Runtime> = pallet_nfts::MintSettings<
		BalanceOf<Runtime>,
		BlockNumberFor<Runtime>,
		CollectionIdOf<Runtime>,
	>;

	/// A type alias for the pallet_nfts tips held by a single item.
	pub type ItemTipOf<Runtime> = pallet_nfts::ItemTip<
		<Runtime as pallet_nfts::Config>::CollectionId,
		<Runtime as pallet_nfts::Config>::ItemId,
		<Runtime as frame_system::Config>::AccountId,
		BalanceOf<Runtime>,
	>;

	/// Alias for the pallet nfts MintSettings type for the provided Runtime.
	pub type AttributeNamespaceOf<Runtime> =
		pallet_nfts::AttributeNamespace<<Runtime as frame_system::Config>::AccountId>;

	pub type AccountIdLookupOf<T> = <<T as frame_system::Config>::Lookup as StaticLookup>::Source;
}
