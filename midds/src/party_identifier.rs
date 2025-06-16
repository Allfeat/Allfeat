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

use crate::Midds;
use frame_support::sp_runtime::RuntimeDebug;
use parity_scale_codec::{Decode, DecodeWithMemTracking, Encode, MaxEncodedLen};
use scale_info::TypeInfo;

pub use super::types::party_identifier::{
    EntityName, EntityType, Ipi, Isni, PersonAliases, PersonFullName, PersonGender, PersonType,
};

#[cfg(feature = "runtime-benchmarks")]
use crate::benchmarking::party_identifier::BenchmarkHelper;

/// Core struct used to uniquely identify a music industry party (either a person or an entity)
/// as a MIDDS.
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
pub struct PartyIdentifier {
    /// ISNI identifier (max 16 characters).
    pub isni: Isni,
    /// IPI identifier (11-digit u64).
    pub ipi: Ipi,
    /// Variant defining if the party is a `Person` or an `Entity` with data.
    pub party_type: PartyType,
}

// Implements the `Midds` trait to integrate this type into the MIDDS protocol.
impl Midds for PartyIdentifier {
    const NAME: &'static str = "Party Identifier";

    #[cfg(feature = "runtime-benchmarks")]
    type BenchmarkHelper = BenchmarkHelper;
}

/// Enum representing whether a party is a person or an entity.
///
/// Used to branch logic and data structure based on the nature of the party.
#[derive(
    RuntimeDebug,
    Clone,
    PartialEq,
    Eq,
    Encode,
    Decode,
    DecodeWithMemTracking,
    MaxEncodedLen,
    TypeInfo,
)]
pub enum PartyType {
    Person(Person),
    Entity(Entity),
}

/// Data structure representing an individual involved in, as example, music production or rights.
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
pub struct Person {
    /// Legal name of the person.
    pub full_name: PersonFullName,
    /// Alternative names/stage names.
    pub aliases: PersonAliases,
    /// Indicates if this is a solo artist or a group.
    pub person_type: PersonType,
    /// Declared gender identity.
    pub genre: Option<PersonGender>,
}

/// Data structure representing an organization or company involved in music industry.
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
pub struct Entity {
    /// Entity Name.
    pub name: EntityName,
    /// The role played by the organization (e.g., publisher, producer).
    pub entity_type: EntityType,
}
