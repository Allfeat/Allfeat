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

use allfeat_music_genres::GenreId;
use frame_support::{BoundedVec, sp_runtime::RuntimeDebug, traits::ConstU32};
use parity_scale_codec::{Decode, DecodeWithMemTracking, Encode, MaxEncodedLen};
use scale_info::TypeInfo;

use crate::MiddsId;

/// The ISRC (International Standard Recording Code) for uniquely identifying a recording.
pub type Isrc = BoundedVec<u8, ConstU32<12>>;

/// The main title of the track.
pub type TrackTitle = BoundedVec<u8, ConstU32<256>>;

/// Alternative titles or aliases for the track.
pub type TrackTitleAliases = BoundedVec<TrackTitle, ConstU32<16>>;

/// The year the track was recorded (4-digit Gregorian year).
pub type TrackRecordYear = u16;

/// Additional genres that describe the track.
pub type TrackGenres = BoundedVec<GenreId, ConstU32<5>>;

/// Total duration of the track in seconds.
pub type TrackDuration = u16;

/// Beats per minute (BPM) representing the tempo of the track.
pub type TrackBeatsPerMinute = u16;

/// List of producer MIDDS identifiers involved in the track.
pub type TrackProducers = BoundedVec<MiddsId, ConstU32<64>>;

/// List of performer MIDDS identifiers (e.g., singers, instrumentalists).
pub type TrackPerformers = BoundedVec<MiddsId, ConstU32<256>>;

/// List of additional contributors (e.g., engineers, featured artists).
pub type TrackContributors = BoundedVec<MiddsId, ConstU32<256>>;

/// Free-text field indicating the place where the recording took place.
pub type TrackRecordingPlace = BoundedVec<u8, ConstU32<256>>;

/// Free-text field indicating where the mixing of the track occurred.
pub type TrackMixingPlace = BoundedVec<u8, ConstU32<256>>;

/// Free-text field indicating where the mastering of the track was performed.
pub type TrackMasteringPlace = BoundedVec<u8, ConstU32<256>>;

/// Enumeration of common versions or variants of a track.
/// Helps categorize the nature or use of a specific recording.
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
