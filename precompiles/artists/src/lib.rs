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

#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::dispatch::{GetDispatchInfo, PostDispatchInfo};
use num_enum::{IntoPrimitive, TryFromPrimitive};
use pallet_evm::AddressMapping;
use parity_scale_codec::Encode;
use precompile_utils::{
	prelude::*,
	solidity::{
		codec::{Reader, Writer},
		Codec,
	},
};
use sp_core::{MaxEncodedLen, H160, H256, U256};
use sp_runtime::{traits::Dispatchable, SaturatedConversion};
use sp_std::{marker::PhantomData, prelude::*};
extern crate alloc;

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

/// Precompile exposing pallet_artists as an EVM-compatible interface.
pub struct ArtistsPrecompile<Runtime>(PhantomData<Runtime>);

type ArtistOf<T> = Artist<<T as pallet_artists::Config>::MaxNameLen>;
type ArtistDataOf<T> = ArtistData<<T as pallet_artists::Config>::MaxNameLen>;

#[precompile_utils::precompile]
#[precompile::test_concrete_types(mock::Runtime)]
impl<Runtime> ArtistsPrecompile<Runtime>
where
	Runtime: pallet_artists::Config + pallet_evm::Config + pallet_timestamp::Config,
	Runtime::AccountId: Into<H160>,
	Runtime::Hash: From<H256>,
	Runtime::RuntimeCall: Dispatchable<PostInfo = PostDispatchInfo> + GetDispatchInfo,
	Runtime::RuntimeCall: From<pallet_artists::Call<Runtime>>,
	<Runtime::RuntimeCall as Dispatchable>::RuntimeOrigin: From<Option<Runtime::AccountId>>,
	<Runtime as pallet_timestamp::Config>::Moment: Into<U256>,
	H256: From<<Runtime as frame_system::Config>::Hash>,
{
	#[precompile::public("get_artist(address)")]
	#[precompile::view]
	fn get_artist(
		handle: &mut impl PrecompileHandle,
		account: Address,
	) -> EvmResult<ArtistOf<Runtime>> {
		handle.record_db_read::<Runtime>(pallet_artists::Artist::<Runtime>::max_encoded_len())?;

		let account: H160 = account.into();
		let account: Runtime::AccountId = Runtime::AddressMapping::into_account_id(account);

		let result = pallet_artists::Pallet::<Runtime>::get_artist_by_id(&account);
		Ok(Self::artist_to_output(result)?)
	}

	fn artist_to_output(
		artist: Option<pallet_artists::Artist<Runtime>>,
	) -> MayRevert<ArtistOf<Runtime>> {
		if artist.is_none() {
			return Ok(ArtistOf::<Runtime>::default());
		}

		let artist = artist.expect("none case checked above; qed");
		let artist_data: ArtistDataOf<Runtime> = ArtistData {
			owner: Address::from(artist.owner().clone().into()),
			registered_at: (*artist.registered_at()).saturated_into(),
			main_name: artist.main_name().to_vec().into(),
			main_type: (*artist.main_type()).into(),
			extra_types: FromBitFlagsWrapper::<pallet_artists::types::ExtraArtistTypes>::from(
				artist.extra_types(),
			),
			genres: artist.genres().iter().map(|genre| genre.encode()).collect(),
			description: match artist.description() {
				Some(x) => DescriptionPreimage { has_preimage: true, preimage: (*x).into() },
				None => DescriptionPreimage { has_preimage: false, preimage: Default::default() },
			},
			assets: artist.assets().iter().map(|hash| (*hash).into()).collect(),
		};
		let evm_artist: ArtistOf<Runtime> = Artist { is_artist: true, data: artist_data };

		Ok(evm_artist)
	}
}

#[derive(Eq, PartialEq, Default, Debug, solidity::Codec)]
struct DescriptionPreimage {
	has_preimage: bool,
	preimage: H256,
}

#[derive(Eq, PartialEq, Default, Debug, IntoPrimitive, TryFromPrimitive)]
#[repr(u16)]
enum ArtistType {
	#[default]
	Singer,
	Instrumentalist,
	Composer,
	Lyricist,
	Producer,
	DiscJokey,
	Conductor,
	Arranger,
	Engineer,
	Director,
}

impl Codec for ArtistType {
	fn read(reader: &mut Reader) -> MayRevert<ArtistType> {
		let value256: U256 =
			reader.read().map_err(|_| RevertReason::read_out_of_bounds(Self::signature()))?;

		let value_as_u16: u16 = value256
			.try_into()
			.map_err(|_| RevertReason::value_is_too_large(Self::signature()))?;

		value_as_u16
			.try_into()
			.map_err(|_| RevertReason::custom("Unknown artist type").into())
	}

	fn write(writer: &mut Writer, value: Self) {
		let value_as_u16: u16 = value.into();
		U256::write(writer, value_as_u16.into());
	}

	fn has_static_size() -> bool {
		true
	}

	fn signature() -> String {
		"uint16".into()
	}
}

impl From<pallet_artists::types::ArtistTypeFlag> for ArtistType {
	fn from(value: pallet_artists::types::ArtistTypeFlag) -> Self {
		match value {
			pallet_artists::types::ArtistTypeFlag::Singer => Self::Singer,
			pallet_artists::types::ArtistTypeFlag::Instrumentalist => Self::Instrumentalist,
			pallet_artists::types::ArtistTypeFlag::Composer => Self::Composer,
			pallet_artists::types::ArtistTypeFlag::Lyricist => Self::Lyricist,
			pallet_artists::types::ArtistTypeFlag::Producer => Self::Producer,
			pallet_artists::types::ArtistTypeFlag::DiscJokey => Self::DiscJokey,
			pallet_artists::types::ArtistTypeFlag::Conductor => Self::Conductor,
			pallet_artists::types::ArtistTypeFlag::Arranger => Self::Arranger,
			pallet_artists::types::ArtistTypeFlag::Engineer => Self::Engineer,
			pallet_artists::types::ArtistTypeFlag::Director => Self::Director,
		}
	}
}

impl From<pallet_artists::types::ArtistType> for ArtistType {
	fn from(value: pallet_artists::types::ArtistType) -> Self {
		match value {
			pallet_artists::types::ArtistType::Singer => Self::Singer,
			pallet_artists::types::ArtistType::Instrumentalist => Self::Instrumentalist,
			pallet_artists::types::ArtistType::Composer => Self::Composer,
			pallet_artists::types::ArtistType::Lyricist => Self::Lyricist,
			pallet_artists::types::ArtistType::Producer => Self::Producer,
			pallet_artists::types::ArtistType::DiscJokey => Self::DiscJokey,
			pallet_artists::types::ArtistType::Conductor => Self::Conductor,
			pallet_artists::types::ArtistType::Arranger => Self::Arranger,
			pallet_artists::types::ArtistType::Engineer => Self::Engineer,
			pallet_artists::types::ArtistType::Director => Self::Director,
		}
	}
}

trait FromBitFlagsWrapper<BitflagWrapper> {
	fn from(value: &BitflagWrapper) -> Vec<Self>
	where
		Self: Sized;
}

impl FromBitFlagsWrapper<pallet_artists::types::ExtraArtistTypes> for ArtistType {
	fn from(value: &pallet_artists::types::ExtraArtistTypes) -> Vec<Self> {
		let mut v: Vec<Self> = Vec::new();
		value.0.iter().for_each(|x| v.push(x.into()));
		v
	}
}

#[derive(Eq, PartialEq, Debug, solidity::Codec)]
struct ArtistData<NameLen> {
	owner: Address,
	registered_at: u32,
	main_name: BoundedBytes<NameLen>,
	main_type: ArtistType,
	extra_types: Vec<ArtistType>,
	// Genres are stored as the scale encoded enum
	genres: Vec<Vec<u8>>,
	description: DescriptionPreimage,
	assets: Vec<H256>,
}

impl<T> Default for ArtistData<T> {
	fn default() -> Self {
		Self {
			owner: Default::default(),
			registered_at: Default::default(),
			main_type: Default::default(),
			extra_types: Default::default(),
			main_name: BoundedBytes::from(vec![]),
			genres: Default::default(),
			description: Default::default(),
			assets: Default::default(),
		}
	}
}

#[derive(Debug, solidity::Codec)]
struct Artist<NameLen> {
	is_artist: bool,
	data: ArtistData<NameLen>,
}

impl<T> Default for Artist<T> {
	fn default() -> Self {
		Self { is_artist: Default::default(), data: Default::default() }
	}
}
