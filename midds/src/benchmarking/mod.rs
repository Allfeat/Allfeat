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

extern crate alloc;

pub mod musical_work;
pub mod party_identifier;
pub mod release;
pub mod track;

use alloc::{vec, vec::Vec};
use core::fmt::Debug;
use frame_support::{BoundedVec, traits::Get};
use parity_scale_codec::Encode;

use crate::Midds;

pub trait BenchmarkHelperT<T: Midds> {
    /// Construct a minimal instance of the MIDDS (without any dynamic sized type >0)
    /// This is the minimum sized instance possible.
    fn build_base() -> T;
    fn build_sized(target_size: usize) -> T;

    /// Utilities gard that ensure we do not deal with an input size lower or higher than the minimal and maximal expected for this MIDDS.
    fn build_base_with_checked_target_size(target_size: usize) -> T {
        let base = Self::build_base();
        let minimal_size = base.encoded_size();
        let max_size = T::max_encoded_len();

        if target_size < minimal_size || target_size > max_size {
            panic!("Expected input benchmark size in range of valid MIDDS size !")
        }

        base
    }
}

/// Benchmark helper function to generate a boundedvec type based on a specified size.
fn fill_boundedvec<T: Clone + Debug, N: Get<u32>>(
    value: T,
    requested_size: usize,
) -> BoundedVec<T, N> {
    let max_size = N::get();
    let actual_size = requested_size.min(max_size as usize);

    let mut vec = BoundedVec::<T, N>::with_bounded_capacity(actual_size);
    for _ in 0..actual_size {
        vec.try_push(value.clone()).expect("Within bounds");
    }
    vec
}

pub fn fill_boundedvec_to_fit<T, N>(
    value: T,
    max_items: usize,
    base_encoded_size: usize,
    target_size: usize,
) -> BoundedVec<T, N>
where
    T: Clone + Encode + core::fmt::Debug,
    N: Get<u32>,
{
    let mut low = 0;
    let mut high = max_items.min(N::get() as usize);
    let mut best = 0;

    while low <= high {
        let mid = (low + high) / 2;
        let vec: Vec<T> = vec![value.clone(); mid];
        let encoded = vec.encode();
        let total_size = base_encoded_size + encoded.len();

        match total_size.cmp(&target_size) {
            core::cmp::Ordering::Equal => {
                best = mid;
                break;
            }
            core::cmp::Ordering::Less => {
                best = mid;
                low = mid + 1;
            }
            core::cmp::Ordering::Greater => {
                if mid == 0 {
                    break;
                }
                high = mid - 1;
            }
        }
    }

    fill_boundedvec::<T, N>(value, best)
}

#[cfg(test)]
mod tests {
    use crate::{
        musical_work::MusicalWork, pallet_prelude::PartyIdentifier, release::Release, track::Track,
    };

    use super::*;
    use frame_support::traits::ConstU32;

    #[test]
    fn fill_boundedvec_respects_bounds() {
        type Bound = ConstU32<5>;

        let vec = super::fill_boundedvec::<u8, Bound>(1u8, 3);
        assert_eq!(vec.len(), 3);
        let as_vec: Vec<u8> = vec.clone().into();
        assert_eq!(as_vec, vec![1u8; 3]);

        let vec = super::fill_boundedvec::<u8, Bound>(2u8, 10);
        assert_eq!(vec.len(), 5);
    }

    #[test]
    fn fill_boundedvec_to_fit_calculates_capacity() {
        type Bound = ConstU32<10>;
        let base_size = 10usize;

        let result = super::fill_boundedvec_to_fit::<u8, Bound>(1u8, 10, base_size, 15);
        assert_eq!(result.len(), 4);
        assert!(base_size + result.encode().len() <= 15);
    }

    #[test]
    fn fill_boundedvec_to_fit_handles_small_target() {
        type Bound = ConstU32<10>;
        let base_size = 10usize;

        let result = super::fill_boundedvec_to_fit::<u8, Bound>(1u8, 10, base_size, base_size);
        assert_eq!(result.len(), 0);
    }

    fn verify_full_range<T: Midds>() {
        let min = <T as Midds>::BenchmarkHelper::build_base().encoded_size();
        let max = T::max_encoded_len();

        let at_min = <T as Midds>::BenchmarkHelper::build_sized(min);
        assert_eq!(at_min.encoded_size(), min);

        let at_max = <T as Midds>::BenchmarkHelper::build_sized(max);
        assert!(at_max.encoded_size() <= max);
    }

    #[test]
    fn benchmark_helpers_cover_full_size() {
        verify_full_range::<Track>();
        verify_full_range::<Release>();
        verify_full_range::<MusicalWork>();
        verify_full_range::<PartyIdentifier>();
    }
}
