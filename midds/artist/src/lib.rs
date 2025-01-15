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

#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo)]
pub enum ArtistEditableField<Hash, NameLimit, FNameLimit, LNameLimit>
where
	NameLimit: Get<u32> + TypeInfo,
	FNameLimit: Get<u32> + TypeInfo,
	LNameLimit: Get<u32> + TypeInfo,
	Hash: HashT,
{
	Name(Option<BoundedVec<u8, NameLimit>>),
	FirstName(Option<BoundedVec<u8, FNameLimit>>),
	LastName(Option<BoundedVec<u8, LNameLimit>>),
	Artwork(Option<Artwork<Hash>>),
	ServicesInfo(ServicesInfoEditableFields),
}

#[derive(Encode, Default, MaxEncodedLen, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo)]
pub struct Artist<Hash, NameLimit, FNameLimit, LNameLimit>
where
	NameLimit: Get<u32> + TypeInfo,
	FNameLimit: Get<u32> + TypeInfo,
	LNameLimit: Get<u32> + TypeInfo,
	Hash: HashT,
{
	pub name: Option<BoundedVec<u8, NameLimit>>,
	pub first_name: Option<BoundedVec<u8, FNameLimit>>,
	pub last_name: Option<BoundedVec<u8, LNameLimit>>,
	pub artwork: Option<Artwork<Hash>>,
	pub services_data: ServicesInfo,
}

impl<H, N, F, L> Midds for Artist<H, N, F, L>
where
	Artist<H, N, F, L>: Encode + Default,
	N: Get<u32> + TypeInfo + PartialEq + Eq + Clone + Default,
	F: Get<u32> + TypeInfo + PartialEq + Eq + Clone + Default,
	L: Get<u32> + TypeInfo + PartialEq + Eq + Clone + Default,
	H: HashT + Default,
{
	type Hash = H;
	type EditableFields = ArtistEditableField<H, N, F, L>;

	fn is_complete(&self) -> bool {
		self.name.is_some() && self.first_name.is_some() && self.last_name.is_some()
	}

	fn hash(&self) -> <H as HashT>::Output {
		let mut bytes = Vec::new();

		bytes.extend_from_slice(&self.name.encode());
		bytes.extend_from_slice(&self.first_name.encode());
		bytes.extend_from_slice(&self.last_name.encode());

		match &self.artwork {
			Some(value) => {
				bytes.push(1);
				bytes.extend_from_slice(&value.integrity_hash.encode());
				bytes.extend_from_slice(&value.location.encode());
			},
			None => bytes.push(0),
		};

		bytes.extend_from_slice(&self.services_data.encode());

		<H as HashT>::hash(&bytes)
	}

	fn update_field(&mut self, data: Self::EditableFields) {
		match data {
			ArtistEditableField::Name(x) => self.name = x,
			ArtistEditableField::FirstName(x) => self.first_name = x,
			ArtistEditableField::LastName(x) => self.last_name = x,
			ArtistEditableField::Artwork(x) => self.artwork = x,
			ArtistEditableField::ServicesInfo(x) => match x {
				ServicesInfoEditableFields::SpotifyId(x) => self.services_data.spotify_id = x,
				ServicesInfoEditableFields::AppleMusicId(x) => self.services_data.am_id = x,
				ServicesInfoEditableFields::DeezerId(x) => self.services_data.deezer_id = x,
				ServicesInfoEditableFields::SoundcloudId(x) => self.services_data.soundcloud_id = x,
				ServicesInfoEditableFields::YoutubeId(x) => self.services_data.youtube_id = x,
			},
		}
	}
}

#[derive(Encode, Default, MaxEncodedLen, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo)]
pub struct ServicesInfo {
	pub spotify_id: Option<SpotifyArtistId>,
	pub am_id: Option<AMArtistId>,
	pub deezer_id: Option<DeezerArtistId>,
	pub soundcloud_id: Option<SoundcloudUserId>,
	pub youtube_id: Option<YoutubeChannelId>,
}

#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo)]
pub enum ServicesInfoEditableFields {
	SpotifyId(Option<SpotifyArtistId>),
	AppleMusicId(Option<AMArtistId>),
	DeezerId(Option<DeezerArtistId>),
	SoundcloudId(Option<SoundcloudUserId>),
	YoutubeId(Option<YoutubeChannelId>),
}
