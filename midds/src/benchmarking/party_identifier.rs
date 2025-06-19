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
    pallet_prelude::{Artist, PartyIdentifier},
    party_identifier::{Isni, PartyType},
    types::party_identifier::{
        ArtistAlias, ArtistAliases, ArtistFullName, ArtistGender, ArtistType,
    },
};

use super::{BenchmarkHelperT, fill_boundedvec_to_fit};

pub struct BenchmarkHelper;

impl BenchmarkHelperT<PartyIdentifier> for BenchmarkHelper {
    fn build_base() -> PartyIdentifier {
        PartyIdentifier {
            isni: None,
            ipi: Some(0),
            party_type: PartyType::Artist(Artist {
                full_name: Default::default(),
                aliases: Default::default(),
                artist_type: ArtistType::Solo,
                genre: Some(ArtistGender::Neither),
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
        if let PartyType::Artist(ref mut artist) = midds.party_type {
            artist.full_name =
                fill_boundedvec_to_fit(b'F', ArtistFullName::bound(), current_size, target_size);
        }

        let current_size = midds.encoded_size();
        if let PartyType::Artist(ref mut artist) = midds.party_type {
            let mut alias = ArtistAlias::new();
            alias.try_push(b'F').unwrap();

            artist.aliases =
                fill_boundedvec_to_fit(alias, ArtistAliases::bound(), current_size, target_size);
        }

        midds
    }
}
