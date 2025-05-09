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
use parity_scale_codec::{Decode, DecodeWithMemTracking, Encode, MaxEncodedLen};
use scale_info::TypeInfo;

use crate::MiddsId;

use super::track::{TrackTitle, TrackTitleAliases};

/// A 13-digit EAN code used to identify the release (also supports UPC codes).
pub type Ean = BoundedVec<u8, ConstU32<13>>;

/// Title of the release (album, EP, etc.), reusing track title format.
pub type ReleaseTitle = TrackTitle;

/// Alternative titles or translations of the release.
pub type ReleaseTitleAliases = TrackTitleAliases;

/// List of producer MIDDS IDs associated with this release.
pub type ReleaseProducers = BoundedVec<MiddsId, ConstU32<256>>;

/// List of track MIDDS IDs included in this release.
pub type ReleaseTracks = BoundedVec<MiddsId, ConstU32<1024>>;

/// Name of the company or entity responsible for distribution.
pub type ReleaseDistributor = BoundedVec<u8, ConstU32<256>>;

/// Name of the company or entity responsible for manufacturing physical media.
pub type ReleaseManufacturer = BoundedVec<u8, ConstU32<256>>;

/// Name of a person or entity that contributed to the release cover (design, photography, etc.).
pub type ReleaseCoverContributor = BoundedVec<u8, ConstU32<256>>;

/// List of contributors to the release cover.
pub type ReleaseCoverContributors = BoundedVec<ReleaseCoverContributor, ConstU32<64>>;

/// The general type of release based on track count or intent.
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
pub enum ReleaseType {
	/// Long Play album (usually 8+ tracks).
	Lp = 0,
	/// Double album (2 discs or extensive track list).
	DoubleLp = 1,
	/// Extended Play (typically 4–6 tracks).
	Ep = 2,
	/// A standalone track or 2-track release.
	Single = 3,
	/// Informal or promotional compilation, often non-commercial.
	Mixtape = 4,
}

/// The format of the physical or digital medium used for distribution.
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
pub enum ReleaseFormat {
	/// Compact Disc.
	Cd = 0,
	/// Double Compact Disc.
	DoubleCd = 1,
	/// 7-inch vinyl record.
	Vynil7 = 2,
	/// 10-inch vinyl record.
	Vinyl10 = 3,
	/// Audio cassette.
	Cassette = 4,
	/// Digital Versatile Disc containing audio.
	AudioDvd = 5,
}

/// The packaging type used for the physical release.
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
pub enum ReleasePackaging {
	/// Fold-out cardboard packaging.
	Digipack = 0,
	/// Standard plastic CD case.
	JewelCase = 1,
	/// Thin, plastic alternative packaging.
	SnapCase = 2,
}

/// The official status of the release in its publication lifecycle.
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
pub enum ReleaseStatus {
	/// Properly released by the artist or label.
	Official = 0,
	/// Used for marketing or sent to press/radio.
	Promotional = 1,
	/// Reissued at a later date (possibly remastered).
	ReRelease = 2,
	/// Includes bonus content or packaging.
	SpecialEdition = 3,
	/// Improved audio version of an earlier release.
	Remastered = 4,
	/// Unofficial or unauthorized release.
	Bootleg = 5,
	/// Placeholder or unverified metadata.
	PseudoRelease = 6,
	/// Removed shortly after being released.
	Withdrawn = 7,
	/// Intentionally removed from catalog/history.
	Expunged = 8,
	/// Planned but never released.
	Cancelled = 9,
}
