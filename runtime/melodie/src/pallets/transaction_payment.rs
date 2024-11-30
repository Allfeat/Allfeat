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

use crate::*;
use frame::traits::fungible::{Balanced, Credit};
use frame_support::{
	parameter_types,
	traits::{Imbalance, OnUnbalanced},
	weights::ConstantMultiplier,
};
use shared_runtime::{currency::MICROALFT, SlowAdjustingFeeUpdate, WeightToFee};

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
	pub const TransactionByteFee: Balance = 10 * MICROALFT;
	pub const OperationalFeeMultiplier: u8 = 5;
}

impl pallet_transaction_payment::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type OnChargeTransaction = pallet_transaction_payment::FungibleAdapter<Balances, DealWithFees>;
	type OperationalFeeMultiplier = OperationalFeeMultiplier;
	type WeightToFee = WeightToFee;
	type LengthToFee = ConstantMultiplier<Balance, TransactionByteFee>;
	type FeeMultiplierUpdate = SlowAdjustingFeeUpdate<Self>;
}
