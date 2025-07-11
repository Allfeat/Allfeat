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

/// Full legal name of a artist.
/// Limited to 256 bytes to ensure bounded storage on-chain.
pub type ArtistFullName = BoundedVec<u8, ConstU32<256>>;

/// An alias or stage name for a artist.
/// Limited to 128 bytes.
pub type ArtistAlias = BoundedVec<u8, ConstU32<128>>;

/// Name of a legal entity (e.g., label, publisher, rights organization).
/// Limited to 128 bytes.
pub type EntityName = BoundedVec<u8, ConstU32<128>>;

/// List of aliases for a artist.
/// Maximum of 10 aliases, each stored as a `ArtistAlias`.
pub type ArtistAliases = BoundedVec<ArtistAlias, ConstU32<12>>;

/// ISNI (International Standard Name Identifier) code.
/// Fixed length of 16 ASCII characters stored as bytes.
pub type Isni = BoundedVec<u8, ConstU32<16>>;

/// IPI (Interested Parties Information) code.
/// Stored as a `u64`, since the IPI number is an 11-digit identifier.
pub type Ipi = u64;

/// Identifies the type of artist, specifying whether the artist is a solo performer or a collective entity.
///
/// - `Person`: A single individual artist.
/// - `Group`: A group of people, such as a band or duo.
/// - `Orchestra`: A large instrumental ensemble, typically classical.
/// - `Choir`: A group of singers performing together, typically choral music.
/// - `Other`: Any other type of artist not covered by the above categories.
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
pub enum ArtistType {
    Person,
    Group,
    Orchestra,
    Choir,
    Other,
}

/// Identifies the the type of an organization.
/// - `Publisher`: responsible for rights and licensing.
/// - `Producer`: oversees the creation of musical works.
/// - `DistribAggr`: distributes or aggregates content to platforms.
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
pub enum EntityType {
    Publisher,
    Producer,
}

/// Declared gender identity of a artist.
/// - `Male`: male.
/// - `Female`: female.
/// - `Neither`: unspecified, non-binary, or not disclosed.
#[derive(
    Clone,
    PartialEq,
    Eq,
    Encode,
    Decode,
    DecodeWithMemTracking,
    TypeInfo,
    MaxEncodedLen,
    RuntimeDebug,
)]
pub enum ArtistGender {
    Male,
    Female,
    Neither,
}
