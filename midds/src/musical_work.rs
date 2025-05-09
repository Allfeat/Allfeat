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
	types::{
		musical_work::{
			Iswc, MusicalWorkBpm, MusicalWorkCreationYear, MusicalWorkParticipants,
			MusicalWorkTitle, MusicalWorkType,
		},
		utils::{Key, Language, Mode},
	},
	Midds,
};

#[cfg(feature = "runtime-benchmarks")]
use crate::benchmarking::musical_work::BenchmarkHelper;

/// Core data structure representing a musical work (composition).
///
/// A musical work encapsulates metadata about an original or derived
/// musical creation, including its participants, structure, and identity.
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
pub struct MusicalWork {
	/// The ISWC (International Standard Musical Work Code) uniquely identifying the work.
	pub iswc: Iswc,

	/// The title of the musical work.
	pub title: MusicalWorkTitle,

	/// The year the work was created (4-digit Gregorian year).
	pub creation_year: MusicalWorkCreationYear,

	/// Indicates whether the work is instrumental (i.e., without lyrics).
	pub instrumental: bool,

	/// The optional language of the lyrics (if any).
	pub language: Option<Language>,

	/// Optional tempo in beats per minute (BPM).
	pub bpm: Option<MusicalWorkBpm>,

	/// Optional musical key of the work (e.g., C, G#, etc.).
	pub key: Option<Key>,

	/// Optional musical mode (e.g., major or minor).
	pub mode: Option<Mode>,

	/// Type of the musical work (original, medley, mashup, or adaptation).
	pub work_type: MusicalWorkType,

	/// List of contributors to the work, along with their roles.
	pub participants: MusicalWorkParticipants,
}

/// Trait implementation allowing the musical work to be used
/// as a MIDDS (Music Industry Decentralized Data Structure).
impl Midds for MusicalWork {
	#[cfg(feature = "runtime-benchmarks")]
	type BenchmarkHelper = BenchmarkHelper;
}
