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
use fp_evm::ExitError;
use frame_support::{
	dispatch::{GetDispatchInfo, PostDispatchInfo},
	traits::OriginTrait,
	BoundedVec, DefaultNoBound,
};
use frame_system::pallet_prelude::BlockNumberFor;
use pallet_evm::AddressMapping;
use pallet_evm_precompile_nfts_types::{
	alias::*,
	solidity::{
		AttributeNamespaceInfo, CancelAttributesApprovalWitness, CollectionDetails,
		CollectionSettings, MintSettings, MintWitness, OptionalAddress, OptionalMintWitness,
		OptionalU256,
	},
};
use parity_scale_codec::MaxEncodedLen;
use precompile_utils::prelude::*;
use sp_core::{H160, U256};
use sp_runtime::traits::{Dispatchable, StaticLookup};

#[cfg(not(feature = "std"))]
use sp_std::vec::Vec;

/// Alias for the key limit type for the provided Runtime.
type KeyLimitOf<Runtime> = <Runtime as pallet_nfts::Config>::KeyLimit;

/// Alias for the key limit type for the provided Runtime.
type ValueLimitOf<Runtime> = <Runtime as pallet_nfts::Config>::ValueLimit;

/// Alias for the string limit type for the provided Runtime.
type StringLimitOf<Runtime> = <Runtime as pallet_nfts::Config>::StringLimit;

/// This trait ensure we can convert EVM address to CollectionIds
/// We will require Runtime to have this trait implemented
pub trait AddressToCollectionId<CollectionId> {
	// Get assetId from address
	fn address_to_collection_id(address: H160) -> Option<CollectionId>;

	// Get address from AssetId
	fn collection_id_to_address(collection_id: CollectionId) -> H160;
}

/// The following distribution has been decided for the precompiles
/// 0-1023: Ethereum Mainnet Precompiles
/// 1024-2047 Precompiles that are not in Ethereum Mainnet but are neither Allfeat specific
/// 2048-4095 Astar specific precompiles
/// Nfts precompiles can only fall between
///     0xFFFFFFFF00000000000000000000000000000000 - 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF
/// The precompile for CollectionId X, where X is a u128 (i.e.16 bytes), if 0XFFFFFFFF +
/// Bytes(CollectionId) In order to route the address to NftsPrecompile<Runtime>, we first check
/// whether the CollectionId exists in pallet-nfts
/// We cannot do this right now, so instead we check whether the owner is not set. If so, we
/// do not route to the precompiles

/// This means that every address that starts with 0xFFFFFFFF will go through an additional db read,
/// but the probability for this to happen is 2^-32 for random addresses
#[derive(Clone, DefaultNoBound)]
pub struct NftsPrecompileSet<Runtime>(PhantomData<Runtime>);

impl<Runtime> NftsPrecompileSet<Runtime> {
	pub fn new() -> Self {
		Self(PhantomData)
	}
}

#[precompile_utils::precompile]
#[precompile::test_concrete_types(mock::Runtime)]
#[precompile::precompile_set]
impl<Runtime> NftsPrecompileSet<Runtime>
where
	Runtime: pallet_nfts::Config + pallet_evm::Config + frame_system::Config,
	Runtime::RuntimeCall: Dispatchable<PostInfo = PostDispatchInfo> + GetDispatchInfo,
	Runtime::RuntimeCall: From<pallet_nfts::Call<Runtime>>,
	Runtime: AddressToCollectionId<CollectionIdOf<Runtime>>,
	<Runtime as frame_system::Config>::AccountId: Into<H160>,
	<Runtime::RuntimeCall as Dispatchable>::RuntimeOrigin: From<Option<Runtime::AccountId>>,
	BalanceOf<Runtime>: TryFrom<U256> + Into<U256> + solidity::Codec,
	<<Runtime as frame_system::Config>::RuntimeCall as Dispatchable>::RuntimeOrigin: OriginTrait,
	ItemIdOf<Runtime>: TryFrom<U256> + Into<U256> + Copy,
	MintWitnessFor<Runtime>: TryFrom<MintWitness, Error = Revert>,
	BlockNumberFor<Runtime>: TryFrom<U256>,
	MintSettingsFor<Runtime>: TryFrom<MintSettings, Error = Revert>,
{
	/// PrecompileSet discriminant. Allows to knows if the address maps to an asset id,
	/// and if this is the case which one.
	#[precompile::discriminant]
	fn discriminant(address: H160, gas: u64) -> DiscriminantResult<CollectionIdOf<Runtime>> {
		let extra_cost = RuntimeHelper::<Runtime>::db_read_gas_cost();
		if gas < extra_cost {
			return DiscriminantResult::OutOfGas;
		}

		let collection_id = match Runtime::address_to_collection_id(address) {
			Some(collection_id) => collection_id,
			None => return DiscriminantResult::None(extra_cost),
		};

		if pallet_nfts::Collection::<Runtime>::get(collection_id).is_some() {
			DiscriminantResult::Some(collection_id, extra_cost)
		} else {
			DiscriminantResult::None(extra_cost)
		}
	}

	// Getters
	#[precompile::public("get_details()")]
	#[precompile::view]
	fn get_details(
		collection_id: CollectionIdOf<Runtime>,
		handle: &mut impl PrecompileHandle,
	) -> EvmResult<CollectionDetails> {
		// TODO: benchmark this function so we can measure ref time & PoV correctly
		// Storage item: Asset:
		// Blake2_128(16) + CollectionId + CollectionDetails
		handle.record_db_read::<Runtime>(
			16 + CollectionIdOf::<Runtime>::max_encoded_len()
				+ CollectionDetailsOf::<Runtime>::max_encoded_len(),
		)?;

		let collection_details = pallet_nfts::Collection::<Runtime>::get(collection_id)
			.expect("checked with discriminant");

		Ok(collection_details.into())
	}

	// Extrinsics
	#[precompile::public("mint(uint256,address,(bool,((bool,uint256),(bool,uint256))))")]
	fn mint(
		collection_id: CollectionIdOf<Runtime>,
		handle: &mut impl PrecompileHandle,
		item_id: U256,
		mint_to: Address,
		witness_data: OptionalMintWitness,
	) -> EvmResult<bool> {
		handle.record_log_costs_manual(3, 32)?;

		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);
		let item_id: ItemIdOf<Runtime> = Self::u256_to_item_id(item_id)?;
		let witness_data: Option<MintWitnessFor<Runtime>> = witness_data.try_into()?;
		let mint_to: H160 = mint_to.into();

		{
			let mint_to: Runtime::AccountId = Runtime::AddressMapping::into_account_id(mint_to);
			let mint_call = pallet_nfts::Call::<Runtime>::mint {
				collection: collection_id,
				item: item_id,
				mint_to: Runtime::Lookup::unlookup(mint_to),
				witness_data,
			};

			RuntimeHelper::<Runtime>::try_dispatch(handle, Some(origin).into(), mint_call)?;
		}

		Ok(true)
	}

	#[precompile::public("burn(uint256)")]
	fn burn(
		collection_id: CollectionIdOf<Runtime>,
		handle: &mut impl PrecompileHandle,
		item_id: U256,
	) -> EvmResult<bool> {
		handle.record_log_costs_manual(3, 32)?;

		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);
		let item_id: ItemIdOf<Runtime> = Self::u256_to_item_id(item_id)?;

		{
			let burn_call =
				pallet_nfts::Call::<Runtime>::burn { collection: collection_id, item: item_id };

			RuntimeHelper::<Runtime>::try_dispatch(handle, Some(origin).into(), burn_call)?;
		}

		Ok(true)
	}

	#[precompile::public("transfer(uint256,address)")]
	fn transfer(
		collection_id: CollectionIdOf<Runtime>,
		handle: &mut impl PrecompileHandle,
		item_id: U256,
		dest: Address,
	) -> EvmResult<bool> {
		handle.record_log_costs_manual(3, 32)?;

		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);
		let item_id: ItemIdOf<Runtime> = Self::u256_to_item_id(item_id)?;
		let dest: H160 = dest.into();

		{
			let dest: Runtime::AccountId = Runtime::AddressMapping::into_account_id(dest);
			let transfer_call = pallet_nfts::Call::<Runtime>::transfer {
				collection: collection_id,
				item: item_id,
				dest: Runtime::Lookup::unlookup(dest),
			};

			RuntimeHelper::<Runtime>::try_dispatch(handle, Some(origin).into(), transfer_call)?;
		}

		Ok(true)
	}

	#[precompile::public("lock_item_transfer(uint256)")]
	fn lock_item_transfer(
		collection_id: CollectionIdOf<Runtime>,
		handle: &mut impl PrecompileHandle,
		item_id: U256,
	) -> EvmResult<bool> {
		handle.record_log_costs_manual(3, 32)?;

		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);
		let item_id: ItemIdOf<Runtime> = Self::u256_to_item_id(item_id)?;

		{
			let lock_item_transfer_call = pallet_nfts::Call::<Runtime>::lock_item_transfer {
				collection: collection_id,
				item: item_id,
			};

			RuntimeHelper::<Runtime>::try_dispatch(
				handle,
				Some(origin).into(),
				lock_item_transfer_call,
			)?;
		}

		Ok(true)
	}

	#[precompile::public("unlock_item_transfer(uint256)")]
	fn unlock_item_transfer(
		collection_id: CollectionIdOf<Runtime>,
		handle: &mut impl PrecompileHandle,
		item_id: U256,
	) -> EvmResult<bool> {
		handle.record_log_costs_manual(3, 32)?;

		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);
		let item_id: ItemIdOf<Runtime> = Self::u256_to_item_id(item_id)?;

		{
			let unlock_item_transfer_call = pallet_nfts::Call::<Runtime>::unlock_item_transfer {
				collection: collection_id,
				item: item_id,
			};

			RuntimeHelper::<Runtime>::try_dispatch(
				handle,
				Some(origin).into(),
				unlock_item_transfer_call,
			)?;
		}

		Ok(true)
	}

	#[precompile::public("seal_collection((bool,bool,bool,bool,bool))")]
	fn seal_collection(
		collection_id: CollectionIdOf<Runtime>,
		handle: &mut impl PrecompileHandle,
		settings: CollectionSettings,
	) -> EvmResult<bool> {
		handle.record_log_costs_manual(3, 32)?;

		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);
		let settings: pallet_nfts::CollectionSettings = settings.into();

		{
			let lock_collection_call = pallet_nfts::Call::<Runtime>::lock_collection {
				collection: collection_id,
				lock_settings: settings,
			};

			RuntimeHelper::<Runtime>::try_dispatch(
				handle,
				Some(origin).into(),
				lock_collection_call,
			)?;
		}

		Ok(true)
	}

	#[precompile::public("transfer_ownership(address)")]
	fn transfer_ownership(
		collection_id: CollectionIdOf<Runtime>,
		handle: &mut impl PrecompileHandle,
		owner: Address,
	) -> EvmResult<bool> {
		handle.record_log_costs_manual(3, 32)?;

		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);
		let owner: H160 = owner.into();

		{
			let owner: Runtime::AccountId = Runtime::AddressMapping::into_account_id(owner);
			let transfer_ownership_call = pallet_nfts::Call::<Runtime>::transfer_ownership {
				collection: collection_id,
				new_owner: Runtime::Lookup::unlookup(owner),
			};

			RuntimeHelper::<Runtime>::try_dispatch(
				handle,
				Some(origin).into(),
				transfer_ownership_call,
			)?;
		}

		Ok(true)
	}

	#[precompile::public("set_team(address,address,address)")]
	fn set_team(
		collection_id: CollectionIdOf<Runtime>,
		handle: &mut impl PrecompileHandle,
		issuer: Address,
		admin: Address,
		freezer: Address,
	) -> EvmResult<bool> {
		handle.record_log_costs_manual(3, 32)?;

		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);
		let issuer: H160 = issuer.into();
		let admin: H160 = admin.into();
		let freezer: H160 = freezer.into();

		{
			let issuer: Runtime::AccountId = Runtime::AddressMapping::into_account_id(issuer);
			let admin: Runtime::AccountId = Runtime::AddressMapping::into_account_id(admin);
			let freezer: Runtime::AccountId = Runtime::AddressMapping::into_account_id(freezer);
			let set_team_call = pallet_nfts::Call::<Runtime>::set_team {
				collection: collection_id,
				issuer: Some(Runtime::Lookup::unlookup(issuer)),
				admin: Some(Runtime::Lookup::unlookup(admin)),
				freezer: Some(Runtime::Lookup::unlookup(freezer)),
			};

			RuntimeHelper::<Runtime>::try_dispatch(handle, Some(origin).into(), set_team_call)?;
		}

		Ok(true)
	}

	#[precompile::public("approve_transfer(uint256,address,(bool,uint256))")]
	fn approve_transfer(
		collection_id: CollectionIdOf<Runtime>,
		handle: &mut impl PrecompileHandle,
		item: U256,
		delegate: Address,
		maybe_deadline: OptionalU256,
	) -> EvmResult<bool> {
		handle.record_log_costs_manual(3, 32)?;

		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);
		let delegate: H160 = delegate.into();
		let item: ItemIdOf<Runtime> = Self::u256_to_item_id(item)?;
		let maybe_deadline: Option<BlockNumberFor<Runtime>> = maybe_deadline.try_into_option()?;

		{
			let delegate: Runtime::AccountId = Runtime::AddressMapping::into_account_id(delegate);
			let approve_transfer_call = pallet_nfts::Call::<Runtime>::approve_transfer {
				collection: collection_id,
				item,
				delegate: Runtime::Lookup::unlookup(delegate),
				maybe_deadline,
			};

			RuntimeHelper::<Runtime>::try_dispatch(
				handle,
				Some(origin).into(),
				approve_transfer_call,
			)?;
		}

		Ok(true)
	}

	#[precompile::public("cancel_approval(uint256,address)")]
	fn cancel_approval(
		collection_id: CollectionIdOf<Runtime>,
		handle: &mut impl PrecompileHandle,
		item: U256,
		delegate: Address,
	) -> EvmResult<bool> {
		handle.record_log_costs_manual(3, 32)?;

		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);
		let delegate: H160 = delegate.into();
		let item: ItemIdOf<Runtime> = Self::u256_to_item_id(item)?;

		{
			let delegate: Runtime::AccountId = Runtime::AddressMapping::into_account_id(delegate);
			let cancel_approval_call = pallet_nfts::Call::<Runtime>::cancel_approval {
				collection: collection_id,
				item,
				delegate: Runtime::Lookup::unlookup(delegate),
			};

			RuntimeHelper::<Runtime>::try_dispatch(
				handle,
				Some(origin).into(),
				cancel_approval_call,
			)?;
		}

		Ok(true)
	}

	#[precompile::public("cancel_all_approvals(uint256)")]
	fn clear_all_transfer_approvals(
		collection_id: CollectionIdOf<Runtime>,
		handle: &mut impl PrecompileHandle,
		item: U256,
	) -> EvmResult<bool> {
		handle.record_log_costs_manual(3, 32)?;

		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);
		let item: ItemIdOf<Runtime> = Self::u256_to_item_id(item)?;

		{
			let clear_all_transfer_approvals_call =
				pallet_nfts::Call::<Runtime>::clear_all_transfer_approvals {
					collection: collection_id,
					item,
				};

			RuntimeHelper::<Runtime>::try_dispatch(
				handle,
				Some(origin).into(),
				clear_all_transfer_approvals_call,
			)?;
		}

		Ok(true)
	}

	#[precompile::public("seal_item_properties(uint256,bool,bool)")]
	fn lock_item_properties(
		collection_id: CollectionIdOf<Runtime>,
		handle: &mut impl PrecompileHandle,
		item: U256,
		lock_metadata: bool,
		lock_attributes: bool,
	) -> EvmResult<bool> {
		handle.record_log_costs_manual(3, 32)?;

		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);
		let item: ItemIdOf<Runtime> = Self::u256_to_item_id(item)?;

		{
			let lock_item_properties_call = pallet_nfts::Call::<Runtime>::lock_item_properties {
				collection: collection_id,
				item,
				lock_metadata,
				lock_attributes,
			};

			RuntimeHelper::<Runtime>::try_dispatch(
				handle,
				Some(origin).into(),
				lock_item_properties_call,
			)?;
		}

		Ok(true)
	}

	#[precompile::public("set_collection_attribute((uint8,address),string,string)")]
	fn set_collection_attribute(
		collection_id: CollectionIdOf<Runtime>,
		handle: &mut impl PrecompileHandle,
		namespace: AttributeNamespaceInfo,
		key: BoundedString<KeyLimitOf<Runtime>>,
		value: BoundedString<ValueLimitOf<Runtime>>,
	) -> EvmResult<bool> {
		handle.record_log_costs_manual(3, 32)?;

		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);

		{
			let namespace = namespace.into_pallet_type::<Runtime>();
			let key: Vec<u8> = key.into();
			let key: BoundedVec<u8, KeyLimitOf<Runtime>> =
				key.try_into().map_err(|_| RevertReason::value_is_too_large("key type"))?;
			let value: Vec<u8> = value.into();
			let value: BoundedVec<u8, ValueLimitOf<Runtime>> =
				value.try_into().map_err(|_| RevertReason::value_is_too_large("value type"))?;

			let set_attribute_call = pallet_nfts::Call::<Runtime>::set_attribute {
				collection: collection_id,
				maybe_item: None,
				namespace,
				key,
				value,
			};

			RuntimeHelper::<Runtime>::try_dispatch(
				handle,
				Some(origin).into(),
				set_attribute_call,
			)?;
		}

		Ok(true)
	}

	#[precompile::public("set_item_attribute(uint256,(uint8,address),string,string)")]
	fn set_item_attribute(
		collection_id: CollectionIdOf<Runtime>,
		handle: &mut impl PrecompileHandle,
		item: U256,
		namespace: AttributeNamespaceInfo,
		key: BoundedString<KeyLimitOf<Runtime>>,
		value: BoundedString<ValueLimitOf<Runtime>>,
	) -> EvmResult<bool> {
		handle.record_log_costs_manual(3, 32)?;

		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);

		{
			let item: ItemIdOf<Runtime> = Self::u256_to_item_id(item)?;
			let namespace = namespace.into_pallet_type::<Runtime>();
			let key: Vec<u8> = key.into();
			let key: BoundedVec<u8, KeyLimitOf<Runtime>> =
				key.try_into().map_err(|_| RevertReason::value_is_too_large("key type"))?;
			let value: Vec<u8> = value.into();
			let value: BoundedVec<u8, ValueLimitOf<Runtime>> =
				value.try_into().map_err(|_| RevertReason::value_is_too_large("value type"))?;

			let set_attribute_call = pallet_nfts::Call::<Runtime>::set_attribute {
				collection: collection_id,
				maybe_item: Some(item),
				namespace,
				key,
				value,
			};

			RuntimeHelper::<Runtime>::try_dispatch(
				handle,
				Some(origin).into(),
				set_attribute_call,
			)?;
		}

		Ok(true)
	}

	#[precompile::public("clear_collection_attribute((uint8,address),string)")]
	fn clear_collection_attribute(
		collection_id: CollectionIdOf<Runtime>,
		handle: &mut impl PrecompileHandle,
		namespace: AttributeNamespaceInfo,
		key: BoundedString<KeyLimitOf<Runtime>>,
	) -> EvmResult<bool> {
		handle.record_log_costs_manual(3, 32)?;

		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);

		{
			let namespace = namespace.into_pallet_type::<Runtime>();
			let key: Vec<u8> = key.into();
			let key: BoundedVec<u8, KeyLimitOf<Runtime>> =
				key.try_into().map_err(|_| RevertReason::value_is_too_large("key type"))?;

			let clear_attribute_call = pallet_nfts::Call::<Runtime>::clear_attribute {
				collection: collection_id,
				maybe_item: None,
				namespace,
				key,
			};

			RuntimeHelper::<Runtime>::try_dispatch(
				handle,
				Some(origin).into(),
				clear_attribute_call,
			)?;
		}

		Ok(true)
	}

	#[precompile::public("clear_item_attribute(uint256,(uint8,address),string)")]
	fn clear_item_attribute(
		collection_id: CollectionIdOf<Runtime>,
		handle: &mut impl PrecompileHandle,
		item: U256,
		namespace: AttributeNamespaceInfo,
		key: BoundedString<KeyLimitOf<Runtime>>,
	) -> EvmResult<bool> {
		handle.record_log_costs_manual(3, 32)?;

		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);

		{
			let item: ItemIdOf<Runtime> = Self::u256_to_item_id(item)?;
			let namespace = namespace.into_pallet_type::<Runtime>();
			let key: Vec<u8> = key.into();
			let key: BoundedVec<u8, KeyLimitOf<Runtime>> =
				key.try_into().map_err(|_| RevertReason::value_is_too_large("key type"))?;

			let clear_attribute_call = pallet_nfts::Call::<Runtime>::clear_attribute {
				collection: collection_id,
				maybe_item: Some(item),
				namespace,
				key,
			};

			RuntimeHelper::<Runtime>::try_dispatch(
				handle,
				Some(origin).into(),
				clear_attribute_call,
			)?;
		}

		Ok(true)
	}

	#[precompile::public("approve_item_attributes(uint256,address)")]
	fn approve_item_attributes(
		collection_id: CollectionIdOf<Runtime>,
		handle: &mut impl PrecompileHandle,
		item: U256,
		delegate: Address,
	) -> EvmResult<bool> {
		handle.record_log_costs_manual(3, 32)?;

		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);
		let delegate: H160 = delegate.into();

		{
			let item: ItemIdOf<Runtime> = Self::u256_to_item_id(item)?;
			let delegate: Runtime::AccountId = Runtime::AddressMapping::into_account_id(delegate);

			let approve_item_attributes_call =
				pallet_nfts::Call::<Runtime>::approve_item_attributes {
					collection: collection_id,
					item,
					delegate: Runtime::Lookup::unlookup(delegate),
				};

			RuntimeHelper::<Runtime>::try_dispatch(
				handle,
				Some(origin).into(),
				approve_item_attributes_call,
			)?;
		}

		Ok(true)
	}

	#[precompile::public("cancel_item_attributes_approval(uint256,address,(uint256))")]
	fn cancel_item_attributes_approval(
		collection_id: CollectionIdOf<Runtime>,
		handle: &mut impl PrecompileHandle,
		item: U256,
		delegate: Address,
		witness: CancelAttributesApprovalWitness,
	) -> EvmResult<bool> {
		handle.record_log_costs_manual(3, 32)?;

		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);
		let delegate: H160 = delegate.into();

		{
			let item: ItemIdOf<Runtime> = Self::u256_to_item_id(item)?;
			let witness: pallet_nfts::CancelAttributesApprovalWitness = witness.try_into()?;
			let delegate: Runtime::AccountId = Runtime::AddressMapping::into_account_id(delegate);

			let cancel_item_attributes_approval_call =
				pallet_nfts::Call::<Runtime>::cancel_item_attributes_approval {
					collection: collection_id,
					item,
					delegate: Runtime::Lookup::unlookup(delegate),
					witness,
				};

			RuntimeHelper::<Runtime>::try_dispatch(
				handle,
				Some(origin).into(),
				cancel_item_attributes_approval_call,
			)?;
		}

		Ok(true)
	}

	#[precompile::public("set_metadata(uint256,string)")]
	fn set_metadata(
		collection_id: CollectionIdOf<Runtime>,
		handle: &mut impl PrecompileHandle,
		item: U256,
		data: BoundedString<StringLimitOf<Runtime>>,
	) -> EvmResult<bool> {
		handle.record_log_costs_manual(3, 32)?;

		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);

		{
			let item: ItemIdOf<Runtime> = Self::u256_to_item_id(item)?;
			let data: Vec<u8> = data.into();
			let data: BoundedVec<u8, StringLimitOf<Runtime>> =
				data.try_into().map_err(|_| RevertReason::value_is_too_large("data type"))?;

			let set_metadata_call = pallet_nfts::Call::<Runtime>::set_metadata {
				collection: collection_id,
				item,
				data,
			};

			RuntimeHelper::<Runtime>::try_dispatch(handle, Some(origin).into(), set_metadata_call)?;
		}

		Ok(true)
	}

	#[precompile::public("clear_metadata(uint256)")]
	fn clear_metadata(
		collection_id: CollectionIdOf<Runtime>,
		handle: &mut impl PrecompileHandle,
		item: U256,
	) -> EvmResult<bool> {
		handle.record_log_costs_manual(3, 32)?;

		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);

		{
			let item: ItemIdOf<Runtime> = Self::u256_to_item_id(item)?;

			let clear_metadata_call =
				pallet_nfts::Call::<Runtime>::clear_metadata { collection: collection_id, item };

			RuntimeHelper::<Runtime>::try_dispatch(
				handle,
				Some(origin).into(),
				clear_metadata_call,
			)?;
		}

		Ok(true)
	}

	#[precompile::public("set_collection_metadata(string)")]
	fn set_collection_metadata(
		collection_id: CollectionIdOf<Runtime>,
		handle: &mut impl PrecompileHandle,
		data: BoundedString<StringLimitOf<Runtime>>,
	) -> EvmResult<bool> {
		handle.record_log_costs_manual(3, 32)?;

		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);

		{
			let data: Vec<u8> = data.into();
			let data: BoundedVec<u8, StringLimitOf<Runtime>> =
				data.try_into().map_err(|_| RevertReason::value_is_too_large("data type"))?;

			let set_collection_metadata_call =
				pallet_nfts::Call::<Runtime>::set_collection_metadata {
					collection: collection_id,
					data,
				};

			RuntimeHelper::<Runtime>::try_dispatch(
				handle,
				Some(origin).into(),
				set_collection_metadata_call,
			)?;
		}

		Ok(true)
	}

	#[precompile::public("clear_collection_metadata()")]
	fn clear_collection_metadata(
		collection_id: CollectionIdOf<Runtime>,
		handle: &mut impl PrecompileHandle,
	) -> EvmResult<bool> {
		handle.record_log_costs_manual(3, 32)?;

		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);

		{
			let clear_collection_metadata_call =
				pallet_nfts::Call::<Runtime>::clear_collection_metadata {
					collection: collection_id,
				};

			RuntimeHelper::<Runtime>::try_dispatch(
				handle,
				Some(origin).into(),
				clear_collection_metadata_call,
			)?;
		}

		Ok(true)
	}

	#[precompile::public("set_collection_max_supply(uint32)")]
	fn set_collection_max_supply(
		collection_id: CollectionIdOf<Runtime>,
		handle: &mut impl PrecompileHandle,
		max_supply: u32,
	) -> EvmResult<bool> {
		handle.record_log_costs_manual(3, 32)?;

		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);

		{
			let set_collection_max_supply_call =
				pallet_nfts::Call::<Runtime>::set_collection_max_supply {
					collection: collection_id,
					max_supply,
				};

			RuntimeHelper::<Runtime>::try_dispatch(
				handle,
				Some(origin).into(),
				set_collection_max_supply_call,
			)?;
		}

		Ok(true)
	}

	#[precompile::public("update_mint_settings(((uint8,uint256),(bool,uint256),(bool,uint256),(bool,uint256),(bool,bool,bool)))")]
	fn update_mint_settings(
		collection_id: CollectionIdOf<Runtime>,
		handle: &mut impl PrecompileHandle,
		mint_settings: MintSettings,
	) -> EvmResult<bool> {
		handle.record_log_costs_manual(3, 32)?;

		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);

		{
			let mint_settings: MintSettingsFor<Runtime> = mint_settings.try_into()?;

			let update_mint_settings_call = pallet_nfts::Call::<Runtime>::update_mint_settings {
				collection: collection_id,
				mint_settings,
			};

			RuntimeHelper::<Runtime>::try_dispatch(
				handle,
				Some(origin).into(),
				update_mint_settings_call,
			)?;
		}

		Ok(true)
	}

	#[precompile::public("set_price(uint256,(bool,uint256),(bool,address))")]
	fn set_price(
		collection_id: CollectionIdOf<Runtime>,
		handle: &mut impl PrecompileHandle,
		item: U256,
		price: OptionalU256,
		whitelisted_buyer: OptionalAddress,
	) -> EvmResult<bool> {
		handle.record_log_costs_manual(3, 32)?;

		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);

		{
			let item: ItemIdOf<Runtime> = Self::u256_to_item_id(item)?;
			let price: Option<BalanceOf<Runtime>> = price.try_into_option()?;
			let whitelisted_buyer: Option<AccountIdLookupOf<Runtime>> =
				whitelisted_buyer.into_option_lookup::<Runtime>();

			let set_price_call = pallet_nfts::Call::<Runtime>::set_price {
				collection: collection_id,
				item,
				price,
				whitelisted_buyer,
			};

			RuntimeHelper::<Runtime>::try_dispatch(handle, Some(origin).into(), set_price_call)?;
		}

		Ok(true)
	}

	#[precompile::public("buy_item(uint256,uint256)")]
	fn buy_item(
		collection_id: CollectionIdOf<Runtime>,
		handle: &mut impl PrecompileHandle,
		item: U256,
		bid_price: U256,
	) -> EvmResult<bool> {
		handle.record_log_costs_manual(3, 32)?;

		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);

		{
			let item: ItemIdOf<Runtime> = Self::u256_to_item_id(item)?;
			let bid_price: BalanceOf<Runtime> = Self::u256_to_balance(bid_price)?;

			let buy_item_call = pallet_nfts::Call::<Runtime>::buy_item {
				collection: collection_id,
				item,
				bid_price,
			};

			RuntimeHelper::<Runtime>::try_dispatch(handle, Some(origin).into(), buy_item_call)?;
		}

		Ok(true)
	}

	fn u256_to_item_id(value: U256) -> MayRevert<ItemIdOf<Runtime>> {
		value
			.try_into()
			.map_err(|_| RevertReason::value_is_too_large("ItemId type").into())
	}

	fn u256_to_balance(value: U256) -> MayRevert<BalanceOf<Runtime>> {
		value
			.try_into()
			.map_err(|_| RevertReason::value_is_too_large("Balance type").into())
	}
}
