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

//! Autogenerated weights for pallet_sudo
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 32.0.0
//! DATE: 2025-06-11, STEPS: `50`, REPEAT: `20`, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! WORST CASE MAP SIZE: `1000000`
//! HOSTNAME: `melodie-node-weights`, CPU: `<UNKNOWN>`
//! EXECUTION: , WASM-EXECUTION: Compiled, CHAIN: None, DB CACHE: 1024

// Executed Command:
// frame-omni-bencher
// v1
// benchmark
// pallet
// --runtime
// ./target/release/wbuild/melodie-runtime/melodie_runtime.compact.compressed.wasm
// --genesis-builder-preset=development
// --pallet=pallet_sudo
// --extrinsic=*
// --output=./runtime/shared/src/weights/sudo.rs
// --header=./HEADER
// --template=./.maintain/frame-weight-template.hbs

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]
#![allow(missing_docs)]

use frame_support::{traits::Get, weights::{Weight, constants::RocksDbWeight}};
use core::marker::PhantomData;

/// Weight functions needed for pallet_sudo.
pub trait WeightInfo {
	fn set_key() -> Weight;
	fn sudo() -> Weight;
	fn sudo_as() -> Weight;
	fn remove_key() -> Weight;
	fn check_only_sudo_account() -> Weight;
}

/// Weights for pallet_sudo using the Allfeat node and recommended hardware.
pub struct AllfeatWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> pallet_sudo::WeightInfo for AllfeatWeight<T> {
	/// Storage: `Sudo::Key` (r:1 w:1)
	/// Proof: `Sudo::Key` (`max_values`: Some(1), `max_size`: Some(32), added: 527, mode: `MaxEncodedLen`)
	fn set_key() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `132`
		//  Estimated: `1517`
		// Minimum execution time: 17_920_000 picoseconds.
		Weight::from_parts(19_920_000, 1517)
			.saturating_add(T::DbWeight::get().reads(1_u64))
			.saturating_add(T::DbWeight::get().writes(1_u64))
	}
	/// Storage: `Sudo::Key` (r:1 w:0)
	/// Proof: `Sudo::Key` (`max_values`: Some(1), `max_size`: Some(32), added: 527, mode: `MaxEncodedLen`)
	fn sudo() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `132`
		//  Estimated: `1517`
		// Minimum execution time: 19_880_000 picoseconds.
		Weight::from_parts(21_880_000, 1517)
			.saturating_add(T::DbWeight::get().reads(1_u64))
	}
	/// Storage: `Sudo::Key` (r:1 w:0)
	/// Proof: `Sudo::Key` (`max_values`: Some(1), `max_size`: Some(32), added: 527, mode: `MaxEncodedLen`)
	fn sudo_as() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `132`
		//  Estimated: `1517`
		// Minimum execution time: 19_921_000 picoseconds.
		Weight::from_parts(21_960_000, 1517)
			.saturating_add(T::DbWeight::get().reads(1_u64))
	}
	/// Storage: `Sudo::Key` (r:1 w:1)
	/// Proof: `Sudo::Key` (`max_values`: Some(1), `max_size`: Some(32), added: 527, mode: `MaxEncodedLen`)
	fn remove_key() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `132`
		//  Estimated: `1517`
		// Minimum execution time: 16_561_000 picoseconds.
		Weight::from_parts(18_161_000, 1517)
			.saturating_add(T::DbWeight::get().reads(1_u64))
			.saturating_add(T::DbWeight::get().writes(1_u64))
	}
	/// Storage: `Sudo::Key` (r:1 w:0)
	/// Proof: `Sudo::Key` (`max_values`: Some(1), `max_size`: Some(32), added: 527, mode: `MaxEncodedLen`)
	fn check_only_sudo_account() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `132`
		//  Estimated: `1517`
		// Minimum execution time: 8_240_000 picoseconds.
		Weight::from_parts(9_400_000, 1517)
			.saturating_add(T::DbWeight::get().reads(1_u64))
	}
}
