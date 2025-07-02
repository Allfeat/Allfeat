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

use frame_support::{
    LOG_TARGET, migrations::VersionedMigration, traits::UncheckedOnRuntimeUpgrade,
};
use midds::{
    pallet_prelude::Track,
    types::{
        track::{TrackGenres, TrackVersion},
        utils::Key,
    },
};
use pallet_midds::MiddsOf;
use sp_core::Get;
use sp_runtime::Weight;

mod legacy {
    use allfeat_music_genres::GenreId;
    use midds::{
        MiddsId,
        types::track::{
            Isrc, TrackBeatsPerMinute, TrackContributors, TrackDuration, TrackMasteringPlace,
            TrackMixingPlace, TrackPerformers, TrackProducers, TrackRecordYear,
            TrackRecordingPlace, TrackTitle, TrackTitleAliases,
        },
    };
    use parity_scale_codec::{Decode, DecodeWithMemTracking, Encode, MaxEncodedLen};
    use scale_info::TypeInfo;
    use sp_core::{ConstU32, RuntimeDebug};
    use sp_runtime::BoundedVec;

    /// Legacy
    pub type TrackGenre = GenreId;

    /// Legacy
    pub type TrackGenreExtras = BoundedVec<GenreId, ConstU32<5>>;

    /// Legacy type
    #[repr(u8)]
    #[derive(
        RuntimeDebug,
        Clone,
        Copy,
        PartialEq,
        Eq,
        Encode,
        Decode,
        DecodeWithMemTracking,
        MaxEncodedLen,
        TypeInfo,
    )]
    pub enum Key {
        C = 0,
        Cs = 1, // C♯ / D♭
        D = 2,
        Ds = 3, // D♯ / E♭
        E = 4,
        F = 5,
        Fs = 6, // F♯ / G♭
        G = 7,
        Gs = 8, // G♯ / A♭
        A = 9,
        As = 10, // A♯ / B♭
        B = 11,
    }

    #[repr(u8)]
    #[derive(
        Clone,
        Copy,
        PartialEq,
        Eq,
        Encode,
        Decode,
        DecodeWithMemTracking,
        TypeInfo,
        MaxEncodedLen,
        RuntimeDebug,
    )]
    pub enum TrackVersion {
        /// Original recording version.
        Original = 0,
        /// Shortened version for radio broadcasting.
        RadioEdit = 1,
        /// Extended version, typically with added sections.
        Extended = 2,
        /// Instrument-only version.
        Instrumental = 3,
        /// Vocals-only version.
        Acapella = 4,
        /// A modified or remixed version by another artist or producer.
        Remix = 5,
        /// A recording of a live performance.
        Live = 6,
        /// An acoustic version, usually unplugged.
        Acoustic = 7,
        /// Early or incomplete version of a track.
        Demo = 8,
        /// Newly recorded version of an existing track.
        ReRecorded = 9,
        /// Different take/version of the same session.
        AlternateTake = 10,
        /// Version recorded with an orchestral arrangement.
        Orchestral = 11,
        /// Karaoke version without lead vocals.
        Karaoke = 12,
        /// Version with explicit lyrics.
        Clean = 13,
        /// Censored or family-safe version.
        Explicit = 14,
        /// TV-friendly version used in broadcast.
        TvTrack = 15,
        /// Dub version, typically with reverb-heavy effects.
        Dub = 16,
        /// Generic edit, purpose-specific.
        Edit = 17,
        /// Mono audio version.
        Mono = 18,
        /// Stereo audio version.
        Stereo = 19,
        /// Rehearsal take, often raw or unpolished.
        Rehearsal = 20,
    }

    /// Legacy
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
}

impl From<legacy::Key> for Key {
    fn from(value: legacy::Key) -> Self {
        match value {
            legacy::Key::C => Key::C,
            legacy::Key::Cs => Key::Cs, // C♯
            legacy::Key::D => Key::D,
            legacy::Key::Ds => Key::Ds, // D♯
            legacy::Key::E => Key::E,
            legacy::Key::F => Key::F,
            legacy::Key::Fs => Key::Fs, // F♯
            legacy::Key::G => Key::G,
            legacy::Key::Gs => Key::Gs, // G♯
            legacy::Key::A => Key::A,
            legacy::Key::As => Key::As, // A♯
            legacy::Key::B => Key::B,
        }
    }
}

impl From<legacy::TrackVersion> for TrackVersion {
    fn from(old: legacy::TrackVersion) -> Self {
        match old {
            legacy::TrackVersion::Original => TrackVersion::Original,
            legacy::TrackVersion::RadioEdit => TrackVersion::RadioEdit,
            legacy::TrackVersion::Extended => TrackVersion::Extended,
            legacy::TrackVersion::Instrumental => TrackVersion::Instrumental,
            legacy::TrackVersion::Acapella => TrackVersion::Acapella,
            legacy::TrackVersion::Remix => TrackVersion::Remix,
            legacy::TrackVersion::Live => TrackVersion::Live,
            legacy::TrackVersion::Acoustic => TrackVersion::Acoustic,
            legacy::TrackVersion::Demo => TrackVersion::Demo,
            legacy::TrackVersion::ReRecorded => TrackVersion::ReRecorded,
            legacy::TrackVersion::AlternateTake => TrackVersion::AlternateTake,
            legacy::TrackVersion::Orchestral => TrackVersion::Orchestral,
            legacy::TrackVersion::Karaoke => TrackVersion::Karaoke,
            legacy::TrackVersion::Clean => TrackVersion::Clean,
            legacy::TrackVersion::Explicit => TrackVersion::Original,
            legacy::TrackVersion::TvTrack => TrackVersion::TvTrack,
            legacy::TrackVersion::Dub => TrackVersion::Dub,
            legacy::TrackVersion::Rehearsal => TrackVersion::Rehearsal,
            _ => TrackVersion::Edit,
        }
    }
}

impl From<legacy::Track> for Track {
    fn from(value: legacy::Track) -> Self {
        let mut genres: TrackGenres = Default::default();

        if let Some(x) = value.genre {
            genres.try_push(x).unwrap()
        }

        Self {
            isrc: value.isrc,
            musical_work: value.musical_work,
            artist: value.artist,
            producers: value.producers,
            performers: value.performers,
            contributors: value.contributors,
            title: value.title,
            title_aliases: value.title_aliases,
            recording_year: value.recording_year,
            genres,
            version: value.version.map(TrackVersion::from),
            duration: value.duration,
            bpm: value.bpm,
            key: value.key.map(Key::from),
            recording_place: value.recording_place,
            mixing_place: value.mixing_place,
            mastering_place: value.mastering_place,
        }
    }
}

pub type TrackV1ToV2<T> = VersionedMigration<
    1,
    2,
    InnerTrackMigrationV1ToV2<T>,
    pallet_midds::Pallet<T, crate::midds::Tracks>,
    <T as frame_system::Config>::DbWeight,
>;

pub struct InnerTrackMigrationV1ToV2<T: pallet_midds::Config<crate::midds::Tracks>>(
    core::marker::PhantomData<T>,
);

impl<T: pallet_midds::Config<crate::midds::Tracks>> UncheckedOnRuntimeUpgrade
    for InnerTrackMigrationV1ToV2<T>
where
    <T as pallet_midds::Config<crate::midds::Tracks>>::MIDDS: From<legacy::Track>,
{
    fn on_runtime_upgrade() -> Weight {
        let mut count = 0;

        MiddsOf::<T, crate::midds::Tracks>::translate::<legacy::Track, _>(|_k, old| {
            count += 1;
            Some(old.into())
        });

        log::info!(
            target: LOG_TARGET,
            "Storage migration v3 for tracks finished.",
        );

        // calculate and return migration weights
        T::DbWeight::get().reads_writes(count as u64 + 1, count as u64 + 1)
    }
}
