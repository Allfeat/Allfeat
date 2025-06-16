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
