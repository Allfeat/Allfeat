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
        release::{
            Ean, ReleaseCoverContributor, ReleaseCoverContributors, ReleaseDistributor,
            ReleaseFormat, ReleaseManufacturer, ReleasePackaging, ReleaseProducers, ReleaseStatus,
            ReleaseTitle, ReleaseTitleAliases, ReleaseTracks, ReleaseType,
        },
        utils::{Country, Date},
    },
};
use parity_scale_codec::Encode;

use super::{BenchmarkHelperT, fill_boundedvec_to_fit};

pub struct BenchmarkHelper;

impl BenchmarkHelperT<Release> for BenchmarkHelper {
    fn build_base() -> Release {
        Release {
            ean_upc: Default::default(),
            artist: 1,
            producers: Default::default(),
            tracks: Default::default(),
            distributor_name: Default::default(),
            manufacturer_name: Default::default(),
            cover_contributors: Default::default(),
            title: Default::default(),
            title_aliases: Default::default(),
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

    fn build_sized(target_size: usize) -> Release {
        let mut midds = Self::build_base_with_checked_target_size(target_size);

        if midds.encoded_size() >= target_size {
            return midds;
        }

        let current_size = midds.encoded_size();
        midds.ean_upc = fill_boundedvec_to_fit(b'a', Ean::bound(), current_size, target_size);
        let current_size = midds.encoded_size();
        midds.producers =
            fill_boundedvec_to_fit(0, ReleaseProducers::bound(), current_size, target_size);
        let current_size = midds.encoded_size();
        midds.tracks = fill_boundedvec_to_fit(0, ReleaseTracks::bound(), current_size, target_size);
        let current_size = midds.encoded_size();
        midds.distributor_name =
            fill_boundedvec_to_fit(b'D', ReleaseDistributor::bound(), current_size, target_size);
        let current_size = midds.encoded_size();
        midds.manufacturer_name = fill_boundedvec_to_fit(
            b'M',
            ReleaseManufacturer::bound(),
            current_size,
            target_size,
        );

        let mut cover_contributor_name = ReleaseCoverContributor::new();
        cover_contributor_name.try_push(b'C').unwrap();
        let current_size = midds.encoded_size();
        // TODO: Make it more precise by correctly filling the name possibilites length
        midds.cover_contributors = fill_boundedvec_to_fit(
            cover_contributor_name,
            ReleaseCoverContributors::bound(),
            current_size,
            target_size,
        );

        let current_size = midds.encoded_size();
        midds.title =
            fill_boundedvec_to_fit(b'T', ReleaseTitle::bound(), current_size, target_size);

        let mut alias_title = ReleaseTitle::new();
        alias_title.try_push(b'T').unwrap();
        let current_size = midds.encoded_size();
        // TODO: Make it more precise by correctly filling the alias title possibilites length
        midds.title_aliases = fill_boundedvec_to_fit(
            alias_title,
            ReleaseTitleAliases::bound(),
            current_size,
            target_size,
        );

        midds
    }
}
