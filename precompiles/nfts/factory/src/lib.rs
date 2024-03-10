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
use pallet_evm_precompile_nfts_types::solidity::CollectionConfig;
use precompile_utils::prelude::*;
use sp_core::{H160, U256};
use sp_runtime::traits::{Dispatchable, StaticLookup};

/// Alias for the Balance type for the provided Runtime.
type BalanceOf<Runtime> = <<Runtime as pallet_nfts::Config>::Currency as Currency<
	<Runtime as frame_system::Config>::AccountId,
>>::Balance;

/// Alias for the pallet nfts CollectionConfig type for the provided Runtime.
type CollectionConfigFor<Runtime> = pallet_nfts::CollectionConfig<
	BalanceOf<Runtime>,
	BlockNumberFor<Runtime>,
	CollectionIdOf<Runtime>,
>;

/// Alias for the Collection Id type for the provided Runtime.
type CollectionIdOf<Runtime> = <Runtime as pallet_nfts::Config>::CollectionId;

/// Alias for the Item Id type for the provided Runtime.
type ItemIdOf<Runtime> = <Runtime as pallet_nfts::Config>::ItemId;

pub struct NftsFactoryPrecompile<Runtime>(PhantomData<Runtime>);

#[precompile_utils::precompile]
#[precompile::test_concrete_types(mock::Runtime)]
impl<Runtime> NftsFactoryPrecompile<Runtime>
where
	Runtime: pallet_nfts::Config + pallet_evm::Config + frame_system::Config,
	Runtime::RuntimeCall: Dispatchable<PostInfo = PostDispatchInfo> + GetDispatchInfo,
	Runtime::RuntimeCall: From<pallet_nfts::Call<Runtime>>,
	<Runtime::RuntimeCall as Dispatchable>::RuntimeOrigin: From<Option<Runtime::AccountId>>,
	<<Runtime as frame_system::Config>::RuntimeCall as Dispatchable>::RuntimeOrigin: OriginTrait,
	ItemIdOf<Runtime>: TryFrom<U256> + Into<U256> + Copy,
	CollectionIdOf<Runtime>: TryFrom<U256> + Into<U256> + Copy,
	CollectionConfigFor<Runtime>: TryFrom<CollectionConfig, Error = Revert>,
{
	// Extrinsics

	#[precompile::public(
	"create(address,((bool,bool,bool,bool,bool),(bool,uint256),((uint8,uint256),(bool,uint256),(bool,uint256),(bool,uint256),(bool,bool,bool))))"
	)]
	fn create(
		handle: &mut impl PrecompileHandle,
		admin: Address,
		config: CollectionConfig,
	) -> EvmResult<bool> {
		handle.record_log_costs_manual(3, 32)?;

		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);
		let admin: H160 = admin.into();

		{
			let admin: Runtime::AccountId = Runtime::AddressMapping::into_account_id(admin);
			let config: CollectionConfigFor<Runtime> = config.try_into()?;
			let create_call = pallet_nfts::Call::<Runtime>::create {
				admin: Runtime::Lookup::unlookup(admin),
				config,
			};

			RuntimeHelper::<Runtime>::try_dispatch(handle, Some(origin).into(), create_call)?;
		}

		Ok(true)
	}

	#[precompile::public("set_accept_ownership(uint256)")]
	fn set_accept_ownership(
		handle: &mut impl PrecompileHandle,
		collection: U256,
	) -> EvmResult<bool> {
		handle.record_log_costs_manual(3, 32)?;

		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);

		{
			let collection: CollectionIdOf<Runtime> = Self::u256_to_collection_id(collection)?;
			let set_accept_ownership_call = pallet_nfts::Call::<Runtime>::set_accept_ownership {
				maybe_collection: Some(collection),
			};
			RuntimeHelper::<Runtime>::try_dispatch(
				handle,
				Some(origin).into(),
				set_accept_ownership_call,
			)?;
		}

		Ok(true)
	}

	fn u256_to_collection_id(value: U256) -> MayRevert<CollectionIdOf<Runtime>> {
		value
			.try_into()
			.map_err(|_| RevertReason::value_is_too_large("CollectionId type").into())
	}
}
