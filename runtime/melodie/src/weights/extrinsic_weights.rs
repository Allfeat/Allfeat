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

//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 57.0.0
//! DATE: 2026-06-12 (Y/M/D)
//! HOSTNAME: `c3-16-2026-06-12-14-19`, CPU: `AMD EPYC-Milan Processor`
//!
//! SHORT-NAME: `extrinsic`, LONG-NAME: `ExtrinsicBase`, RUNTIME: `allfeat-melodie-3`
//! WARMUPS: `10`, REPEAT: `100`
//! WEIGHT-PATH: `/tmp/.tmpjjNvej`
//! WEIGHT-METRIC: `Average`, WEIGHT-MUL: `1.0`, WEIGHT-ADD: `0`

// Executed Command:
//  pop
//  bench
//  overhead
//  --runtime=/home/debian/Allfeat/target/release/wbuild/melodie-runtime/melodie_runtime.wasm
//  --genesis-builder=runtime
//  --genesis-builder-preset=development
//  --weight-path=.
//  --profile=release

pub mod constants {
	use polkadot_sdk::frame_support::{
		parameter_types,
		weights::{Weight, constants},
	};

	parameter_types! {
		/// Weight of executing a NO-OP extrinsic, for example `System::remark`.
		/// Calculated by multiplying the *Average* with `1.0` and adding `0`.
		///
		/// Stats nanoseconds:
		///   Min, Max: 137_921, 141_075
		///   Average:  138_893
		///   Median:   138_756
		///   Std-Dev:  559.75
		///
		/// Percentiles nanoseconds:
		///   99th: 140_429
		///   95th: 139_861
		///   75th: 139_205
		pub const ExtrinsicBaseWeight: Weight =
			Weight::from_parts(constants::WEIGHT_REF_TIME_PER_NANOS.saturating_mul(138_893), 346);
	}

	#[cfg(test)]
	mod test_weights {
		use polkadot_sdk::frame_support::weights::constants;

		/// Checks that the weight exists and is sane.
		// NOTE: If this test fails but you are sure that the generated values are fine,
		// you can delete it.
		#[test]
		fn sane() {
			let w = super::constants::ExtrinsicBaseWeight::get();

			// At least 10 µs.
			assert!(
				w.ref_time() >= 10u64 * constants::WEIGHT_REF_TIME_PER_MICROS,
				"Weight should be at least 10 µs."
			);
			// At most 1 ms.
			assert!(
				w.ref_time() <= constants::WEIGHT_REF_TIME_PER_MILLIS,
				"Weight should be at most 1 ms."
			);
		}
	}
}
