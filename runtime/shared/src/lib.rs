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
// TODO: check for use #![deny(unused_crate_dependencies)]

use crate::currency::AFT;
use allfeat_primitives::{Balance, BlockNumber};
use frame_support::{
	traits::Get,
	weights::{
		constants::ExtrinsicBaseWeight, FeePolynomial, Weight, WeightToFee as WeightToFeeT,
		WeightToFeeCoefficient, WeightToFeeCoefficients, WeightToFeePolynomial,
	},
};
use frame_system::limits::BlockLength;
use pallet_evm::FeeCalculator;
use pallet_transaction_payment::{Multiplier, OnChargeTransaction, TargetedFeeAdjustment};
use sp_core::{parameter_types, U256};
use sp_runtime::{traits::Bounded, FixedPointNumber, Perbill, Perquintill};

pub mod elections;
pub mod identity;

mod currency;

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
///   - `[0, MAXIMUM_BLOCK_WEIGHT]`
///   - `[Balance::min, Balance::max]`
///
/// Yet, it can be used for any other sort of change to weight-fee. Some examples being:
///   - Setting it to `0` will essentially disable the weight fee.
///   - Setting it to `1` will cause the literal `#[weight = x]` values to be charged.
pub struct WeightToFee;
impl WeightToFeeT for WeightToFee {
	type Balance = Balance;

	fn weight_to_fee(weight: &Weight) -> Self::Balance {
		let time_poly: FeePolynomial<Balance> = RefTimeToFee::polynomial().into();
		let proof_poly: FeePolynomial<Balance> = ProofSizeToFee::polynomial().into();

		// Take the maximum instead of the sum to charge by the more scarce resource.
		time_poly.eval(weight.ref_time()).max(proof_poly.eval(weight.proof_size()))
	}
}

/// Maps the reference time component of `Weight` to a fee.
pub struct RefTimeToFee;
impl WeightToFeePolynomial for RefTimeToFee {
	type Balance = Balance;

	fn polynomial() -> WeightToFeeCoefficients<Self::Balance> {
		// Map base extrinsic weight to 1/500 AFT.
		let p = AFT;
		let q = 500 * Balance::from(ExtrinsicBaseWeight::get().ref_time());

		smallvec::smallvec![WeightToFeeCoefficient {
			degree: 1,
			negative: false,
			coeff_frac: Perbill::from_rational(p % q, q),
			coeff_integer: p / q,
		}]
	}
}

/// Maps the proof size component of `Weight` to a fee.
pub struct ProofSizeToFee;
impl WeightToFeePolynomial for ProofSizeToFee {
	type Balance = Balance;

	fn polynomial() -> WeightToFeeCoefficients<Self::Balance> {
		// Map 10kb proof to 1 AFT.
		let p = AFT;
		let q = 10_000;

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

/// Macro to set a value (e.g. when using the `parameter_types` macro) to either a production value
/// or to an environment variable or testing value (in case the `fast-runtime` feature is selected)
/// or one of two testing values depending on feature.
/// Note that the environment variable is evaluated _at compile time_.
///
/// Usage:
/// ```Rust
/// parameter_types! {
/// 	// Note that the env variable version parameter cannot be const.
/// 	pub LaunchPeriod: BlockNumber = prod_or_fast!(7 * DAYS, 1, "KSM_LAUNCH_PERIOD");
/// 	pub const VotingPeriod: BlockNumber = prod_or_fast!(7 * DAYS, 1 * MINUTES);
/// 	pub const EpochDuration: BlockNumber =
/// 		prod_or_fast!(1 * HOURS, "fast-runtime", 1 * MINUTES, "fast-runtime-10m", 10 * MINUTES);
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
