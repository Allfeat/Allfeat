// This file is part of Allfeat.

// Copyright (C) 2022-2025 Allfeat.
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

use crate::*;
use frame_support::{
	dispatch::DispatchClass,
	parameter_types,
	sp_runtime::Perbill,
	traits::{
		fungible::{Balanced, Credit},
		Imbalance, OnUnbalanced,
	},
	weights::{
		ConstantMultiplier, WeightToFeeCoefficient, WeightToFeeCoefficients, WeightToFeePolynomial,
	},
};
use shared_runtime::{
	currency::{MICROAFT, MILLIAFT},
	SlowAdjustingFeeUpdate,
};

pub struct DealWithFees;
impl OnUnbalanced<Credit<AccountId, Balances>> for DealWithFees {
	fn on_unbalanceds(mut fees_then_tips: impl Iterator<Item = Credit<AccountId, Balances>>) {
		if let Some(mut amount) = fees_then_tips.next() {
			// for fees, 100% to author
			if let Some(tips) = fees_then_tips.next() {
				// for tips, if any, 100% to author
				tips.merge_into(&mut amount);
			}

			if let Some(author) = Authorship::author() {
				match Balances::resolve(&author, amount) {
					Ok(_) => (),
					Err(_amount) => {
						todo!()
					},
				}
			}
		}
	}
}

parameter_types! {
	pub const TransactionByteFee: Balance = 10 * MICROAFT;
	pub const OperationalFeeMultiplier: u8 = 5;
	pub const WeightFeeFactor: Balance = 10 * MILLIAFT;
}

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
		let p = WeightFeeFactor::get();
		let q = Balance::from(
			RuntimeBlockWeights::get().get(DispatchClass::Normal).base_extrinsic.ref_time(),
		);
		let coefficient = WeightToFeeCoefficient {
			degree: 1,
			negative: false,
			coeff_frac: Perbill::from_rational(p % q, q),
			coeff_integer: p / q,
		};
		smallvec::smallvec![coefficient]
	}
}

impl pallet_transaction_payment::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type OnChargeTransaction = pallet_transaction_payment::FungibleAdapter<Balances, DealWithFees>;
	type OperationalFeeMultiplier = OperationalFeeMultiplier;
	type WeightToFee = WeightToFee;
	type LengthToFee = ConstantMultiplier<Balance, TransactionByteFee>;
	type FeeMultiplierUpdate = SlowAdjustingFeeUpdate<Self>;
	type WeightInfo = pallet_transaction_payment::weights::SubstrateWeight<Runtime>;
}
