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

#[cfg(feature = "runtime-benchmarks")]
use crate::benchmarking::track::BenchmarkHelper;

use crate::{
    Midds, MiddsId,
    types::{
        track::{
            Isrc, TrackBeatsPerMinute, TrackContributors, TrackDuration, TrackGenre,
            TrackGenreExtras, TrackMasteringPlace, TrackMixingPlace, TrackPerformers,
            TrackProducers, TrackRecordYear, TrackRecordingPlace, TrackTitle, TrackTitleAliases,
            TrackVersion,
        },
        utils::Key,
    },
};

/// A Track represents a specific recorded performance or production
/// of a musical work. It links metadata such as contributors,
/// recording details, and identification codes.
#[derive(
    Clone,
    Eq,
    PartialEq,
    Encode,
    Decode,
    TypeInfo,
    DecodeWithMemTracking,
    MaxEncodedLen,
    RuntimeDebug,
)]
pub struct Track {
    /// ISRC (International Standard Recording Code) that uniquely identifies this recording.
    pub isrc: Isrc,

    /// The linked musical work this track is based on (must refer to a registered MIDDS).
    pub musical_work: MiddsId,

    /// Main artist MIDDS identifier (typically the primary performer).
    pub artist: MiddsId,

    /// List of producer MIDDS identifiers who participated in the production.
    pub producers: TrackProducers,

    /// List of performer MIDDS identifiers who contributed to the performance.
    pub performers: TrackPerformers,

    /// Additional contributors (e.g., sound engineers, featured artists).
    pub contributors: TrackContributors,

    /// Main title of the track.
    pub title: TrackTitle,

    /// Optional list of alternative titles for the track.
    pub title_aliases: TrackTitleAliases,

    /// Year the track was recorded (4-digit Gregorian year).
    pub recording_year: Option<TrackRecordYear>,

    /// Primary musical genre associated with the track.
    pub genre: Option<TrackGenre>,

    /// Optional additional genres for more precise classification.
    pub genre_extras: TrackGenreExtras,

    /// Version or type of the track (e.g., Remix, Acoustic, Live).
    pub version: Option<TrackVersion>,

    /// Duration of the track in seconds.
    pub duration: Option<TrackDuration>,

    /// Beats per minute (BPM), representing the tempo of the track.
    pub bpm: Option<TrackBeatsPerMinute>,

    /// Musical key (e.g., C, G#, etc.) the track is in.
    pub key: Option<Key>,

    /// Free-text field indicating where the recording took place.
    pub recording_place: Option<TrackRecordingPlace>,

    /// Free-text field indicating where the mixing of the track occurred.
    pub mixing_place: Option<TrackMixingPlace>,

    /// Free-text field indicating where the mastering of the track occurred.
    pub mastering_place: Option<TrackMasteringPlace>,
}

// Implements the `Midds` trait, marking this struct as a MIDDS object.
impl Midds for Track {
    const NAME: &'static str = "Track";

    #[cfg(feature = "runtime-benchmarks")]
    type BenchmarkHelper = BenchmarkHelper;
}
