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
	pallet_prelude::PartyIdentifier,
	party_identifier::{PartyType, Person, PersonGender, PersonType},
};

use super::{fill_boundedvec, BenchmarkHelperT};

pub struct BenchmarkHelper;

impl BenchmarkHelperT<PartyIdentifier> for BenchmarkHelper {
	const FIELD_MAX_SIZE: u32 = 256;

	fn build_mock(size: u32) -> PartyIdentifier {
		let isni = b"0000000106751234".to_vec().try_into().expect("benchmark value valid");
		let ipi = 987654321;

		PartyIdentifier {
			isni,
			ipi,
			party_type: PartyType::Person(Person {
				full_name: fill_boundedvec(b'x', size),
				aliases: fill_boundedvec(fill_boundedvec(b'x', size), size),
				person_type: PersonType::Solo,
				genre: PersonGender::Male,
			}),
		}
	}
}
