use num_enum::{IntoPrimitive, TryFromPrimitive};
use precompile_utils::{
	prelude::{MayRevert, Revert, RevertReason},
	solidity,
	solidity::{
		codec::{Reader, Writer},
		Codec,
	},
};
use sp_core::U256;

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

/// Helper struct used to encode CollectionConfig types.
#[derive(Default, Debug, Clone, solidity::Codec)]
pub struct CollectionConfig {
	pub(crate) settings: CollectionSettings,
	pub(crate) max_supply: OptionalU256,
	pub(crate) mint_settings: MintSettings,
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
	pub(crate) fn all_enabled() -> Self {
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
			s.0.set(pallet_nfts::CollectionSetting::TransferableItems, false)
		}
		if !self.is_unlocked_metadata {
			s.0.set(pallet_nfts::CollectionSetting::UnlockedMetadata, false)
		}
		if !self.is_unlocked_attributes {
			s.0.set(pallet_nfts::CollectionSetting::UnlockedAttributes, false)
		}
		if !self.is_unlocked_max_supply {
			s.0.set(pallet_nfts::CollectionSetting::UnlockedMaxSupply, false)
		}
		if !self.is_deposit_required {
			s.0.set(pallet_nfts::CollectionSetting::DepositRequired, false)
		}

		s
	}
}

/// Helper struct used to encode ItemSettings types.
#[derive(Default, Debug, Clone, solidity::Codec)]
pub(crate) struct ItemSettings {
	pub is_transferable: bool,
	pub is_unlocked_metadata: bool,
	pub is_unlocked_attributes: bool,
}

impl ItemSettings {
	fn all_enabled() -> Self {
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
pub(crate) struct MintSettings {
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
#[derive(Clone, Debug, Eq, PartialEq, IntoPrimitive, TryFromPrimitive)]
#[repr(u8)]
pub(crate) enum MintType {
	Issuer,
	Public,
	HolderOf,
}

impl Default for MintType {
	fn default() -> Self {
		MintType::Issuer
	}
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
pub(crate) struct MintInfo {
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
