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

use frame_support::weights::constants::ParityDbWeight;
use sp_runtime::Weight;

/// Weight functions needed for pallet_validators.
pub trait WeightInfo {
    fn add_validator() -> Weight;
    fn remove_validator() -> Weight;
}

impl WeightInfo for () {
    /// Storage: `Validators::Validators` (r:1 w:1)
    /// Proof: `Validators::Validators` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
    fn add_validator() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `198`
        //  Estimated: `1683`
        // Minimum execution time: 18_000_000 picoseconds.
        Weight::from_parts(19_400_000, 1683)
            .saturating_add(ParityDbWeight::get().reads(1_u64))
            .saturating_add(ParityDbWeight::get().writes(1_u64))
    }
    /// Storage: `Validators::Validators` (r:1 w:1)
    /// Proof: `Validators::Validators` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
    fn remove_validator() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `101`
        //  Estimated: `1586`
        // Minimum execution time: 16_800_000 picoseconds.
        Weight::from_parts(19_120_000, 1586)
            .saturating_add(ParityDbWeight::get().reads(1_u64))
            .saturating_add(ParityDbWeight::get().writes(1_u64))
    }
}
