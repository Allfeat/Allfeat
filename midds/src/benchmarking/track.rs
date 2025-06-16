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
    pallet_prelude::Track,
    types::{
        track::{
            Isrc, TrackContributors, TrackGenreExtras, TrackMasteringPlace, TrackMixingPlace,
            TrackPerformers, TrackProducers, TrackRecordingPlace, TrackTitle, TrackTitleAliases,
            TrackVersion,
        },
        utils::Key,
    },
};

use super::{BenchmarkHelperT, fill_boundedvec_to_fit};
use allfeat_music_genres::GenreId;
use parity_scale_codec::Encode;

pub struct BenchmarkHelper;

impl BenchmarkHelperT<Track> for BenchmarkHelper {
    fn build_base() -> Track {
        Track {
            isrc: Default::default(),
            musical_work: 0,
            artist: 1,
            producers: Default::default(),
            performers: Default::default(),
            contributors: Default::default(),
            title: Default::default(),
            title_aliases: Default::default(),
            recording_year: None,
            genre: None,
            genre_extras: Default::default(),
            version: None,
            duration: None,
            bpm: None,
            key: None,
            recording_place: Default::default(),
            mixing_place: Default::default(),
            mastering_place: Default::default(),
        }
    }

    fn build_sized(target_size: usize) -> Track {
        let mut midds = Self::build_base_with_checked_target_size(target_size);

        if midds.encoded_size() >= target_size {
            return midds;
        }

        midds.recording_year = Some(2019);
        if midds.encoded_size() >= target_size {
            return midds;
        }
        midds.genre = Some(GenreId::Electronic);
        if midds.encoded_size() >= target_size {
            return midds;
        }
        midds.duration = Some(200);
        if midds.encoded_size() >= target_size {
            return midds;
        }
        midds.bpm = Some(172);
        if midds.encoded_size() >= target_size {
            return midds;
        }
        midds.key = Some(Key::Fs);
        if midds.encoded_size() >= target_size {
            return midds;
        }

        // TODO: make the benchmark with the biggest possible type
        midds.version = Some(TrackVersion::Original);
        if midds.encoded_size() >= target_size {
            return midds;
        }

        let current_size = midds.encoded_size();
        midds.isrc = fill_boundedvec_to_fit(b'0', Isrc::bound(), current_size, target_size);
        let current_size = midds.encoded_size();
        midds.producers =
            fill_boundedvec_to_fit(0, TrackProducers::bound(), current_size, target_size);
        let current_size = midds.encoded_size();
        midds.performers =
            fill_boundedvec_to_fit(0, TrackPerformers::bound(), current_size, target_size);
        let current_size = midds.encoded_size();
        midds.contributors =
            fill_boundedvec_to_fit(0, TrackContributors::bound(), current_size, target_size);
        let current_size = midds.encoded_size();
        midds.title = fill_boundedvec_to_fit(b'T', TrackTitle::bound(), current_size, target_size);

        let mut alias_title = TrackTitle::new();
        alias_title.try_push(b'T').unwrap();
        let current_size = midds.encoded_size();
        // TODO: Make it more precise by correctly filling the alias title possibilites length
        midds.title_aliases = fill_boundedvec_to_fit(
            alias_title,
            TrackTitleAliases::bound(),
            current_size,
            target_size,
        );

        let genre = GenreId::Swing;
        let current_size = midds.encoded_size();
        midds.genre_extras =
            fill_boundedvec_to_fit(genre, TrackGenreExtras::bound(), current_size, target_size);
        let current_size = midds.encoded_size();
        midds.recording_place = Some(fill_boundedvec_to_fit(
            b'P',
            TrackRecordingPlace::bound(),
            current_size,
            target_size,
        ));
        let current_size = midds.encoded_size();
        midds.mixing_place = Some(fill_boundedvec_to_fit(
            b'P',
            TrackMixingPlace::bound(),
            current_size,
            target_size,
        ));
        let current_size = midds.encoded_size();
        midds.mastering_place = Some(fill_boundedvec_to_fit(
            b'P',
            TrackMasteringPlace::bound(),
            current_size,
            target_size,
        ));

        midds
    }
}
