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

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

use core::marker::PhantomData;
use frame_support::{
	dispatch::{GetDispatchInfo, PostDispatchInfo},
	traits::{Currency, OriginTrait},
};
use frame_system::pallet_prelude::BlockNumberFor;
use pallet_evm::AddressMapping;
use pallet_evm_precompile_nfts_types::solidity::{OptionalPriceWithDirection, OptionalU256};
use precompile_utils::prelude::*;
use sp_core::U256;
use sp_runtime::traits::Dispatchable;

/// Alias for the Balance type for the provided Runtime.
pub type BalanceOf<Runtime> = <<Runtime as pallet_nfts::Config>::Currency as Currency<
	<Runtime as frame_system::Config>::AccountId,
>>::Balance;

/// Alias for the Collection Id type for the provided Runtime.
pub type CollectionIdOf<Runtime> = <Runtime as pallet_nfts::Config>::CollectionId;

/// Alias for the Item Id type for the provided Runtime.
pub type ItemIdOf<Runtime> = <Runtime as pallet_nfts::Config>::ItemId;

pub struct NftsSwapPrecompile<Runtime>(PhantomData<Runtime>);

#[precompile_utils::precompile]
#[precompile::test_concrete_types(mock::Runtime)]
impl<Runtime> NftsSwapPrecompile<Runtime>
where
	Runtime: pallet_nfts::Config + pallet_evm::Config + frame_system::Config,
	Runtime::RuntimeCall: Dispatchable<PostInfo = PostDispatchInfo> + GetDispatchInfo,
	Runtime::RuntimeCall: From<pallet_nfts::Call<Runtime>>,
	<Runtime::RuntimeCall as Dispatchable>::RuntimeOrigin: From<Option<Runtime::AccountId>>,
	<<Runtime as frame_system::Config>::RuntimeCall as Dispatchable>::RuntimeOrigin: OriginTrait,
	ItemIdOf<Runtime>: TryFrom<U256> + Into<U256> + Copy,
	CollectionIdOf<Runtime>: TryFrom<U256> + Into<U256> + Copy,
	BlockNumberFor<Runtime>: TryFrom<U256>,
	BalanceOf<Runtime>: TryFrom<U256>,
{
	// Extrinsics

	#[precompile::public(
		"create_swap(uint256,uint256,uint256,(bool,uint256),(bool,(uint256,uint8)),uint256)"
	)]
	fn create_swap(
		handle: &mut impl PrecompileHandle,
		offered_collection: U256,
		offered_item: U256,
		desired_collection: U256,
		maybe_desired_item: OptionalU256,
		maybe_price: OptionalPriceWithDirection,
		duration: U256,
	) -> EvmResult<bool> {
		handle.record_log_costs_manual(3, 32)?;

		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);

		{
			let offered_collection: CollectionIdOf<Runtime> =
				Self::u256_to_collection_id(offered_collection)?;
			let offered_item: ItemIdOf<Runtime> = Self::u256_to_item_id(offered_item)?;
			let desired_collection: CollectionIdOf<Runtime> =
				Self::u256_to_collection_id(desired_collection)?;
			let maybe_desired_item: Option<ItemIdOf<Runtime>> =
				maybe_desired_item.try_into_option()?;
			let maybe_price: Option<pallet_nfts::PriceWithDirection<BalanceOf<Runtime>>> =
				maybe_price.try_into()?;
			let duration: BlockNumberFor<Runtime> = duration
				.try_into()
				.map_err(|_| RevertReason::value_is_too_large("BlockNumber type"))?;

			let create_swap_call = pallet_nfts::Call::<Runtime>::create_swap {
				offered_collection,
				offered_item,
				desired_collection,
				maybe_desired_item,
				maybe_price,
				duration,
			};

			RuntimeHelper::<Runtime>::try_dispatch(handle, Some(origin).into(), create_swap_call)?;
		}

		Ok(true)
	}

	#[precompile::public("cancel_swap(uint256,uint256)")]
	fn cancel_swap(
		handle: &mut impl PrecompileHandle,
		offered_collection: U256,
		offered_item: U256,
	) -> EvmResult<bool> {
		handle.record_log_costs_manual(3, 32)?;

		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);

		{
			let offered_collection: CollectionIdOf<Runtime> =
				Self::u256_to_collection_id(offered_collection)?;
			let offered_item: ItemIdOf<Runtime> = Self::u256_to_item_id(offered_item)?;

			let cancel_swap_call =
				pallet_nfts::Call::<Runtime>::cancel_swap { offered_collection, offered_item };

			RuntimeHelper::<Runtime>::try_dispatch(handle, Some(origin).into(), cancel_swap_call)?;
		}

		Ok(true)
	}

	#[precompile::public("claim_swap(uint256,uint256,uint256,uint256,(bool,(uint256,uint8)))")]
	fn claim_swap(
		handle: &mut impl PrecompileHandle,
		send_collection: U256,
		send_item: U256,
		receive_collection: U256,
		receive_item: U256,
		witness_price: OptionalPriceWithDirection,
	) -> EvmResult<bool> {
		handle.record_log_costs_manual(3, 32)?;

		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);

		{
			let send_collection: CollectionIdOf<Runtime> =
				Self::u256_to_collection_id(send_collection)?;
			let send_item: ItemIdOf<Runtime> = Self::u256_to_item_id(send_item)?;
			let receive_collection: CollectionIdOf<Runtime> =
				Self::u256_to_collection_id(receive_collection)?;
			let receive_item: ItemIdOf<Runtime> = Self::u256_to_item_id(receive_item)?;
			let witness_price: Option<pallet_nfts::PriceWithDirection<BalanceOf<Runtime>>> =
				witness_price.try_into()?;

			let claim_swap_call = pallet_nfts::Call::<Runtime>::claim_swap {
				send_collection,
				send_item,
				receive_collection,
				receive_item,
				witness_price,
			};

			RuntimeHelper::<Runtime>::try_dispatch(handle, Some(origin).into(), claim_swap_call)?;
		}

		Ok(true)
	}

	fn u256_to_item_id(value: U256) -> MayRevert<ItemIdOf<Runtime>> {
		value
			.try_into()
			.map_err(|_| RevertReason::value_is_too_large("ItemId type").into())
	}

	fn u256_to_collection_id(value: U256) -> MayRevert<CollectionIdOf<Runtime>> {
		value
			.try_into()
			.map_err(|_| RevertReason::value_is_too_large("CollectionId type").into())
	}
}
