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

use frame_support::sp_runtime::RuntimeDebug;
use parity_scale_codec::{Decode, DecodeWithMemTracking, Encode, MaxEncodedLen};
use scale_info::TypeInfo;

use crate::{
    Midds, MiddsId,
    types::{
        release::{
            Ean, ReleaseCoverContributors, ReleaseDistributor, ReleaseFormat, ReleaseManufacturer,
            ReleasePackaging, ReleaseProducers, ReleaseStatus, ReleaseTitle, ReleaseTitleAliases,
            ReleaseTracks, ReleaseType,
        },
        utils::{Country, Date},
    },
};

#[cfg(feature = "runtime-benchmarks")]
use crate::benchmarking::release::BenchmarkHelper;

/// A MIDDS representing a musical release (album, EP, single, etc.).
/// It contains metadata and references to related MIDDS like tracks, producers, and artist.
///
/// This structure is used to register and manage a complete music release on-chain.
#[derive(
    Clone,
    Eq,
    PartialEq,
    Encode,
    Decode,
    DecodeWithMemTracking,
    TypeInfo,
    MaxEncodedLen,
    RuntimeDebug,
)]
pub struct Release {
    /// EAN or UPC code identifying the release (physical or digital).
    pub ean_upc: Ean,

    /// The main artist MIDDS ID associated with this release.
    pub artist: MiddsId,

    /// List of producer MIDDS IDs who contributed to this release.
    pub producers: ReleaseProducers,

    /// List of track MIDDS IDs that are part of this release.
    pub tracks: ReleaseTracks,

    /// Name of the distributor responsible for the release.
    pub distributor_name: ReleaseDistributor,

    /// Name of the manufacturer responsible for physical production.
    pub manufacturer_name: ReleaseManufacturer,

    /// Contributors to the release cover (designers, photographers, etc.).
    pub cover_contributors: ReleaseCoverContributors,

    /// Official title of the release.
    pub title: ReleaseTitle,

    /// Alternative titles (e.g. translations, acronyms, stylistic variations).
    pub title_aliases: ReleaseTitleAliases,

    /// Type of the release (e.g. LP, EP, Single, Mixtape).
    pub release_type: ReleaseType,

    /// Format of the release medium (e.g. CD, Vinyl, Cassette).
    pub format: ReleaseFormat,

    /// Packaging used for the physical release (e.g. Digipack, Jewel Case).
    pub packaging: ReleasePackaging,

    /// Official status of the release (e.g. Official, Promotional, Remastered).
    pub status: ReleaseStatus,

    /// Release date.
    pub date: Date,

    /// Country where the release was published or made available.
    pub country: Country,
}

impl Midds for Release {
    const NAME: &'static str = "Release";

    #[cfg(feature = "runtime-benchmarks")]
    type BenchmarkHelper = BenchmarkHelper;
}
