#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

mod types;

use crate::types::{
	CollectionConfig, CollectionSettings, MintWitness, OptionalMintWitness, OptionalU256,
};
use core::marker::PhantomData;
use fp_evm::ExitError;
use frame_support::{
	dispatch::{GetDispatchInfo, PostDispatchInfo},
	traits::{Currency, OriginTrait},
};
use frame_system::pallet_prelude::BlockNumberFor;
use pallet_evm::AddressMapping;
use precompile_utils::prelude::*;
use sp_core::{H160, U256};
use sp_runtime::traits::{Dispatchable, StaticLookup};

/// Alias for the Balance type for the provided Runtime.
pub type BalanceOf<Runtime> = <<Runtime as pallet_nfts::Config>::Currency as Currency<
	<Runtime as frame_system::Config>::AccountId,
>>::Balance;

/// Alias for the pallet nfts CollectionConfig type for the provided Runtime.
pub type CollectionConfigFor<Runtime> = pallet_nfts::CollectionConfig<
	BalanceOf<Runtime>,
	BlockNumberFor<Runtime>,
	CollectionIdOf<Runtime>,
>;

/// Alias for the pallet nfts MintWitness type for the provided Runtime.
pub type MintWitnessFor<Runtime> = pallet_nfts::MintWitness<ItemIdOf<Runtime>, BalanceOf<Runtime>>;

/// Alias for the pallet nfts MintSettings type for the provided Runtime.
pub type MintSettingsFor<Runtime> =
	pallet_nfts::MintSettings<BalanceOf<Runtime>, BlockNumberFor<Runtime>, CollectionIdOf<Runtime>>;

/// Alias for the Collection Id type for the provided Runtime.
pub type CollectionIdOf<Runtime> = <Runtime as pallet_nfts::Config>::CollectionId;

/// Alias for the Item Id type for the provided Runtime.
pub type ItemIdOf<Runtime> = <Runtime as pallet_nfts::Config>::ItemId;

/// Solidity selector of the Created log, which is the Keccak of the Log signature.
pub const SELECTOR_LOG_COLLECTION_CREATED: [u8; 32] =
	keccak256!("Collection_created(uint256,address,address)");

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
pub struct NftsPrecompileSet<Runtime>(PhantomData<Runtime>);

#[precompile_utils::precompile]
#[precompile::precompile_set]
impl<Runtime> NftsPrecompileSet<Runtime>
where
	Runtime: pallet_nfts::Config + pallet_evm::Config + frame_system::Config,
	Runtime::RuntimeCall: Dispatchable<PostInfo = PostDispatchInfo> + GetDispatchInfo,
	Runtime::RuntimeCall: From<pallet_nfts::Call<Runtime>>,
	Runtime: AddressToCollectionId<CollectionIdOf<Runtime>>,
	<Runtime::RuntimeCall as Dispatchable>::RuntimeOrigin: From<Option<Runtime::AccountId>>,
	BalanceOf<Runtime>: TryFrom<U256> + Into<U256> + solidity::Codec,
	<<Runtime as frame_system::Config>::RuntimeCall as Dispatchable>::RuntimeOrigin: OriginTrait,
	ItemIdOf<Runtime>: TryFrom<U256> + Into<U256> + Copy,
	CollectionConfigFor<Runtime>: TryFrom<CollectionConfig, Error = Revert>,
	MintWitnessFor<Runtime>: TryFrom<MintWitness, Error = Revert>,
	BlockNumberFor<Runtime>: TryFrom<U256>,
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

	// Extrinsics

	/* TODO move out (incompatible with discriminant)
	#[precompile::public(
	"create(address,((bool,bool,bool,bool,bool),(bool,uint256),((uint8,uint256),(bool,uint256),(bool,uint256),(bool,uint256),(bool,bool,bool))))"
	)]
	fn create(
		handle: &mut impl PrecompileHandle,
		admin: Address,
		config: CollectionConfig,
	) -> EvmResult<bool> {
		handle.record_log_costs_manual(3, 32)?;
		// pallet_nfts::NextCollectionId:
		// CollectionId(16)
		handle.record_db_read::<Runtime>(16)?;

		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);
		let admin: H160 = admin.into();
		// Retrieve next collection ID for emitting events.
		let next_id: U256 = pallet_nfts::NextCollectionId::<Runtime>::get()
			.or(CollectionIdOf::<Runtime>::initial_value())
			.ok_or(RevertReason::custom("cannot fetch next collection id"))?
			.into();

		{
			let admin: Runtime::AccountId = Runtime::AddressMapping::into_account_id(admin);
			let config: CollectionConfigFor<Runtime> = config.try_into()?;
			let create_call = pallet_nfts::Call::<Runtime>::create {
				admin: Runtime::Lookup::unlookup(admin),
				config,
			};

			RuntimeHelper::<Runtime>::try_dispatch(handle, Some(origin).into(), create_call)?;
		}
		log3(
			handle.context().address,
			SELECTOR_LOG_COLLECTION_CREATED,
			handle.context().caller,
			admin,
			solidity::encode_event_data(next_id),
		)
		.record(handle)?;

		Ok(true)
	}*/

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
				owner: Runtime::Lookup::unlookup(owner),
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

	fn u256_to_item_id(value: U256) -> MayRevert<ItemIdOf<Runtime>> {
		value
			.try_into()
			.map_err(|_| RevertReason::value_is_too_large("ItemId type").into())
	}
}
