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

use parity_scale_codec::Encode;

use crate::{
    pallet_prelude::PartyIdentifier,
    party_identifier::{
        Isni, PartyType, Person, PersonAliases, PersonFullName, PersonGender, PersonType,
    },
    types::party_identifier::PersonAlias,
};

use super::{BenchmarkHelperT, fill_boundedvec_to_fit};

pub struct BenchmarkHelper;

impl BenchmarkHelperT<PartyIdentifier> for BenchmarkHelper {
    fn build_base() -> PartyIdentifier {
        PartyIdentifier {
            isni: None,
            ipi: Some(0),
            party_type: PartyType::Person(Person {
                full_name: Default::default(),
                aliases: Default::default(),
                person_type: PersonType::Solo,
                genre: Some(PersonGender::Male),
            }),
        }
    }

    fn build_sized(target_size: usize) -> PartyIdentifier {
        let mut midds = Self::build_base_with_checked_target_size(target_size);

        if midds.encoded_size() >= target_size {
            return midds;
        }

        let current_size = midds.encoded_size();
        midds.isni = Some(fill_boundedvec_to_fit(
            b'a',
            Isni::bound(),
            current_size,
            target_size,
        ));

        let current_size = midds.encoded_size();
        if let PartyType::Person(ref mut person) = midds.party_type {
            person.full_name =
                fill_boundedvec_to_fit(b'F', PersonFullName::bound(), current_size, target_size);
        }

        let current_size = midds.encoded_size();
        if let PartyType::Person(ref mut person) = midds.party_type {
            let mut alias = PersonAlias::new();
            alias.try_push(b'F').unwrap();

            // TODO: Make it more precise by correctly filling the alias possibilites length
            person.aliases =
                fill_boundedvec_to_fit(alias, PersonAliases::bound(), current_size, target_size);
        }

        midds
    }
}
