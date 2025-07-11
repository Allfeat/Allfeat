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

#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{
    Blake2_256, Parameter, StorageHasher, dispatch::DispatchResult, pallet_prelude::Member,
};
use parity_scale_codec::MaxEncodedLen;

mod musical_work;
mod party_identifier;
mod release;
mod track;

pub mod types;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

/// Generic Midds Identifier expected to be used for storing in pallets.
pub type MiddsId = u64;

/// Substrate-compatible MIDDS (Music Industry Decantralized Data Structure) interface definition.
pub trait Midds: Parameter + Member + MaxEncodedLen {
    const NAME: &'static str;

    #[cfg(feature = "runtime-benchmarks")]
    type BenchmarkHelper: benchmarking::BenchmarkHelperT<Self>;

    /// Return the integrity hash (with Blake2) of the encoded MIDDS.
    fn hash(&self) -> [u8; 32] {
        Blake2_256::hash(&self.encode())
    }

    /// A function that a MIDDS can implement to enforce specific validation logic.
    fn validate(&self) -> DispatchResult {
        Ok(())
    }
}

pub mod pallet_prelude {
    pub use super::{
        musical_work::MusicalWork,
        party_identifier::{Artist, Entity, PartyIdentifier, PartyType},
        release::Release,
        track::Track,
    };

    #[cfg(feature = "runtime-benchmarks")]
    pub use super::benchmarking::BenchmarkHelperT;
}
