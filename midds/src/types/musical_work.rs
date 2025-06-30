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

use frame_support::{BoundedVec, sp_runtime::RuntimeDebug, traits::ConstU32};
use parity_scale_codec::{Decode, DecodeWithMemTracking, Encode, MaxEncodedLen};
use scale_info::TypeInfo;

use crate::MiddsId;

/// The official title of the musical work, limited to 256 bytes.
pub type MusicalWorkTitle = BoundedVec<u8, ConstU32<256>>;

/// The year the musical work was created (Gregorian year).
pub type MusicalWorkCreationYear = u16;

/// The tempo of the work in beats per minute (BPM).
pub type MusicalWorkBpm = u16;

/// List of participants involved in the creation of the musical work.
/// Each participant includes their MIDDS ID and their role.
pub type MusicalWorkParticipants = BoundedVec<Participant, ConstU32<512>>;

/// International Standard Musical Work Code (ISWC) – max 11 characters.
pub type Iswc = BoundedVec<u8, ConstU32<11>>;

/// A collection of references to other musical works this work is derived from.
/// Used in medleys, mashups, and adaptations.
pub type DerivedWorks = BoundedVec<MiddsId, ConstU32<512>>;

/// Opus number of a classical work.
/// Example: "Op. 27 No. 2"
pub type Opus = BoundedVec<u8, ConstU32<128>>;

/// Catalog number referencing a thematic index (e.g., BWV, K., Hob.).
/// Example: "BWV 1007", "K. 550"
pub type CatalogNumber = BoundedVec<u8, ConstU32<128>>;

/// Enumeration of the types of musical works.
#[derive(
    Clone,
    PartialEq,
    Eq,
    Encode,
    Decode,
    DecodeWithMemTracking,
    MaxEncodedLen,
    RuntimeDebug,
    TypeInfo,
)]
pub enum MusicalWorkType {
    /// A standalone, original composition.
    Original,
    /// A combination of multiple existing works (referenced via their IDs).
    Medley(DerivedWorks),
    /// A mixed version using components of existing works.
    Mashup(DerivedWorks),
    /// A modified version of existing work(s), with a reference to the adapted work.
    Adaptation(MiddsId),
}

/// Describes a participant in the creation of the musical work.
#[derive(
    Clone,
    PartialEq,
    Eq,
    Encode,
    Decode,
    DecodeWithMemTracking,
    MaxEncodedLen,
    RuntimeDebug,
    TypeInfo,
)]
pub struct Participant {
    /// MIDDS ID reference of the person or entity.
    pub id: MiddsId,
    /// The specific role this participant played in the work.
    pub role: ParticipantRole,
}

/// Enum representing the creative or editorial role a participant had in the musical work.
#[derive(
    Clone,
    PartialEq,
    Eq,
    Encode,
    Decode,
    DecodeWithMemTracking,
    MaxEncodedLen,
    RuntimeDebug,
    TypeInfo,
)]
pub enum ParticipantRole {
    /// Original author of the lyrics.
    Author,
    /// Composer of the music.
    Composer,
    /// Arranger of an existing work (e.g. orchestration).
    Arranger,
    /// Adapter of music or lyrics from original sources.
    Adapter,
    /// Publisher who reviewed or modified the work in a non-creative capacity.
    Publisher,
}

/// Struct representing some additional informations if the work is a classical one.
#[derive(
    Clone,
    Default,
    PartialEq,
    Eq,
    Encode,
    Decode,
    DecodeWithMemTracking,
    MaxEncodedLen,
    RuntimeDebug,
    TypeInfo,
)]
pub struct ClassicalInfo {
    pub opus: Option<Opus>,
    pub catalog_number: Option<CatalogNumber>,
    pub number_of_voices: Option<u16>,
}
