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
//! Autogenerated weights for pallet_midds_stakeholders
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 32.0.0
//! DATE: 2024-11-18, STEPS: `50`, REPEAT: `20`, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! WORST CASE MAP SIZE: `1000000`
//! HOSTNAME: `weights-melodie-bencher`, CPU: `<UNKNOWN>`
//! EXECUTION: , WASM-EXECUTION: Compiled, CHAIN: None, DB CACHE: 1024

// Executed Command:
// frame-omni-bencher
// v1
// benchmark
// pallet
// --runtime
// ./target/production/wbuild/melodie-runtime/melodie_runtime.compact.compressed.wasm
// --genesis-builder-preset=development
// --pallet=pallet_midds_stakeholders
// --extrinsic=*
// --output=./runtime/shared/src/weights/midds_stakeholders.rs
// --header=./HEADER
// --template=./.maintain/frame-weight-template.hbs

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]
#![allow(missing_docs)]

use polkadot_sdk::polkadot_sdk_frame as frame;
use frame::{traits::Get, deps::frame_support::weights::{Weight, constants::RocksDbWeight}};
use core::marker::PhantomData;

/// Weight functions needed for pallet_midds_stakeholders.
pub trait WeightInfo {
	fn register() -> Weight;
	fn update_field() -> Weight;
	fn unregister() -> Weight;
}

/// Weights for pallet_midds_stakeholders using the Allfeat node and recommended hardware.
pub struct AllfeatWeight<T>(PhantomData<T>);
impl<T: polkadot_sdk::frame_system::Config> pallet_midds::WeightInfo for AllfeatWeight<T> {
	/// Storage: `Stakeholders::PendingMidds` (r:1 w:1)
	/// Proof: `Stakeholders::PendingMidds` (`max_values`: None, `max_size`: Some(870), added: 3345, mode: `MaxEncodedLen`)
	/// Storage: `Balances::Holds` (r:1 w:1)
	/// Proof: `Balances::Holds` (`max_values`: None, `max_size`: Some(139), added: 2614, mode: `MaxEncodedLen`)
	fn register() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `4`
		//  Estimated: `4335`
		// Minimum execution time: 56_241_000 picoseconds.
		Weight::from_parts(57_121_000, 4335)
			.saturating_add(T::DbWeight::get().reads(2_u64))
			.saturating_add(T::DbWeight::get().writes(2_u64))
	}
	/// Storage: `Stakeholders::PendingMidds` (r:1 w:2)
	/// Proof: `Stakeholders::PendingMidds` (`max_values`: None, `max_size`: Some(870), added: 3345, mode: `MaxEncodedLen`)
	/// Storage: `Balances::Holds` (r:1 w:1)
	/// Proof: `Balances::Holds` (`max_values`: None, `max_size`: Some(139), added: 2614, mode: `MaxEncodedLen`)
	fn update_field() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `283`
		//  Estimated: `4335`
		// Minimum execution time: 55_801_000 picoseconds.
		Weight::from_parts(56_521_000, 4335)
			.saturating_add(T::DbWeight::get().reads(2_u64))
			.saturating_add(T::DbWeight::get().writes(3_u64))
	}
	/// Storage: `Stakeholders::PendingMidds` (r:1 w:1)
	/// Proof: `Stakeholders::PendingMidds` (`max_values`: None, `max_size`: Some(870), added: 3345, mode: `MaxEncodedLen`)
	/// Storage: `Balances::Holds` (r:1 w:1)
	/// Proof: `Balances::Holds` (`max_values`: None, `max_size`: Some(139), added: 2614, mode: `MaxEncodedLen`)
	fn unregister() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `283`
		//  Estimated: `4335`
		// Minimum execution time: 49_120_000 picoseconds.
		Weight::from_parts(49_840_000, 4335)
			.saturating_add(T::DbWeight::get().reads(2_u64))
			.saturating_add(T::DbWeight::get().writes(2_u64))
	}
}