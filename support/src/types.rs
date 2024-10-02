// This file is part of Allfeat.

// Copyright (C) 2022-2024 Allfeat.
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

use frame_support::traits::ConstU32;
use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use sp_runtime::{traits::Hash as HashT, BoundedVec, RuntimeDebug};

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
