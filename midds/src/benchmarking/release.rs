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

use crate::{
    release::Release,
    types::{
        release::{ReleaseFormat, ReleasePackaging, ReleaseStatus, ReleaseType},
        utils::{Country, Date},
    },
};
use alloc::vec;

use super::{BenchmarkHelperT, fill_boundedvec};

pub struct BenchmarkHelper;

impl BenchmarkHelperT<Release> for BenchmarkHelper {
    const FIELD_MAX_SIZE: u32 = 1024;

    fn build_sized_mock(size: u32) -> Release {
        let midds_id = 1;

        Release {
            ean_upc: b"4006381333931".to_vec().try_into().expect("valid EAN"),
            artist: midds_id,
            producers: fill_boundedvec(midds_id, size),
            tracks: fill_boundedvec(midds_id, size),
            distributor_name: fill_boundedvec(b'x', size),
            manufacturer_name: fill_boundedvec(b'x', size),
            cover_contributors: fill_boundedvec(fill_boundedvec(b'x', size), size),
            country: Country::FR,
            title: fill_boundedvec(b'x', size),
            title_aliases: fill_boundedvec(fill_boundedvec(b'x', size), size),
            release_type: ReleaseType::Single,
            format: ReleaseFormat::Cd,
            packaging: ReleasePackaging::SnapCase,
            status: ReleaseStatus::Official,
            date: Date {
                year: 2025,
                month: 5,
                day: 8,
            },
        }
    }

    fn build_mock() -> Release {
        Release {
            ean_upc: b"6024351234567".to_vec().try_into().expect("valid EAN"),
            artist: 1,
            producers: vec![1, 2].try_into().unwrap(),
            tracks: vec![1, 2].try_into().unwrap(),
            distributor_name: b"Universal Music Group".to_vec().try_into().unwrap(),
            manufacturer_name: b"Optimal Media GmbH".to_vec().try_into().unwrap(),
            cover_contributors: vec![
                b"Daniel Caesar".to_vec().try_into().unwrap(),
                b"Alexis Belhumeur".to_vec().try_into().unwrap(),
            ]
            .try_into()
            .unwrap(),
            title: b"After Hours".to_vec().try_into().unwrap(),
            title_aliases: vec![
                "Après Minuit".as_bytes().to_vec().try_into().unwrap(),
                "夜深人静".as_bytes().to_vec().try_into().unwrap(),
            ]
            .try_into()
            .unwrap(),
            release_type: ReleaseType::Lp,
            format: ReleaseFormat::Vinyl10,
            packaging: ReleasePackaging::Digipack,
            status: ReleaseStatus::Official,
            date: Date {
                year: 2020,
                month: 3,
                day: 20,
            },
            country: Country::US,
        }
    }
}
