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

//! Common runtime code for Allfeat.

#![cfg_attr(not(feature = "std"), no_std)]

use allfeat_primitives::{Balance, BlockNumber};
use frame_support::{
    parameter_types,
    sp_runtime::{FixedPointNumber, Perbill, Perquintill, traits::Bounded},
};
use frame_system::limits::BlockLength;
use pallet_transaction_payment::{Multiplier, TargetedFeeAdjustment};
use sp_core::U256;

pub mod elections;

pub mod currency;

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

/// We assume that an on-initialize consumes 1% of the weight on average, hence a single extrinsic
/// will not be allowed to consume more than `AvailableBlockRatio - 1%`.
pub const AVERAGE_ON_INITIALIZE_RATIO: Perbill = Perbill::from_percent(1);
/// We allow `Normal` extrinsics to fill up the block up to 75%, the rest can be used
/// by  Operational  extrinsics.
pub const NORMAL_DISPATCH_RATIO: Perbill = Perbill::from_percent(75);

/// Convert a balance to an unsigned 256-bit number, use in nomination pools.
pub struct BalanceToU256;
impl frame_support::sp_runtime::traits::Convert<Balance, U256> for BalanceToU256 {
    fn convert(n: Balance) -> U256 {
        n.into()
    }
}

/// Convert an unsigned 256-bit number to balance, use in nomination pools.
pub struct U256ToBalance;
impl frame_support::sp_runtime::traits::Convert<U256, Balance> for U256ToBalance {
    fn convert(n: U256) -> Balance {
        use frame_support::traits::Defensive;
        n.try_into().defensive_unwrap_or(Balance::MAX)
    }
}
