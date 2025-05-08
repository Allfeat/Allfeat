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

use frame_support::{sp_runtime::RuntimeDebug, traits::ConstU32, BoundedVec};
use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;

/// Full legal name of a person.
/// Limited to 256 bytes to ensure bounded storage on-chain.
pub type PersonFullName = BoundedVec<u8, ConstU32<256>>;

/// An alias or stage name for a person.
/// Limited to 128 bytes.
pub type PersonAlias = BoundedVec<u8, ConstU32<128>>;

/// Name of a legal entity (e.g., label, publisher, rights organization).
/// Limited to 128 bytes.
pub type EntityName = BoundedVec<u8, ConstU32<128>>;

/// List of aliases for a person.
/// Maximum of 10 aliases, each stored as a `PersonAlias`.
pub type PersonAliases = BoundedVec<PersonAlias, ConstU32<10>>;

/// ISNI (International Standard Name Identifier) code.
/// Fixed length of 16 ASCII characters stored as bytes.
pub type Isni = BoundedVec<u8, ConstU32<16>>;

/// IPI (Interested Parties Information) code.
/// Stored as a `u64`, since the IPI number is an 11-digit identifier.
pub type Ipi = u64;

/// Identifies whether a person is a solo artist or a group.
/// - `Solo`: a single individual.
/// - `Group`: a group of people (band, duo, etc.).
#[derive(Clone, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen, RuntimeDebug)]
pub enum PersonType {
	Solo,
	Group,
}

/// Identifies the the type of an organization.
/// - `Publisher`: responsible for rights and licensing.
/// - `Producer`: oversees the creation of musical works.
/// - `DistribAggr`: distributes or aggregates content to platforms.
#[derive(Clone, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen, RuntimeDebug)]
pub enum EntityType {
	Publisher,
	Producer,
	DistribAggr,
}

/// Declared gender identity of a person.
/// - `Male`: male.
/// - `Female`: female.
/// - `Neither`: unspecified, non-binary, or not disclosed.
#[derive(Clone, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen, RuntimeDebug)]
pub enum PersonGender {
	Male,
	Female,
	Neither,
}
