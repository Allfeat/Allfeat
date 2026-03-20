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

//! Weights for `pallet_ats`
//!
//! Placeholder: re-run benchmarks after migrating to pallet-ats v0.2.0.

#![allow(unused_parens)]
#![allow(unused_imports)]
#![allow(missing_docs)]
#![allow(dead_code)]

use core::marker::PhantomData;
use frame_support::{traits::Get, weights::Weight};
use pallet_ats::WeightInfo;

pub struct AllfeatWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for AllfeatWeight<T> {
    fn create() -> Weight {
        Weight::from_parts(54_000_000, 3694)
            .saturating_add(T::DbWeight::get().reads(4_u64))
            .saturating_add(T::DbWeight::get().writes(7_u64))
    }

    fn update() -> Weight {
        Weight::from_parts(55_000_000, 3694)
            .saturating_add(T::DbWeight::get().reads(3_u64))
            .saturating_add(T::DbWeight::get().writes(4_u64))
    }

    fn revoke(v: u32) -> Weight {
        Weight::from_parts(30_000_000, 3694)
            .saturating_add(Weight::from_parts(5_000_000, 0).saturating_mul(v as u64))
            .saturating_add(T::DbWeight::get().reads(2_u64))
            .saturating_add(T::DbWeight::get().writes(3_u64))
            .saturating_add(
                T::DbWeight::get()
                    .reads_writes(1, 1)
                    .saturating_mul(v as u64),
            )
    }
    fn create_on_behalf(_s: u32) -> Weight {
        Weight::from_parts(20_000, 0)
    }

    fn update_on_behalf(_s: u32) -> Weight {
        Weight::from_parts(20_000, 0)
    }

    fn revoke_on_behalf(v: u32) -> Weight {
        Weight::from_parts(
            20_000_u64.saturating_add(5_000_u64.saturating_mul(u64::from(v))),
            0,
        )
    }
}
