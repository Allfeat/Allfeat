#![cfg_attr(not(feature = "std"), no_std)]

use fp_evm::PrecompileHandle;
use frame_support::dispatch::{GetDispatchInfo, PostDispatchInfo};
use frame_support::{
	storage::types::{StorageDoubleMap, StorageMap, ValueQuery},
	traits::StorageInstance,
	Blake2_128Concat,
};
use pallet_evm::AddressMapping;
use precompile_utils::prelude::*;
use sp_core::{H160, H256, U256};
use sp_runtime::traits::{Bounded, CheckedSub, Dispatchable, StaticLookup};
use sp_std::{
	convert::{TryFrom, TryInto},
	marker::PhantomData,
};

mod eip2612;
use eip2612::Eip2612;

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

/// Solidity selector of the Transfer log, which is the Keccak of the Log signature.
pub const SELECTOR_LOG_TRANSFER: [u8; 32] = keccak256!("Transfer(address,address,uint256)");

/// Solidity selector of the Approval log, which is the Keccak of the Log signature.
pub const SELECTOR_LOG_APPROVAL: [u8; 32] = keccak256!("Approval(address,address,uint256)");

/// Solidity selector of the Deposit log, which is the Keccak of the Log signature.
pub const SELECTOR_LOG_DEPOSIT: [u8; 32] = keccak256!("Deposit(address,uint256)");

/// Solidity selector of the Withdraw log, which is the Keccak of the Log signature.
pub const SELECTOR_LOG_WITHDRAWAL: [u8; 32] = keccak256!("Withdrawal(address,uint256)");

/// Alias for the Balance type for the provided Runtime and Instance.
pub type BalanceOf<Runtime> = <Runtime as pallet_balances::Config>::Balance;

pub struct Approves;
impl StorageInstance for Approves {
	const STORAGE_PREFIX: &'static str = "Approves";

	fn pallet_prefix() -> &'static str {
		"Erc20Instance0Balances"
	}
}
/// Storage type used to store approvals, since `pallet_balances` doesn't
/// handle this behavior.
/// (Owner => Allowed => Amount)
pub type ApprovesStorage<Runtime> = StorageDoubleMap<
	Approves,
	Blake2_128Concat,
	<Runtime as frame_system::Config>::AccountId,
	Blake2_128Concat,
	<Runtime as frame_system::Config>::AccountId,
	BalanceOf<Runtime>,
>;

pub struct Nonces;
impl StorageInstance for Nonces {
	const STORAGE_PREFIX: &'static str = "Nonces";

	fn pallet_prefix() -> &'static str {
		"Erc20Instance0Balances"
	}
}
/// Storage type used to store EIP2612 nonces.
pub type NoncesStorage = StorageMap<
	Nonces,
	// Owner
	Blake2_128Concat,
	H160,
	// Nonce
	U256,
	ValueQuery,
>;

/// Metadata of an ERC20 token.
pub trait Erc20Metadata {
	/// Returns the name of the token.
	fn name() -> &'static str;

	/// Returns the symbol of the token.
	fn symbol() -> &'static str;

	/// Returns the decimals places of the token.
	fn decimals() -> u8;

	/// Must return `true` only if it represents the main native currency of
	/// the network. It must be the currency used in `pallet_evm`.
	fn is_native_currency() -> bool;
}

/// Precompile exposing a pallet_balance as an ERC20.
/// Multiple precompiles can support instances of pallet_balance.
/// The precompile uses an additional storage to store approvals.
pub struct Erc20BalancesPrecompile<Runtime, Metadata: Erc20Metadata>(
	PhantomData<(Runtime, Metadata)>,
);

#[precompile_utils::precompile]
impl<Runtime, Metadata> Erc20BalancesPrecompile<Runtime, Metadata>
where
	Metadata: Erc20Metadata,
	Runtime: pallet_balances::Config + pallet_evm::Config + pallet_timestamp::Config,
	Runtime::RuntimeCall: Dispatchable<PostInfo = PostDispatchInfo> + GetDispatchInfo,
	Runtime::RuntimeCall: From<pallet_balances::Call<Runtime>>,
	<Runtime::RuntimeCall as Dispatchable>::RuntimeOrigin: From<Option<Runtime::AccountId>>,
	BalanceOf<Runtime>: TryFrom<U256> + Into<U256>,
	<Runtime as pallet_timestamp::Config>::Moment: Into<U256>,
{
	#[precompile::public("totalSupply()")]
	#[precompile::view]
	fn total_supply(handle: &mut impl PrecompileHandle) -> EvmResult<U256> {
		// TotalIssuance: Balance(16)
		handle.record_db_read::<Runtime>(16)?;

		Ok(pallet_balances::Pallet::<Runtime>::total_issuance().into())
	}

	#[precompile::public("balanceOf(address)")]
	#[precompile::view]
	fn balance_of(handle: &mut impl PrecompileHandle, owner: Address) -> EvmResult<U256> {
		// frame_system::Account:
		// Blake2128(16) + AccountId(20) + AccountInfo ((4 * 4) + AccountData(16 * 4))
		handle.record_db_read::<Runtime>(116)?;

		let owner: H160 = owner.into();
		let owner: Runtime::AccountId = Runtime::AddressMapping::into_account_id(owner);

		Ok(pallet_balances::Pallet::<Runtime>::usable_balance(&owner).into())
	}

	#[precompile::public("allowance(address,address)")]
	#[precompile::view]
	fn allowance(
		handle: &mut impl PrecompileHandle,
		owner: Address,
		spender: Address,
	) -> EvmResult<U256> {
		// frame_system::ApprovesStorage:
		// (2 * (Blake2128(16) + AccountId(20)) + Balanceof(16)
		handle.record_db_read::<Runtime>(88)?;

		let owner: H160 = owner.into();
		let spender: H160 = spender.into();

		let owner: Runtime::AccountId = Runtime::AddressMapping::into_account_id(owner);
		let spender: Runtime::AccountId = Runtime::AddressMapping::into_account_id(spender);

		Ok(ApprovesStorage::<Runtime>::get(owner, spender).unwrap_or_default().into())
	}

	#[precompile::public("approve(address,uint256)")]
	fn approve(
		handle: &mut impl PrecompileHandle,
		spender: Address,
		value: U256,
	) -> EvmResult<bool> {
		handle.record_cost(RuntimeHelper::<Runtime>::db_write_gas_cost())?;
		handle.record_log_costs_manual(3, 32)?;

		let spender: H160 = spender.into();

		// Write into storage.
		{
			let caller: Runtime::AccountId =
				Runtime::AddressMapping::into_account_id(handle.context().caller);
			let spender: Runtime::AccountId = Runtime::AddressMapping::into_account_id(spender);
			// Amount saturate if too high.
			let value = Self::u256_to_amount(value).unwrap_or_else(|_| Bounded::max_value());

			ApprovesStorage::<Runtime>::insert(caller, spender, value);
		}

		log3(
			handle.context().address,
			SELECTOR_LOG_APPROVAL,
			handle.context().caller,
			spender,
			solidity::encode_event_data(value),
		)
		.record(handle)?;

		// Build output.
		Ok(true)
	}

	#[precompile::public("transfer(address,uint256)")]
	fn transfer(handle: &mut impl PrecompileHandle, to: Address, value: U256) -> EvmResult<bool> {
		handle.record_log_costs_manual(3, 32)?;

		let to: H160 = to.into();

		// Build call with origin.
		{
			let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);
			let to = Runtime::AddressMapping::into_account_id(to);
			let value = Self::u256_to_amount(value).in_field("value")?;

			// Dispatch call (if enough gas).
			RuntimeHelper::<Runtime>::try_dispatch(
				handle,
				Some(origin).into(),
				pallet_balances::Call::<Runtime>::transfer {
					dest: Runtime::Lookup::unlookup(to),
					value,
				},
			)?;
		}

		log3(
			handle.context().address,
			SELECTOR_LOG_TRANSFER,
			handle.context().caller,
			to,
			solidity::encode_event_data(value),
		)
		.record(handle)?;

		Ok(true)
	}

	#[precompile::public("transferFrom(address,address,uint256)")]
	fn transfer_from(
		handle: &mut impl PrecompileHandle,
		from: Address,
		to: Address,
		value: U256,
	) -> EvmResult<bool> {
		// frame_system::ApprovesStorage:
		// (2 * (Blake2128(16) + AccountId(20)) + Balanceof(16)
		handle.record_db_read::<Runtime>(88)?;
		handle.record_cost(RuntimeHelper::<Runtime>::db_write_gas_cost())?;
		handle.record_log_costs_manual(3, 32)?;

		let from: H160 = from.into();
		let to: H160 = to.into();

		{
			let caller: Runtime::AccountId =
				Runtime::AddressMapping::into_account_id(handle.context().caller);
			let from: Runtime::AccountId = Runtime::AddressMapping::into_account_id(from);
			let to: Runtime::AccountId = Runtime::AddressMapping::into_account_id(to);
			let value = Self::u256_to_amount(value).in_field("value")?;

			// If caller is "from", it can spend as much as it wants.
			if caller != from {
				ApprovesStorage::<Runtime>::mutate(from.clone(), caller, |entry| {
					// Get current allowed value, exit if None.
					let allowed = entry.ok_or(revert("spender not allowed"))?;

					// Remove "value" from allowed, exit if underflow.
					let allowed = allowed
						.checked_sub(&value)
						.ok_or_else(|| revert("trying to spend more than allowed"))?;

					// Update allowed value.
					*entry = Some(allowed);

					EvmResult::Ok(())
				})?;
			}

			// Build call with origin. Here origin is the "from"/owner field.
			// Dispatch call (if enough gas).
			RuntimeHelper::<Runtime>::try_dispatch(
				handle,
				Some(from).into(),
				pallet_balances::Call::<Runtime>::transfer {
					dest: Runtime::Lookup::unlookup(to),
					value,
				},
			)?;
		}

		log3(
			handle.context().address,
			SELECTOR_LOG_TRANSFER,
			from,
			to,
			solidity::encode_event_data(value),
		)
		.record(handle)?;

		Ok(true)
	}

	#[precompile::public("name()")]
	#[precompile::view]
	fn name(_handle: &mut impl PrecompileHandle) -> EvmResult<UnboundedBytes> {
		Ok(Metadata::name().into())
	}

	#[precompile::public("symbol()")]
	#[precompile::view]
	fn symbol(_handle: &mut impl PrecompileHandle) -> EvmResult<UnboundedBytes> {
		Ok(Metadata::symbol().into())
	}

	#[precompile::public("decimals()")]
	#[precompile::view]
	fn decimals(_handle: &mut impl PrecompileHandle) -> EvmResult<u8> {
		Ok(Metadata::decimals())
	}

	#[precompile::public("deposit()")]
	#[precompile::fallback]
	#[precompile::payable]
	fn deposit(handle: &mut impl PrecompileHandle) -> EvmResult {
		// Deposit only makes sense for the native currency.
		if !Metadata::is_native_currency() {
			return Err(RevertReason::UnknownSelector.into());
		}

		let caller: Runtime::AccountId =
			Runtime::AddressMapping::into_account_id(handle.context().caller);
		let precompile = Runtime::AddressMapping::into_account_id(handle.context().address);
		let amount = Self::u256_to_amount(handle.context().apparent_value)?;

		if amount.into() == U256::from(0u32) {
			return Err(revert("deposited amount must be non-zero"));
		}

		handle.record_log_costs_manual(2, 32)?;

		// Send back funds received by the precompile.
		RuntimeHelper::<Runtime>::try_dispatch(
			handle,
			Some(precompile).into(),
			pallet_balances::Call::<Runtime>::transfer {
				dest: Runtime::Lookup::unlookup(caller),
				value: amount,
			},
		)?;

		log2(
			handle.context().address,
			SELECTOR_LOG_DEPOSIT,
			handle.context().caller,
			solidity::encode_event_data(handle.context().apparent_value),
		)
		.record(handle)?;

		Ok(())
	}

	#[precompile::public("withdraw(uint256)")]
	fn withdraw(handle: &mut impl PrecompileHandle, value: U256) -> EvmResult {
		// Withdraw only makes sense for the native currency.
		if !Metadata::is_native_currency() {
			return Err(RevertReason::UnknownSelector.into());
		}

		handle.record_log_costs_manual(2, 32)?;

		let account_amount: U256 = {
			let owner: Runtime::AccountId =
				Runtime::AddressMapping::into_account_id(handle.context().caller);
			pallet_balances::Pallet::<Runtime>::usable_balance(&owner).into()
		};

		if value > account_amount {
			return Err(revert("Trying to withdraw more than owned"));
		}

		log2(
			handle.context().address,
			SELECTOR_LOG_WITHDRAWAL,
			handle.context().caller,
			solidity::encode_event_data(value),
		)
		.record(handle)?;

		Ok(())
	}

	#[precompile::public("permit(address,address,uint256,uint256,uint8,bytes32,bytes32)")]
	fn eip2612_permit(
		handle: &mut impl PrecompileHandle,
		owner: Address,
		spender: Address,
		value: U256,
		deadline: U256,
		v: u8,
		r: H256,
		s: H256,
	) -> EvmResult {
		<Eip2612<Runtime, Metadata>>::permit(handle, owner, spender, value, deadline, v, r, s)
	}

	#[precompile::public("nonces(address)")]
	#[precompile::view]
	fn eip2612_nonces(handle: &mut impl PrecompileHandle, owner: Address) -> EvmResult<U256> {
		<Eip2612<Runtime, Metadata>>::nonces(handle, owner)
	}

	#[precompile::public("DOMAIN_SEPARATOR()")]
	#[precompile::view]
	fn eip2612_domain_separator(handle: &mut impl PrecompileHandle) -> EvmResult<H256> {
		<Eip2612<Runtime, Metadata>>::domain_separator(handle)
	}

	fn u256_to_amount(value: U256) -> MayRevert<BalanceOf<Runtime>> {
		value
			.try_into()
			.map_err(|_| RevertReason::value_is_too_large("balance type").into())
	}
}
