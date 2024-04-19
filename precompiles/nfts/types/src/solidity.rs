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

use super::alias::*;
use num_enum::{IntoPrimitive, TryFromPrimitive};
use pallet_evm::AddressMapping;
use precompile_utils::{
	prelude::*,
	solidity::{
		codec::{Reader, Writer},
		Codec,
	},
};
use sp_core::{H160, U256};
use sp_runtime::traits::StaticLookup;

/// Helper struct used to encode Optional uint256 Solidity types.
#[derive(Default, Debug, Clone, solidity::Codec)]
pub struct OptionalU256 {
	pub has_value: bool,
	pub value: U256,
}

impl OptionalU256 {
	pub fn try_into_option<T: TryFrom<U256>>(self) -> MayRevert<Option<T>> {
		match self.has_value {
			true => {
				let x = self
					.value
					.try_into()
					.map_err(|_| RevertReason::value_is_too_large("OptionalU256 type value"))?;
				Ok(Some(x))
			},
			false => Ok(None),
		}
	}
}

impl From<Option<U256>> for OptionalU256 {
	fn from(value: Option<U256>) -> Self {
		match value {
			Some(value) => Self { has_value: true, value },
			None => Self { has_value: false, value: Default::default() },
		}
	}
}

/// Helper struct used to encode Optional MintWitness Solidity types.
#[derive(Default, Debug, Clone, solidity::Codec)]
pub struct OptionalMintWitness {
	pub has_witness: bool,
	pub data: MintWitness,
}

impl<I, B> TryFrom<OptionalMintWitness> for Option<pallet_nfts::MintWitness<I, B>>
where
	I: TryFrom<U256>,
	B: TryFrom<U256>,
{
	type Error = Revert;

	fn try_from(value: OptionalMintWitness) -> MayRevert<Self> {
		Ok(match value.has_witness {
			true => Some(value.data.try_into()?),
			false => None,
		})
	}
}

/// Helper struct used to encode Optional PriceWithDirection Solidity types.
#[derive(Default, Debug, Clone, solidity::Codec)]
pub struct OptionalPriceWithDirection {
	pub has_price_direction: bool,
	pub value: PriceWithDirection,
}

impl From<Option<PriceWithDirection>> for OptionalPriceWithDirection {
	fn from(value: Option<PriceWithDirection>) -> Self {
		match value {
			Some(value) => Self { has_price_direction: true, value },
			None => Self { has_price_direction: false, value: Default::default() },
		}
	}
}

impl<Amount> TryFrom<OptionalPriceWithDirection> for Option<pallet_nfts::PriceWithDirection<Amount>>
where
	Amount: TryFrom<U256>,
{
	type Error = Revert;

	fn try_from(value: OptionalPriceWithDirection) -> MayRevert<Self> {
		Ok(match value.has_price_direction {
			true => Some(value.value.try_into()?),
			false => None,
		})
	}
}

/// Helper struct used to encode CollectionConfig types.
#[derive(Default, Debug, Clone, solidity::Codec)]
pub struct CollectionConfig {
	pub settings: CollectionSettings,
	pub max_supply: OptionalU256,
	pub mint_settings: MintSettings,
}

impl<P, B, T> TryFrom<CollectionConfig> for pallet_nfts::CollectionConfig<P, B, T>
where
	P: TryFrom<U256>,
	B: TryFrom<U256>,
	T: TryFrom<U256>,
{
	type Error = Revert;

	fn try_from(value: CollectionConfig) -> MayRevert<Self> {
		Ok(Self {
			settings: value.settings.into(),
			max_supply: value.max_supply.try_into_option()?,
			mint_settings: value.mint_settings.try_into()?,
		})
	}
}

/// Helper struct used to encode CollectionSettings types.
#[derive(Default, Debug, Clone, solidity::Codec)]
pub struct CollectionSettings {
	pub is_transferable_items: bool,
	pub is_unlocked_metadata: bool,
	pub is_unlocked_attributes: bool,
	pub is_unlocked_max_supply: bool,
	pub is_deposit_required: bool,
}

impl CollectionSettings {
	pub fn all_enabled() -> Self {
		Self {
			is_transferable_items: true,
			is_unlocked_metadata: true,
			is_unlocked_attributes: true,
			is_unlocked_max_supply: true,
			is_deposit_required: true,
		}
	}
}

impl Into<pallet_nfts::CollectionSettings> for CollectionSettings {
	fn into(self) -> pallet_nfts::CollectionSettings {
		let mut s = pallet_nfts::CollectionSettings::all_enabled();

		if !self.is_transferable_items {
			s.0.insert(pallet_nfts::CollectionSetting::TransferableItems)
		}
		if !self.is_unlocked_metadata {
			s.0.insert(pallet_nfts::CollectionSetting::UnlockedMetadata)
		}
		if !self.is_unlocked_attributes {
			s.0.insert(pallet_nfts::CollectionSetting::UnlockedAttributes)
		}
		if !self.is_unlocked_max_supply {
			s.0.insert(pallet_nfts::CollectionSetting::UnlockedMaxSupply)
		}
		if !self.is_deposit_required {
			s.0.insert(pallet_nfts::CollectionSetting::DepositRequired)
		}

		s
	}
}

/// Helper struct used to encode ItemSettings types.
#[derive(Default, Debug, Clone, solidity::Codec)]
pub struct ItemSettings {
	pub is_transferable: bool,
	pub is_unlocked_metadata: bool,
	pub is_unlocked_attributes: bool,
}

impl ItemSettings {
	pub fn all_enabled() -> Self {
		Self { is_transferable: true, is_unlocked_metadata: true, is_unlocked_attributes: true }
	}
}

impl Into<pallet_nfts::ItemSettings> for ItemSettings {
	fn into(self) -> pallet_nfts::ItemSettings {
		let mut s = pallet_nfts::ItemSettings::all_enabled();

		if !self.is_transferable {
			s.0.set(pallet_nfts::ItemSetting::Transferable, false)
		}
		if !self.is_unlocked_metadata {
			s.0.set(pallet_nfts::ItemSetting::UnlockedMetadata, false)
		}
		if !self.is_unlocked_attributes {
			s.0.set(pallet_nfts::ItemSetting::UnlockedAttributes, false)
		}

		s
	}
}

/// Helper struct used to encode MintSettings types.
#[derive(Default, Debug, Clone, solidity::Codec)]
pub struct MintSettings {
	pub mint_type: MintInfo,
	pub price: OptionalU256,
	pub start_block: OptionalU256,
	pub end_block: OptionalU256,
	pub default_item_settings: ItemSettings,
}

impl MintSettings {
	pub fn item_settings_all_enabled() -> Self {
		Self {
			mint_type: Default::default(),
			price: Default::default(),
			start_block: Default::default(),
			end_block: Default::default(),
			default_item_settings: ItemSettings::all_enabled(),
		}
	}
}

impl<P, B, T> TryFrom<MintSettings> for pallet_nfts::MintSettings<P, B, T>
where
	P: TryFrom<U256>,
	B: TryFrom<U256>,
	T: TryFrom<U256>,
{
	type Error = Revert;
	fn try_from(value: MintSettings) -> MayRevert<Self> {
		Ok(Self {
			mint_type: value.mint_type.try_into()?,
			price: value.price.try_into_option()?,
			start_block: value.start_block.try_into_option()?,
			end_block: value.end_block.try_into_option()?,
			default_item_settings: value.default_item_settings.into(),
		})
	}
}

/// Convenience type used to encode MintType types.
#[derive(Default, Clone, Debug, Eq, PartialEq, IntoPrimitive, TryFromPrimitive)]
#[repr(u8)]
pub enum MintType {
	#[default]
	Issuer,
	Public,
	HolderOf,
}

impl Codec for MintType {
	fn read(reader: &mut Reader) -> MayRevert<MintType> {
		let value256: U256 =
			reader.read().map_err(|_| RevertReason::read_out_of_bounds(Self::signature()))?;

		let value_as_u8: u8 = value256
			.try_into()
			.map_err(|_| RevertReason::value_is_too_large(Self::signature()))?;

		value_as_u8
			.try_into()
			.map_err(|_| RevertReason::custom("Unknown mint type").into())
	}

	fn write(writer: &mut Writer, value: Self) {
		let value_as_u8: u8 = value.into();
		U256::write(writer, value_as_u8.into());
	}

	fn has_static_size() -> bool {
		true
	}

	fn signature() -> String {
		"uint8".into()
	}
}

/// Helper struct used to encode MintType types.
#[derive(Default, Debug, Clone, solidity::Codec)]
pub struct MintInfo {
	pub mint_type: MintType,
	pub collection_id: U256,
}

impl<T> TryFrom<MintInfo> for pallet_nfts::MintType<T>
where
	T: TryFrom<U256>,
{
	type Error = Revert;

	fn try_from(value: MintInfo) -> MayRevert<Self> {
		match value.mint_type {
			MintType::Issuer => Ok(Self::Issuer),
			MintType::Public => Ok(Self::Public),
			MintType::HolderOf => Ok(Self::HolderOf(
				value
					.collection_id
					.try_into()
					.map_err(|_| RevertReason::value_is_too_large("collection id type"))?,
			)),
		}
	}
}

/// Helper struct used to encode MintWitness types.
#[derive(Default, Debug, Clone, solidity::Codec)]
pub struct MintWitness {
	pub owned_item: OptionalU256,
	pub mint_price: OptionalU256,
}

impl<I, B> TryFrom<MintWitness> for pallet_nfts::MintWitness<I, B>
where
	I: TryFrom<U256>,
	B: TryFrom<U256>,
{
	type Error = Revert;
	fn try_from(value: MintWitness) -> MayRevert<Self> {
		Ok(Self {
			owned_item: value.owned_item.try_into_option()?,
			mint_price: value.mint_price.try_into_option()?,
		})
	}
}

/// Convenience type used to encode PriceDirection types.
#[derive(Default, Clone, Debug, Eq, PartialEq, IntoPrimitive, TryFromPrimitive)]
#[repr(u8)]
pub enum PriceDirection {
	#[default]
	Send,
	Receive,
}

impl Codec for PriceDirection {
	fn read(reader: &mut Reader) -> MayRevert<PriceDirection> {
		let value256: U256 =
			reader.read().map_err(|_| RevertReason::read_out_of_bounds(Self::signature()))?;

		let value_as_u8: u8 = value256
			.try_into()
			.map_err(|_| RevertReason::value_is_too_large(Self::signature()))?;

		value_as_u8
			.try_into()
			.map_err(|_| RevertReason::custom("Unknown direction type").into())
	}

	fn write(writer: &mut Writer, value: Self) {
		let value_as_u8: u8 = value.into();
		U256::write(writer, value_as_u8.into());
	}

	fn has_static_size() -> bool {
		true
	}

	fn signature() -> String {
		"uint8".into()
	}
}

impl From<PriceDirection> for pallet_nfts::PriceDirection {
	fn from(value: PriceDirection) -> Self {
		match value {
			PriceDirection::Send => Self::Send,
			PriceDirection::Receive => Self::Receive,
		}
	}
}

/// Helper struct used to encode PriceWithDirection Solidity types.
#[derive(Default, Debug, Clone, solidity::Codec)]
pub struct PriceWithDirection {
	amount: U256,
	direction: PriceDirection,
}

impl PriceWithDirection {
	pub fn new(amount: U256, direction: PriceDirection) -> Self {
		Self { amount, direction }
	}
}

impl<Amount> TryFrom<PriceWithDirection> for pallet_nfts::PriceWithDirection<Amount>
where
	Amount: TryFrom<U256>,
{
	type Error = Revert;

	fn try_from(value: PriceWithDirection) -> MayRevert<Self> {
		Ok(Self {
			amount: value
				.amount
				.try_into()
				.map_err(|_| RevertReason::value_is_too_large("Balance type"))?,
			direction: value.direction.into(),
		})
	}
}

/// Helper struct used to encode Optional address Solidity types.
#[derive(Default, Debug, Clone, solidity::Codec)]
pub struct OptionalAddress {
	pub has_value: bool,
	pub value: Address,
}

impl From<OptionalAddress> for Option<H160> {
	fn from(value: OptionalAddress) -> Self {
		match value.has_value {
			true => Some(value.value.into()),
			false => None,
		}
	}
}

impl OptionalAddress {
	pub fn into_option_lookup<T: pallet_evm::Config>(self) -> Option<AccountIdLookupOf<T>> {
		match self.has_value {
			true => Some({
				let value_h160: H160 = self.value.into();
				let value = T::AddressMapping::into_account_id(value_h160);
				T::Lookup::unlookup(value)
			}),
			false => None,
		}
	}
}

/// Helper struct used to encode ItemTip types.
#[derive(Default, Debug, Clone, solidity::Codec)]
pub struct ItemTip {
	pub collection_id: U256,
	pub item_id: U256,
	pub receiver: Address,
	pub amount: U256,
}

impl ItemTip {
	pub fn try_into_pallet_type<R>(self) -> MayRevert<ItemTipOf<R>>
	where
		R: pallet_evm::Config + pallet_nfts::Config,
		CollectionIdOf<R>: TryFrom<U256>,
		ItemIdOf<R>: TryFrom<U256>,
		BalanceOf<R>: TryFrom<U256>,
	{
		let account_h160: H160 = self.receiver.into();

		Ok(ItemTipOf::<R> {
			collection: self
				.collection_id
				.try_into()
				.map_err(|_| RevertReason::value_is_too_large("CollectionId type"))?,
			item: self
				.item_id
				.try_into()
				.map_err(|_| RevertReason::value_is_too_large("ItemId type"))?,
			receiver: R::AddressMapping::into_account_id(account_h160),
			amount: self
				.amount
				.try_into()
				.map_err(|_| RevertReason::value_is_too_large("Balance type"))?,
		})
	}
}

/// Convenience type used to encode MintType types.
#[derive(Default, Clone, Debug, Eq, PartialEq, IntoPrimitive, TryFromPrimitive)]
#[repr(u8)]
pub enum AttributeNamespace {
	/// An attribute was set by the pallet.
	Pallet,
	/// An attribute was set by collection's owner.
	#[default]
	CollectionOwner,
	/// An attribute was set by item's owner.
	ItemOwner,
	/// An attribute was set by pre-approved account.
	Account,
}

impl Codec for AttributeNamespace {
	fn read(reader: &mut Reader) -> MayRevert<AttributeNamespace> {
		let value256: U256 =
			reader.read().map_err(|_| RevertReason::read_out_of_bounds(Self::signature()))?;

		let value_as_u8: u8 = value256
			.try_into()
			.map_err(|_| RevertReason::value_is_too_large(Self::signature()))?;

		value_as_u8
			.try_into()
			.map_err(|_| RevertReason::custom("Unknown namespace type").into())
	}

	fn write(writer: &mut Writer, value: Self) {
		let value_as_u8: u8 = value.into();
		U256::write(writer, value_as_u8.into());
	}

	fn has_static_size() -> bool {
		true
	}

	fn signature() -> String {
		"uint8".into()
	}
}

/// Helper struct used to encode MintType types.
#[derive(Default, Debug, Clone, solidity::Codec)]
pub struct AttributeNamespaceInfo {
	pub namespace: AttributeNamespace,
	pub account: Address,
}

impl AttributeNamespaceInfo {
	pub fn into_pallet_type<R>(self) -> AttributeNamespaceOf<R>
	where
		R: pallet_evm::Config,
	{
		match self.namespace {
			AttributeNamespace::CollectionOwner => pallet_nfts::AttributeNamespace::CollectionOwner,
			AttributeNamespace::Pallet => pallet_nfts::AttributeNamespace::Pallet,
			AttributeNamespace::ItemOwner => pallet_nfts::AttributeNamespace::ItemOwner,
			AttributeNamespace::Account => pallet_nfts::AttributeNamespace::Account({
				let account_h160: H160 = self.account.into();
				R::AddressMapping::into_account_id(account_h160)
			}),
		}
	}
}

/// Helper struct used to encode CancelAttributesApprovalWitness types.
#[derive(Default, Debug, Clone, solidity::Codec)]
pub struct CancelAttributesApprovalWitness {
	account_attributes: U256,
}

impl TryFrom<CancelAttributesApprovalWitness> for pallet_nfts::CancelAttributesApprovalWitness {
	type Error = Revert;

	fn try_from(value: CancelAttributesApprovalWitness) -> MayRevert<Self> {
		Ok(Self {
			account_attributes: value.account_attributes.try_into().map_err(|_| {
				RevertReason::value_is_too_large("CancelAttributesApprovalWitness type")
			})?,
		})
	}
}

/// Helper struct used to encode CollectionDetails types.
#[derive(Default, Debug, Clone, solidity::Codec)]
pub struct CollectionDetails {
	/// Collection's owner.
	pub owner: Address,
	/// The total balance deposited by the owner for all the storage data associated with this
	/// collection. Used by `destroy`.
	pub owner_deposit: U256,
	/// The total number of outstanding items of this collection.
	pub items: u32,
	/// The total number of outstanding item metadata of this collection.
	pub item_metadatas: u32,
	/// The total number of outstanding item configs of this collection.
	pub item_configs: u32,
	/// The total number of attributes for this collection.
	pub attributes: u32,
}

impl<AccountId, Balance> From<pallet_nfts::CollectionDetails<AccountId, Balance>>
	for CollectionDetails
where
	AccountId: Clone + Into<H160>,
	Balance: Clone + Into<U256>,
{
	fn from(value: pallet_nfts::CollectionDetails<AccountId, Balance>) -> Self {
		Self {
			owner: {
				let owner_h160: H160 = value.clone().owner().into();
				owner_h160.into()
			},
			owner_deposit: value.clone().owner_deposit().into(),
			items: value.items(),
			item_metadatas: value.item_metadatas(),
			item_configs: value.item_configs(),
			attributes: value.attributes(),
		}
	}
}
