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

use crate::{
	release::Release,
	types::{
		release::{ReleaseFormat, ReleasePackaging, ReleaseStatus, ReleaseType},
		utils::{Country, Date},
	},
};

use super::{fill_boundedvec, BenchmarkHelperT};

pub struct BenchmarkHelper;

impl BenchmarkHelperT<Release> for BenchmarkHelper {
	const FIELD_MAX_SIZE: u32 = 1024;

	fn build_mock(size: u32) -> Release {
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
			date: Date { year: 2025, month: 5, day: 8 },
		}
	}
}
