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
//! Autogenerated weights for pallet_election_provider_multi_phase
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 32.0.0
//! DATE: 2024-03-27, STEPS: `50`, REPEAT: `20`, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! WORST CASE MAP SIZE: `1000000`
//! HOSTNAME: `weights-allfeat`, CPU: `AMD EPYC-Milan Processor`
//! EXECUTION: , WASM-EXECUTION: Compiled, CHAIN: Some("harmonie-dev"), DB CACHE: 1024

// Executed Command:
// ./target/production/allfeat
// benchmark
// pallet
// --chain=harmonie-dev
// --steps=50
// --repeat=20
// --pallet=pallet_election_provider_multi_phase
// --extrinsic=*
// --wasm-execution=compiled
// --heap-pages=4096
// --output=./runtime/harmonie/src/weights/election-provider-multi-phase.rs
// --header=./HEADER
// --template=./.maintain/frame-weight-template.hbs

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]
#![allow(missing_docs)]

use frame_support::{traits::Get, weights::{Weight, constants::RocksDbWeight}};
use core::marker::PhantomData;

/// Weight functions needed for pallet_election_provider_multi_phase.
pub trait WeightInfo {
	fn on_initialize_nothing() -> Weight;
	fn on_initialize_open_signed() -> Weight;
	fn on_initialize_open_unsigned() -> Weight;
	fn finalize_signed_phase_accept_solution() -> Weight;
	fn finalize_signed_phase_reject_solution() -> Weight;
	fn create_snapshot_internal(v: u32, t: u32, ) -> Weight;
	fn elect_queued(a: u32, d: u32, ) -> Weight;
	fn submit() -> Weight;
	fn submit_unsigned(v: u32, t: u32, a: u32, d: u32, ) -> Weight;
	fn feasibility_check(v: u32, t: u32, a: u32, d: u32, ) -> Weight;
}

/// Weights for pallet_election_provider_multi_phase using the Allfeat node and recommended hardware.
pub struct AllfeatWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> pallet_election_provider_multi_phase::WeightInfo for AllfeatWeight<T> {
	/// Storage: `Staking::CurrentEra` (r:1 w:0)
	/// Proof: `Staking::CurrentEra` (`max_values`: Some(1), `max_size`: Some(4), added: 499, mode: `MaxEncodedLen`)
	/// Storage: `Staking::CurrentPlannedSession` (r:1 w:0)
	/// Proof: `Staking::CurrentPlannedSession` (`max_values`: Some(1), `max_size`: Some(4), added: 499, mode: `MaxEncodedLen`)
	/// Storage: `Staking::ErasStartSessionIndex` (r:1 w:0)
	/// Proof: `Staking::ErasStartSessionIndex` (`max_values`: None, `max_size`: Some(16), added: 2491, mode: `MaxEncodedLen`)
	/// Storage: `Babe::EpochIndex` (r:1 w:0)
	/// Proof: `Babe::EpochIndex` (`max_values`: Some(1), `max_size`: Some(8), added: 503, mode: `MaxEncodedLen`)
	/// Storage: `Babe::GenesisSlot` (r:1 w:0)
	/// Proof: `Babe::GenesisSlot` (`max_values`: Some(1), `max_size`: Some(8), added: 503, mode: `MaxEncodedLen`)
	/// Storage: `Babe::CurrentSlot` (r:1 w:0)
	/// Proof: `Babe::CurrentSlot` (`max_values`: Some(1), `max_size`: Some(8), added: 503, mode: `MaxEncodedLen`)
	/// Storage: `Staking::ForceEra` (r:1 w:0)
	/// Proof: `Staking::ForceEra` (`max_values`: Some(1), `max_size`: Some(1), added: 496, mode: `MaxEncodedLen`)
	/// Storage: `ElectionProviderMultiPhase::CurrentPhase` (r:1 w:0)
	/// Proof: `ElectionProviderMultiPhase::CurrentPhase` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	fn on_initialize_nothing() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `854`
		//  Estimated: `3481`
		// Minimum execution time: 16_224_000 picoseconds.
		Weight::from_parts(16_796_000, 3481)
			.saturating_add(T::DbWeight::get().reads(8_u64))
	}
	/// Storage: `ElectionProviderMultiPhase::Round` (r:1 w:0)
	/// Proof: `ElectionProviderMultiPhase::Round` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `ElectionProviderMultiPhase::CurrentPhase` (r:1 w:1)
	/// Proof: `ElectionProviderMultiPhase::CurrentPhase` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	fn on_initialize_open_signed() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `42`
		//  Estimated: `1527`
		// Minimum execution time: 6_961_000 picoseconds.
		Weight::from_parts(7_341_000, 1527)
			.saturating_add(T::DbWeight::get().reads(2_u64))
			.saturating_add(T::DbWeight::get().writes(1_u64))
	}
	/// Storage: `ElectionProviderMultiPhase::Round` (r:1 w:0)
	/// Proof: `ElectionProviderMultiPhase::Round` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `ElectionProviderMultiPhase::CurrentPhase` (r:1 w:1)
	/// Proof: `ElectionProviderMultiPhase::CurrentPhase` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	fn on_initialize_open_unsigned() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `42`
		//  Estimated: `1527`
		// Minimum execution time: 7_782_000 picoseconds.
		Weight::from_parts(8_282_000, 1527)
			.saturating_add(T::DbWeight::get().reads(2_u64))
			.saturating_add(T::DbWeight::get().writes(1_u64))
	}
	/// Storage: `System::Account` (r:1 w:1)
	/// Proof: `System::Account` (`max_values`: None, `max_size`: Some(116), added: 2591, mode: `MaxEncodedLen`)
	/// Storage: `ElectionProviderMultiPhase::QueuedSolution` (r:0 w:1)
	/// Proof: `ElectionProviderMultiPhase::QueuedSolution` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	fn finalize_signed_phase_accept_solution() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `91`
		//  Estimated: `3581`
		// Minimum execution time: 24_728_000 picoseconds.
		Weight::from_parts(25_417_000, 3581)
			.saturating_add(T::DbWeight::get().reads(1_u64))
			.saturating_add(T::DbWeight::get().writes(2_u64))
	}
	/// Storage: `System::Account` (r:1 w:1)
	/// Proof: `System::Account` (`max_values`: None, `max_size`: Some(116), added: 2591, mode: `MaxEncodedLen`)
	fn finalize_signed_phase_reject_solution() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `91`
		//  Estimated: `3581`
		// Minimum execution time: 15_964_000 picoseconds.
		Weight::from_parts(16_505_000, 3581)
			.saturating_add(T::DbWeight::get().reads(1_u64))
			.saturating_add(T::DbWeight::get().writes(1_u64))
	}
	/// Storage: `ElectionProviderMultiPhase::SnapshotMetadata` (r:0 w:1)
	/// Proof: `ElectionProviderMultiPhase::SnapshotMetadata` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `ElectionProviderMultiPhase::DesiredTargets` (r:0 w:1)
	/// Proof: `ElectionProviderMultiPhase::DesiredTargets` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `ElectionProviderMultiPhase::Snapshot` (r:0 w:1)
	/// Proof: `ElectionProviderMultiPhase::Snapshot` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// The range of component `v` is `[1000, 2000]`.
	/// The range of component `t` is `[500, 1000]`.
	fn create_snapshot_internal(v: u32, _t: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0`
		//  Estimated: `0`
		// Minimum execution time: 126_490_000 picoseconds.
		Weight::from_parts(131_586_000, 0)
			// Standard Error: 5_186
			.saturating_add(Weight::from_parts(146_933, 0).saturating_mul(v.into()))
			.saturating_add(T::DbWeight::get().writes(3_u64))
	}
	/// Storage: `ElectionProviderMultiPhase::SignedSubmissionIndices` (r:1 w:1)
	/// Proof: `ElectionProviderMultiPhase::SignedSubmissionIndices` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `ElectionProviderMultiPhase::SignedSubmissionNextIndex` (r:1 w:1)
	/// Proof: `ElectionProviderMultiPhase::SignedSubmissionNextIndex` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `ElectionProviderMultiPhase::SnapshotMetadata` (r:1 w:1)
	/// Proof: `ElectionProviderMultiPhase::SnapshotMetadata` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `ElectionProviderMultiPhase::SignedSubmissionsMap` (r:1 w:0)
	/// Proof: `ElectionProviderMultiPhase::SignedSubmissionsMap` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `ElectionProviderMultiPhase::QueuedSolution` (r:1 w:1)
	/// Proof: `ElectionProviderMultiPhase::QueuedSolution` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `ElectionProviderMultiPhase::Round` (r:1 w:1)
	/// Proof: `ElectionProviderMultiPhase::Round` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `ElectionProviderMultiPhase::CurrentPhase` (r:1 w:1)
	/// Proof: `ElectionProviderMultiPhase::CurrentPhase` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `ElectionProviderMultiPhase::DesiredTargets` (r:0 w:1)
	/// Proof: `ElectionProviderMultiPhase::DesiredTargets` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `ElectionProviderMultiPhase::Snapshot` (r:0 w:1)
	/// Proof: `ElectionProviderMultiPhase::Snapshot` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// The range of component `a` is `[500, 800]`.
	/// The range of component `d` is `[200, 400]`.
	fn elect_queued(a: u32, d: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `265 + a * (576 ±0) + d * (36 ±0)`
		//  Estimated: `3817 + a * (576 ±0) + d * (37 ±0)`
		// Minimum execution time: 246_249_000 picoseconds.
		Weight::from_parts(61_174_077, 3817)
			// Standard Error: 6_059
			.saturating_add(Weight::from_parts(252_001, 0).saturating_mul(a.into()))
			// Standard Error: 9_082
			.saturating_add(Weight::from_parts(250_587, 0).saturating_mul(d.into()))
			.saturating_add(T::DbWeight::get().reads(7_u64))
			.saturating_add(T::DbWeight::get().writes(8_u64))
			.saturating_add(Weight::from_parts(0, 576).saturating_mul(a.into()))
			.saturating_add(Weight::from_parts(0, 37).saturating_mul(d.into()))
	}
	/// Storage: `ElectionProviderMultiPhase::CurrentPhase` (r:1 w:0)
	/// Proof: `ElectionProviderMultiPhase::CurrentPhase` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `ElectionProviderMultiPhase::Round` (r:1 w:0)
	/// Proof: `ElectionProviderMultiPhase::Round` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `ElectionProviderMultiPhase::SnapshotMetadata` (r:1 w:0)
	/// Proof: `ElectionProviderMultiPhase::SnapshotMetadata` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `ElectionProviderMultiPhase::SignedSubmissionIndices` (r:1 w:1)
	/// Proof: `ElectionProviderMultiPhase::SignedSubmissionIndices` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `ElectionProviderMultiPhase::SignedSubmissionNextIndex` (r:1 w:1)
	/// Proof: `ElectionProviderMultiPhase::SignedSubmissionNextIndex` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `TransactionPayment::NextFeeMultiplier` (r:1 w:0)
	/// Proof: `TransactionPayment::NextFeeMultiplier` (`max_values`: Some(1), `max_size`: Some(16), added: 511, mode: `MaxEncodedLen`)
	/// Storage: `ElectionProviderMultiPhase::SignedSubmissionsMap` (r:0 w:1)
	/// Proof: `ElectionProviderMultiPhase::SignedSubmissionsMap` (`max_values`: None, `max_size`: None, mode: `Measured`)
	fn submit() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `820`
		//  Estimated: `2305`
		// Minimum execution time: 36_363_000 picoseconds.
		Weight::from_parts(37_376_000, 2305)
			.saturating_add(T::DbWeight::get().reads(6_u64))
			.saturating_add(T::DbWeight::get().writes(3_u64))
	}
	/// Storage: `ElectionProviderMultiPhase::CurrentPhase` (r:1 w:0)
	/// Proof: `ElectionProviderMultiPhase::CurrentPhase` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `ElectionProviderMultiPhase::Round` (r:1 w:0)
	/// Proof: `ElectionProviderMultiPhase::Round` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `ElectionProviderMultiPhase::DesiredTargets` (r:1 w:0)
	/// Proof: `ElectionProviderMultiPhase::DesiredTargets` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `ElectionProviderMultiPhase::QueuedSolution` (r:1 w:1)
	/// Proof: `ElectionProviderMultiPhase::QueuedSolution` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `ElectionProviderMultiPhase::SnapshotMetadata` (r:1 w:0)
	/// Proof: `ElectionProviderMultiPhase::SnapshotMetadata` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `ElectionProviderMultiPhase::Snapshot` (r:1 w:0)
	/// Proof: `ElectionProviderMultiPhase::Snapshot` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `ElectionProviderMultiPhase::MinimumUntrustedScore` (r:1 w:0)
	/// Proof: `ElectionProviderMultiPhase::MinimumUntrustedScore` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// The range of component `v` is `[1000, 2000]`.
	/// The range of component `t` is `[500, 1000]`.
	/// The range of component `a` is `[500, 800]`.
	/// The range of component `d` is `[200, 400]`.
	fn submit_unsigned(v: u32, t: u32, a: u32, _d: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `147 + t * (20 ±0) + v * (349 ±0)`
		//  Estimated: `1632 + t * (20 ±0) + v * (349 ±0)`
		// Minimum execution time: 4_406_917_000 picoseconds.
		Weight::from_parts(4_430_963_000, 1632)
			// Standard Error: 14_091
			.saturating_add(Weight::from_parts(139_406, 0).saturating_mul(v.into()))
			// Standard Error: 41_757
			.saturating_add(Weight::from_parts(3_882_568, 0).saturating_mul(a.into()))
			.saturating_add(T::DbWeight::get().reads(7_u64))
			.saturating_add(T::DbWeight::get().writes(1_u64))
			.saturating_add(Weight::from_parts(0, 20).saturating_mul(t.into()))
			.saturating_add(Weight::from_parts(0, 349).saturating_mul(v.into()))
	}
	/// Storage: `ElectionProviderMultiPhase::DesiredTargets` (r:1 w:0)
	/// Proof: `ElectionProviderMultiPhase::DesiredTargets` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `ElectionProviderMultiPhase::Snapshot` (r:1 w:0)
	/// Proof: `ElectionProviderMultiPhase::Snapshot` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `ElectionProviderMultiPhase::Round` (r:1 w:0)
	/// Proof: `ElectionProviderMultiPhase::Round` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `ElectionProviderMultiPhase::MinimumUntrustedScore` (r:1 w:0)
	/// Proof: `ElectionProviderMultiPhase::MinimumUntrustedScore` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// The range of component `v` is `[1000, 2000]`.
	/// The range of component `t` is `[500, 1000]`.
	/// The range of component `a` is `[500, 800]`.
	/// The range of component `d` is `[200, 400]`.
	fn feasibility_check(v: u32, t: u32, a: u32, _d: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `122 + t * (20 ±0) + v * (349 ±0)`
		//  Estimated: `1607 + t * (20 ±0) + v * (349 ±0)`
		// Minimum execution time: 3_839_786_000 picoseconds.
		Weight::from_parts(3_878_524_000, 1607)
			// Standard Error: 12_664
			.saturating_add(Weight::from_parts(203_975, 0).saturating_mul(v.into()))
			// Standard Error: 37_528
			.saturating_add(Weight::from_parts(3_235_561, 0).saturating_mul(a.into()))
			.saturating_add(T::DbWeight::get().reads(4_u64))
			.saturating_add(Weight::from_parts(0, 20).saturating_mul(t.into()))
			.saturating_add(Weight::from_parts(0, 349).saturating_mul(v.into()))
	}
}
