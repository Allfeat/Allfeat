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

use allfeat_support::{
	traits::Midds,
	types::{
		AMArtistId, DeezerArtistId, ExternalAsset, SoundcloudUserId, SpotifyArtistId,
		YoutubeChannelId,
	},
};
use frame_support::traits::Get;
use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use sp_runtime::{traits::Hash as HashT, BoundedVec, RuntimeDebug};

pub type Artwork<Hash> = ExternalAsset<Hash>;

#[derive(Encode, MaxEncodedLen, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo)]
pub struct Artist<Identifier, AccountId, BlockNumber, Hash, NameLimit, FNameLimit, LNameLimit>
where
	NameLimit: Get<u32>,
	FNameLimit: Get<u32>,
	LNameLimit: Get<u32>,
	Hash: HashT,
{
	pub identifier: Identifier,
	pub depositor: AccountId,
	pub registered_at: BlockNumber,
	pub name: BoundedVec<u8, NameLimit>,
	pub first_name: BoundedVec<u8, FNameLimit>,
	pub last_name: BoundedVec<u8, LNameLimit>,
	pub artwork: Option<Artwork<Hash>>,
	pub services_data: ServicesInfo,
	// TODO: main genre ?
}

impl<I, AI, BN, H, N, F, L> Midds<I, AI> for Artist<I, AI, BN, H, N, F, L>
where
	Artist<I, AI, BN, H, N, F, L>: Encode,
	N: Get<u32>,
	F: Get<u32>,
	L: Get<u32>,
	H: HashT,
{
	fn depositor(self) -> AI {
		self.depositor
	}

	fn identifier(self) -> I {
		self.identifier
	}

	fn total_bytes(&self) -> u32 {
		self.encoded_size() as u32
	}
}

#[derive(Encode, MaxEncodedLen, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo)]
pub struct ServicesInfo {
	pub spotify_id: Option<SpotifyArtistId>,
	pub am_id: Option<AMArtistId>,
	pub deezer_id: Option<DeezerArtistId>,
	pub soundcloud_id: Option<SoundcloudUserId>,
	pub youtube_id: Option<YoutubeChannelId>,
}
