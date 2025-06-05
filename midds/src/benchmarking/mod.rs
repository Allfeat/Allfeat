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

pub mod musical_work;
pub mod party_identifier;
pub mod release;
pub mod track;

use core::fmt::Debug;
use frame_support::{traits::Get, BoundedVec};

use crate::Midds;

pub trait BenchmarkHelperT<T: Midds> {
	const FIELD_MAX_SIZE: u32;

	fn build_mock() -> T;
	fn build_sized_mock(size: u32) -> T;
}

/// Benchmark helper function to generate a boundedvec type based on a specified size.
pub fn fill_boundedvec<T: Clone + Debug, N: Get<u32>>(
	value: T,
	requested_size: u32,
) -> BoundedVec<T, N> {
	let max_size = N::get();
	let actual_size = requested_size.min(max_size);

	let mut vec = BoundedVec::<T, N>::with_bounded_capacity(actual_size as usize);
	for _ in 0..actual_size {
		vec.try_push(value.clone()).expect("Within bounds");
	}
	vec
}
