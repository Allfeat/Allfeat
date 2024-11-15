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
use frame_support::{
	derive_impl,
	pallet_prelude::DispatchClass,
	weights::constants::{
		BlockExecutionWeight, ExtrinsicBaseWeight, RocksDbWeight, WEIGHT_REF_TIME_PER_SECOND,
	},
};
use frame_system::limits::BlockWeights;
use shared_runtime::{weights, RuntimeBlockLength};

/// We allow for 4 seconds of compute with a 12 second average block time, with maximum proof size
pub const WEIGHT_MILLISECS_PER_BLOCK: u64 = 4000;

pub const MAXIMUM_BLOCK_WEIGHT: frame_support::weights::Weight =
	frame_support::weights::Weight::from_parts(
		WEIGHT_MILLISECS_PER_BLOCK * WEIGHT_REF_TIME_PER_SECOND,
		u64::MAX,
	);

frame_support::parameter_types! {
	pub const Version: sp_version::RuntimeVersion = VERSION;
	pub RuntimeBlockWeights: BlockWeights = BlockWeights::builder()
		.base_block(BlockExecutionWeight::get())
		.for_class(DispatchClass::all(), |weights| {
			weights.base_extrinsic = ExtrinsicBaseWeight::get();
		})
		.for_class(DispatchClass::Normal, |weights| {
			weights.max_total = Some(shared_runtime::NORMAL_DISPATCH_RATIO * MAXIMUM_BLOCK_WEIGHT);
		})
		.for_class(DispatchClass::Operational, |weights| {
			weights.max_total = Some(MAXIMUM_BLOCK_WEIGHT);
			// Operational transactions have some extra reserved space, so that they
			// are included even if block reached `MAXIMUM_BLOCK_WEIGHT`.
			weights.reserved = Some(
				MAXIMUM_BLOCK_WEIGHT - shared_runtime::NORMAL_DISPATCH_RATIO * MAXIMUM_BLOCK_WEIGHT
			);
		})
		.avg_block_initialization(shared_runtime::AVERAGE_ON_INITIALIZE_RATIO)
		.build_or_panic();
}

#[derive_impl(frame_system::config_preludes::SolochainDefaultConfig)]
impl frame_system::Config for Runtime {
	type BlockWeights = RuntimeBlockWeights;
	type BlockLength = RuntimeBlockLength;
	type Nonce = Nonce;
	type Block = Block;
	type Hash = Hash;
	type AccountId = AccountId;
	type BlockHashCount = shared_runtime::BlockHashCount;
	type DbWeight = RocksDbWeight;
	type Version = Version;
	type AccountData = pallet_balances::AccountData<Balance>;
	type SystemWeightInfo = weights::system::AllfeatWeight<Runtime>;
	type SS58Prefix = ConstU16<441>;
	type MaxConsumers = ConstU32<16>;
}
