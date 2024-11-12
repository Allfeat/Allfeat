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
//! Autogenerated weights for pallet_midds
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 43.0.0
//! DATE: 2024-11-08, STEPS: `50`, REPEAT: `20`, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! WORST CASE MAP SIZE: `1000000`
//! HOSTNAME: `MacBook-Pro-de-Lois.local`, CPU: `<UNKNOWN>`
//! EXECUTION: , WASM-EXECUTION: Compiled, CHAIN: None, DB CACHE: 1024

// Executed Command:
// frame-omni-bencher
// v1
// benchmark
// pallet
// --runtime
// ./target/production/wbuild/harmonie-runtime/harmonie_runtime.wasm
// --genesis-builder-preset=development
// --pallet=pallet-midds
// --extrinsic=*
// --header=./HEADER
// --template=./.maintain/frame-weight-template.hbs
// --output=./runtime/shared/src/weights/

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]
#![allow(missing_docs)]

use frame_support::{traits::Get, weights::{Weight, constants::RocksDbWeight}};
use core::marker::PhantomData;

/// Weight functions needed for pallet_midds.
pub trait WeightInfo {
	fn register() -> Weight;
	fn update_field() -> Weight;
	fn unregister() -> Weight;
}

/// Weights for pallet_midds using the Allfeat node and recommended hardware.
pub struct AllfeatWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> pallet_midds::WeightInfo for AllfeatWeight<T> {
	/// Storage: `MusicalWorks::PendingMidds` (r:1 w:1)
	/// Proof: `MusicalWorks::PendingMidds` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Balances::Holds` (r:1 w:1)
	/// Proof: `Balances::Holds` (`max_values`: None, `max_size`: Some(127), added: 2602, mode: `MaxEncodedLen`)
	fn register() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `42`
		//  Estimated: `3592`
		// Minimum execution time: 37_000_000 picoseconds.
		Weight::from_parts(38_000_000, 3592)
			.saturating_add(T::DbWeight::get().reads(2_u64))
			.saturating_add(T::DbWeight::get().writes(2_u64))
	}
	/// Storage: `MusicalWorks::PendingMidds` (r:1 w:2)
	/// Proof: `MusicalWorks::PendingMidds` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Balances::Holds` (r:1 w:1)
	/// Proof: `Balances::Holds` (`max_values`: None, `max_size`: Some(127), added: 2602, mode: `MaxEncodedLen`)
	fn update_field() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `229`
		//  Estimated: `3694`
		// Minimum execution time: 42_000_000 picoseconds.
		Weight::from_parts(43_000_000, 3694)
			.saturating_add(T::DbWeight::get().reads(2_u64))
			.saturating_add(T::DbWeight::get().writes(3_u64))
	}
	/// Storage: `MusicalWorks::PendingMidds` (r:1 w:1)
	/// Proof: `MusicalWorks::PendingMidds` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Balances::Holds` (r:1 w:1)
	/// Proof: `Balances::Holds` (`max_values`: None, `max_size`: Some(127), added: 2602, mode: `MaxEncodedLen`)
	fn unregister() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `229`
		//  Estimated: `3694`
		// Minimum execution time: 32_000_000 picoseconds.
		Weight::from_parts(33_000_000, 3694)
			.saturating_add(T::DbWeight::get().reads(2_u64))
			.saturating_add(T::DbWeight::get().writes(2_u64))
	}
}
