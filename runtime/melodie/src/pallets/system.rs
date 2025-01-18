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
	derive_impl,
	traits::{ConstU16, ConstU32},
	weights::{
		constants::{ParityDbWeight, WEIGHT_REF_TIME_PER_SECOND},
		Weight,
	},
};
use frame_system::limits::BlockWeights;
use shared_runtime::{weights, RuntimeBlockLength, NORMAL_DISPATCH_RATIO};

/// We allow for 4 seconds of compute with a 12 second average block time, with maximum proof size
pub const WEIGHT_MILLISECS_PER_BLOCK: u64 = 4000;

frame_support::parameter_types! {
	pub const Version: sp_version::RuntimeVersion = VERSION;
	/// We allow for 2 seconds of compute with a 6 second average block time.
	pub RuntimeBlockWeights: BlockWeights = BlockWeights::with_sensible_defaults(
		Weight::from_parts(2u64 * WEIGHT_REF_TIME_PER_SECOND, u64::MAX),
		NORMAL_DISPATCH_RATIO,
	);
}

#[derive_impl(frame_system::config_preludes::SolochainDefaultConfig)]
impl frame_system::Config for Runtime {
	type BlockWeights = RuntimeBlockWeights;
	type BlockLength = RuntimeBlockLength;
	type Nonce = Nonce;
	type Block = Block;
	type Hash = allfeat_primitives::Hash;
	type AccountId = AccountId;
	type BlockHashCount = shared_runtime::BlockHashCount;
	type DbWeight = ParityDbWeight;
	type Version = Version;
	type AccountData = pallet_balances::AccountData<Balance>;
	type SystemWeightInfo = weights::system::AllfeatWeight<Runtime>;
	type SS58Prefix = ConstU16<42>;
	type MaxConsumers = ConstU32<16>;
}
