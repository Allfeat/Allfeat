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

//! Common runtime code for Allfeat.

#![cfg_attr(not(feature = "std"), no_std)]

use allfeat_primitives::{Balance, BlockNumber};
use frame_support::{
	traits::Get,
	weights::{
		constants::ExtrinsicBaseWeight, Weight, WeightToFeeCoefficient, WeightToFeeCoefficients,
		WeightToFeePolynomial,
	},
};
use frame_system::limits::BlockLength;
use pallet_evm::FeeCalculator;
use pallet_transaction_payment::{Multiplier, OnChargeTransaction, TargetedFeeAdjustment};
use sp_core::{parameter_types, U256};
use sp_runtime::{traits::Bounded, FixedPointNumber, Perbill, Perquintill};

pub mod elections;
pub mod identity;

pub mod currency;
pub mod opaque;

#[cfg(feature = "test")]
pub mod test;
/// Custom weights for Allfeat runtimes
pub mod weights;

parameter_types! {
	pub const BlockHashCount: BlockNumber = 4096;
	/// The portion of the `NORMAL_DISPATCH_RATIO` that we adjust the fees with. Blocks filled less
	/// than this will decrease the weight and more will increase.
	pub const TargetBlockFullness: Perquintill = Perquintill::from_percent(25);
	/// The adjustment variable of the runtime. Higher values will cause `TargetBlockFullness` to
	/// change the fees more rapidly.
	pub AdjustmentVariable: Multiplier = Multiplier::saturating_from_rational(75, 1_000_000);
	/// Minimum amount of the multiplier. This value cannot be too low. A test case should ensure
	/// that combined with `AdjustmentVariable`, we can recover from the minimum.
	/// See `multiplier_can_grow_from_zero`.
	pub MinimumMultiplier: Multiplier = Multiplier::saturating_from_rational(1, 10u128);
	/// The maximum amount of the multiplier.
	pub MaximumMultiplier: Multiplier = Bounded::max_value();
	/// Maximum length of block. Up to 5MB.
	pub RuntimeBlockLength: BlockLength =
		BlockLength::max_with_normal_ratio(5 * 1024 * 1024, NORMAL_DISPATCH_RATIO);
}

/// Parameterized slow adjusting fee updated based on
/// <https://research.web3.foundation/Polkadot/overview/token-economics#2-slow-adjusting-mechanism>
pub type SlowAdjustingFeeUpdate<R> = TargetedFeeAdjustment<
	R,
	TargetBlockFullness,
	AdjustmentVariable,
	MinimumMultiplier,
	MaximumMultiplier,
>;

/// Handles converting a weight scalar to a fee value, based on the scale and granularity of the
/// node's balance type.
///
/// This should typically create a mapping between the following ranges:
///   - [0, MAXIMUM_BLOCK_WEIGHT]
///   - [Balance::min, Balance::max]
///
/// Yet, it can be used for any other sort of change to weight-fee. Some examples being:
///   - Setting it to `0` will essentially disable the weight fee.
///   - Setting it to `1` will cause the literal `#[weight = x]` values to be charged.
pub struct WeightToFee;
impl WeightToFeePolynomial for WeightToFee {
	type Balance = Balance;
	fn polynomial() -> WeightToFeeCoefficients<Self::Balance> {
		let p = 15_000_000_000_000_000; // Around 0.015 AFT
		let q = Balance::from(ExtrinsicBaseWeight::get().ref_time());
		smallvec::smallvec![WeightToFeeCoefficient {
			degree: 1,
			negative: false,
			coeff_frac: Perbill::from_rational(p % q, q),
			coeff_integer: p / q,
		}]
	}
}

/// We assume that an on-initialize consumes 1% of the weight on average, hence a single extrinsic
/// will not be allowed to consume more than `AvailableBlockRatio - 1%`.
pub const AVERAGE_ON_INITIALIZE_RATIO: Perbill = Perbill::from_percent(1);
/// We allow `Normal` extrinsics to fill up the block up to 75%, the rest can be used
/// by  Operational  extrinsics.
pub const NORMAL_DISPATCH_RATIO: Perbill = Perbill::from_percent(75);

/// Convert a balance to an unsigned 256-bit number, use in nomination pools.
pub struct BalanceToU256;
impl sp_runtime::traits::Convert<Balance, sp_core::U256> for BalanceToU256 {
	fn convert(n: Balance) -> sp_core::U256 {
		n.into()
	}
}

/// Convert an unsigned 256-bit number to balance, use in nomination pools.
pub struct U256ToBalance;
impl sp_runtime::traits::Convert<sp_core::U256, Balance> for U256ToBalance {
	fn convert(n: sp_core::U256) -> Balance {
		use frame_support::traits::Defensive;
		n.try_into().defensive_unwrap_or(Balance::MAX)
	}
}

pub struct TransactionPaymentGasPrice<R, WeightPerGas>(
	core::marker::PhantomData<R>,
	core::marker::PhantomData<WeightPerGas>,
);
impl<R, WeightPerGas> FeeCalculator for TransactionPaymentGasPrice<R, WeightPerGas>
where
	R: pallet_transaction_payment::Config + frame_system::Config,
	WeightPerGas: Get<Weight>,
	<<R as pallet_transaction_payment::Config>::OnChargeTransaction as OnChargeTransaction<R>>::Balance: Into<Balance>
{
	fn min_gas_price() -> (U256, Weight) {
		// substrate
		use frame_support::weights::WeightToFee;
		(
			pallet_transaction_payment::Pallet::<R>::next_fee_multiplier()
				.saturating_mul_int::<Balance>(
					<R as pallet_transaction_payment::Config>::WeightToFee::weight_to_fee(
						&WeightPerGas::get(),
					).into(),
				)
				.into(),
			<R as frame_system::Config>::DbWeight::get().reads(1),
		)
	}
}

/* #[derive(Clone)]
pub struct TransactionConverter<B, R>(PhantomData<B>, PhantomData<R>);

impl<B, R> Default for TransactionConverter<B, R> {
	fn default() -> Self {
		Self(PhantomData, PhantomData)
	}
}

impl<B, R> fp_rpc::ConvertTransaction<<B as BlockT>::Extrinsic> for TransactionConverter<B, R>
where
	B: BlockT,
	R: frame_system::Config + pallet_ethereum::Config,
	Result<pallet_ethereum::RawOrigin, <R as frame_system::Config>::RuntimeOrigin>: From<<R as frame_system::Config>::RuntimeOrigin>,
{
	fn convert_transaction(
		&self,
		transaction: pallet_ethereum::Transaction,
	) -> <B as BlockT>::Extrinsic {
		let extrinsic = UncheckedExtrinsic::new_unsigned(
			pallet_ethereum::Call::<R>::transact { transaction }.into(),
		);
		let encoded = extrinsic.encode();
		<B as BlockT>::Extrinsic::decode(&mut &encoded[..])
			.expect("Encoded extrinsic is always valid")
	}
} */

/// Macro to set a value (e.g. when using the `parameter_types` macro) to either a production value
/// or to an environment variable or testing value (in case the `fast-runtime` feature is selected)
/// or one of two testing values depending on feature.
/// Note that the environment variable is evaluated _at compile time_.
///
/// Usage:
/// ```Rust
/// parameter_types! {
///     // Note that the env variable version parameter cannot be const.
///     pub LaunchPeriod: BlockNumber = prod_or_fast!(7 * DAYS, 1, "KSM_LAUNCH_PERIOD");
///     pub const VotingPeriod: BlockNumber = prod_or_fast!(7 * DAYS, 1 * MINUTES);
///     pub const EpochDuration: BlockNumber =
///         prod_or_fast!(1 * HOURS, "fast-runtime", 1 * MINUTES, "fast-runtime-10m", 10 * MINUTES);
/// }
/// ```
#[macro_export]
macro_rules! prod_or_fast {
	($prod:expr, $test:expr) => {
		if cfg!(feature = "fast-runtime") {
			$test
		} else {
			$prod
		}
	};
	($prod:expr, $test:expr, $env:expr) => {
		if cfg!(feature = "fast-runtime") {
			core::option_env!($env).map(|s| s.parse().ok()).flatten().unwrap_or($test)
		} else {
			$prod
		}
	};
}

#[macro_export]
macro_rules! impl_self_contained_call {
	() => {
		impl fp_self_contained::SelfContainedCall for RuntimeCall {
			type SignedInfo = sp_core::H160;

			fn is_self_contained(&self) -> bool {
				match self {
					RuntimeCall::Ethereum(call) => call.is_self_contained(),
					_ => false,
				}
			}

			fn check_self_contained(
				&self,
			) -> Option<
				Result<
					Self::SignedInfo,
					sp_runtime::transaction_validity::TransactionValidityError,
				>,
			> {
				match self {
					RuntimeCall::Ethereum(call) => call.check_self_contained(),
					_ => None,
				}
			}

			fn validate_self_contained(
				&self,
				info: &Self::SignedInfo,
				dispatch_info: &sp_runtime::traits::DispatchInfoOf<RuntimeCall>,
				len: usize,
			) -> Option<sp_runtime::transaction_validity::TransactionValidity> {
				match self {
					RuntimeCall::Ethereum(call) =>
						call.validate_self_contained(info, dispatch_info, len),
					_ => None,
				}
			}

			fn pre_dispatch_self_contained(
				&self,
				info: &Self::SignedInfo,
				dispatch_info: &sp_runtime::traits::DispatchInfoOf<RuntimeCall>,
				len: usize,
			) -> Option<Result<(), sp_runtime::transaction_validity::TransactionValidityError>> {
				match self {
					RuntimeCall::Ethereum(call) =>
						call.pre_dispatch_self_contained(info, dispatch_info, len),
					_ => None,
				}
			}

			fn apply_self_contained(
				self,
				info: Self::SignedInfo,
			) -> Option<
				sp_runtime::DispatchResultWithInfo<sp_runtime::traits::PostDispatchInfoOf<Self>>,
			> {
				// substrate
				use sp_runtime::traits::Dispatchable;

				match self {
					call @ RuntimeCall::Ethereum(pallet_ethereum::Call::transact { .. }) =>
						Some(call.dispatch(RuntimeOrigin::from(
							pallet_ethereum::RawOrigin::EthereumTransaction(info),
						))),
					_ => None,
				}
			}
		}
	};
}

#[macro_export]
macro_rules! impl_create_signed_transaction {
	() => {
		use sp_runtime::{traits::StaticLookup, SaturatedConversion};

		impl<LocalCall> frame_system::offchain::CreateSignedTransaction<LocalCall> for Runtime
		where
			RuntimeCall: From<LocalCall>,
		{
			fn create_transaction<
				C: frame_system::offchain::AppCrypto<Self::Public, Self::Signature>,
			>(
				call: RuntimeCall,
				public: <allfeat_primitives::Signature as sp_runtime::traits::Verify>::Signer,
				account: allfeat_primitives::AccountId,
				nonce: allfeat_primitives::Nonce,
			) -> Option<(
				RuntimeCall,
				<UncheckedExtrinsic as sp_runtime::traits::Extrinsic>::SignaturePayload,
			)> {
				let tip = 0;
				// take the biggest period possible.
				let period = shared_runtime::BlockHashCount::get()
					.checked_next_power_of_two()
					.map(|c| c / 2)
					.unwrap_or(2) as u64;
				let current_block = System::block_number()
					.saturated_into::<u64>()
					// The `System::block_number` is initialized with `n+1`,
					// so the actual block number is `n`.
					.saturating_sub(1);
				let era = sp_runtime::generic::Era::mortal(period, current_block);
				let extra = (
					frame_system::CheckNonZeroSender::<Runtime>::new(),
					frame_system::CheckSpecVersion::<Runtime>::new(),
					frame_system::CheckTxVersion::<Runtime>::new(),
					frame_system::CheckGenesis::<Runtime>::new(),
					frame_system::CheckEra::<Runtime>::from(era),
					frame_system::CheckNonce::<Runtime>::from(nonce),
					frame_system::CheckWeight::<Runtime>::new(),
					pallet_transaction_payment::ChargeTransactionPayment::<Runtime>::from(tip),
					frame_metadata_hash_extension::CheckMetadataHash::<Runtime>::new(false),
				);
				let raw_payload = SignedPayload::new(call, extra)
					.map_err(|e| {
						log::warn!("Unable to create signed payload: {:?}", e);
					})
					.ok()?;
				let signature = raw_payload.using_encoded(|payload| C::sign(payload, public))?;
				let address = Self::Lookup::unlookup(account);
				let (call, extra, _) = raw_payload.deconstruct();
				Some((call, (address, signature, extra)))
			}
		}

		impl frame_system::offchain::SigningTypes for Runtime {
			type Public = <allfeat_primitives::Signature as sp_runtime::traits::Verify>::Signer;
			type Signature = Signature;
		}

		impl<C> frame_system::offchain::SendTransactionTypes<C> for Runtime
		where
			RuntimeCall: From<C>,
		{
			type Extrinsic = UncheckedExtrinsic;
			type OverarchingCall = RuntimeCall;
		}
	};
}
