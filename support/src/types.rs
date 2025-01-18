// Copyright (C) 2022-2025 Allfeat.
// SPDX-License-Identifier: GPL-3.0-or-later
// This file is part of Allfeat.

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
	sp_runtime::{traits::Hash as HashT, BoundedVec, RuntimeDebug},
	traits::ConstU32,
};
use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;

pub use ipi::IPINameNumber;
pub use iswc::ISWC;

mod ipi {
	use super::*;
	use frame_support::{ensure, sp_runtime::DispatchError};

	#[derive(
		Encode, Default, Decode, Clone, PartialEq, Eq, RuntimeDebug, MaxEncodedLen, TypeInfo,
	)]
	pub struct IPINameNumber(pub u64);

	impl TryFrom<u64> for IPINameNumber {
		type Error = DispatchError;

		fn try_from(value: u64) -> Result<Self, DispatchError> {
			ensure!(value < 100_000_000_000, DispatchError::Other("invalid IPI number length"));

			Ok(Self(value))
		}
	}
}

mod iswc {
	use super::*;
	use core::str::FromStr;

	#[derive(
		Encode, Default, Decode, Clone, PartialEq, Eq, RuntimeDebug, MaxEncodedLen, TypeInfo,
	)]
	pub struct ISWC {
		group1: u16,     // 3 digits
		group2: u16,     // 3 digits
		group3: u16,     // 3 digits
		check_digit: u8, // 1 digit
	}

	impl ISWC {
		fn validate_check_digit(&self) -> bool {
			let mut sum: u32 = 0;
			let digits = [
				self.group1 / 100,
				(self.group1 / 10) % 10,
				self.group1 % 10,
				self.group2 / 100,
				(self.group2 / 10) % 10,
				self.group2 % 10,
				self.group3 / 100,
				(self.group3 / 10) % 10,
				self.group3 % 10,
			];

			for (i, &digit) in digits.iter().enumerate() {
				sum += digit as u32 * (10 - i as u32);
			}

			let calculated_check_digit = (sum % 10) as u8;
			calculated_check_digit == self.check_digit
		}

		fn parse_iswc_string(s: &str) -> Result<(u16, u16, u16, u8), &'static str> {
			if s.len() != 15 {
				return Err("Invalid length");
			}

			// Vérifie que le format est correct
			if &s[0..2] != "T-" || &s[5..6] != "." || &s[9..10] != "." || &s[13..14] != "-" {
				return Err("Invalid format");
			}

			// Extraire et parser chaque groupe et le check digit
			let group1 = s[2..5].parse::<u16>().map_err(|_| "Invalid group1")?;
			let group2 = s[6..9].parse::<u16>().map_err(|_| "Invalid group2")?;
			let group3 = s[10..13].parse::<u16>().map_err(|_| "Invalid group3")?;
			let check_digit = s[14..15].parse::<u8>().map_err(|_| "Invalid check digit")?;

			Ok((group1, group2, group3, check_digit))
		}
	}

	impl FromStr for ISWC {
		type Err = &'static str;

		fn from_str(s: &str) -> Result<Self, Self::Err> {
			match ISWC::parse_iswc_string(s) {
				Ok((group1, group2, group3, check_digit)) => {
					let iswc = ISWC { group1, group2, group3, check_digit };

					if iswc.validate_check_digit() {
						Ok(iswc)
					} else {
						Err("Invalid check digit")
					}
				},
				Err(e) => Err(e),
			}
		}
	}

	#[cfg(test)]
	mod tests {
		use super::*;

		#[test]
		fn test_valid_iswc() {
			let iswc_str = "T-123.456.789-0";
			let iswc = iswc_str.parse::<ISWC>().expect("Should parse successfully");
			assert_eq!(iswc.group1, 123);
			assert_eq!(iswc.group2, 456);
			assert_eq!(iswc.group3, 789);
			assert_eq!(iswc.check_digit, 0);
			assert!(iswc.validate_check_digit());
		}

		#[test]
		fn test_invalid_check_digit() {
			let iswc_str = "T-123.456.789-9"; // Intentionally wrong check digit
			let result = iswc_str.parse::<ISWC>();
			assert!(result.is_err());
			assert_eq!(result.unwrap_err(), "Invalid check digit");
		}

		#[test]
		fn test_invalid_format() {
			let iswc_str = "T11233445667890"; // Wrong format, missing hyphens and dots
			let result = iswc_str.parse::<ISWC>();
			assert!(result.is_err());
			assert_eq!(result.unwrap_err(), "Invalid format");
		}

		#[test]
		fn test_invalid_length() {
			let iswc_str = "T-123.456.78-0"; // Wrong length (too short)
			let result = iswc_str.parse::<ISWC>();
			assert!(result.is_err());
			assert_eq!(result.unwrap_err(), "Invalid length");
		}
	}
}

pub type MusicalWorkTitle = BoundedVec<u8, ConstU32<128>>;

#[derive(Encode, Default, Decode, Clone, PartialEq, Eq, RuntimeDebug, MaxEncodedLen, TypeInfo)]
pub enum MusicalWorkType {
	#[default]
	Instrumental,
	Song,
}

#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, MaxEncodedLen, TypeInfo)]
/// An Asset expected to not be stored on-chain due to the expected size.
/// The location should then be precised with the integrity hash to ensure the integrity process to
/// verify the asset.
pub struct ExternalAsset<Hash>
where
	Hash: HashT,
{
	pub location: AssetLocation,
	pub integrity_hash: Hash::Output,
}

#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, MaxEncodedLen, TypeInfo)]
/// Indicate where an Asset can be stored.
pub enum AssetLocation {
	/// An IPFS location storing an inner CID version 0.
	IPFSv0(CIDv0),
	/// An IPFS location storing an inner CID version 1.
	IPFSv1(CIDv1),
	Filecoin(CIDv1),
	Arweave(TxID),
	HTTP(HttpUrl),
}

pub type HttpUrl = BoundedVec<u8, ConstU32<1024>>;
pub type CIDv0 = BoundedVec<u8, ConstU32<46>>;

/// The limit accommodates the maximum length of a CIDv1, which is typically 57 bytes when using
/// base32 encoding (1 byte for multibase, 4 bytes for version and codec, 52 bytes for SHA-256
/// hash). The 64-byte limit allows for additional flexibility or other encoding variations.
pub type CIDv1 = BoundedVec<u8, ConstU32<64>>;

/// Arweave Transaction Identifier to retrieve data (43 characters)
pub type TxID = BoundedVec<u8, ConstU32<43>>;

// base-62 ID
pub type SpotifyArtistId = BoundedVec<u8, ConstU32<22>>;

// Apple Music Artist ID (only numbers, can store as u64)
pub type AMArtistId = u64;

// Deezer Artist ID (only numbers, can store as u64)
pub type DeezerArtistId = u64;

// Soundcloud User ID string (User is the word defined for Artists by Soundcloud so we use it here
// too) 25 length max
pub type SoundcloudUserId = BoundedVec<u8, ConstU32<25>>;

// Youtube Channel ID, this is the same between a normal video channel and a Youtube Music profile.
pub type YoutubeChannelId = BoundedVec<u8, ConstU32<24>>;
