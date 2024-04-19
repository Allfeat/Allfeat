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

//! # Artists tests.

#![cfg(test)]

use super::*;
use crate::{
	mock::*,
	types::{ArtistType, ArtistTypeFlag, ExtraArtistTypes},
	Error as ArtistsError,
};
use frame_support::{assert_noop, assert_ok};
use genres_registry::ElectronicSubtype;
use parity_scale_codec::{Encode, MaxEncodedLen};
use sp_runtime::{DispatchError::BadOrigin, Saturating};

struct ArtistMock<T: Config> {
	pub main_name: BoundedVec<u8, <T as Config>::MaxNameLen>,
	pub main_type: ArtistType,
	pub extra_types: ExtraArtistTypes,
	pub genres: BoundedVec<MusicGenre, T::MaxGenres>,
	pub description: Option<Vec<u8>>,
	pub assets: BoundedVec<Vec<u8>, T::MaxAssets>,
}

fn tester_artist<T: Config>() -> ArtistMock<T> {
	let mut extra_types = ExtraArtistTypes::default();
	let genres = vec![MusicGenre::Electronic(Some(ElectronicSubtype::House))];

	extra_types.0.insert(ArtistTypeFlag::DiscJokey);
	extra_types.0.insert(ArtistTypeFlag::Instrumentalist);

	ArtistMock {
		main_name: b"Tester".to_vec().try_into().unwrap(),
		main_type: ArtistType::Singer,
		extra_types,
		genres: genres.try_into().unwrap(),
		description: Some(b"A simple tester artist.".to_vec()),
		assets: Default::default(),
	}
}

fn expected_artist_cost<T: Config>(artist: &ArtistMock<T>) -> BalanceOf<T> {
	let hash_size = T::Hash::max_encoded_len();

	let name_size = artist.main_name.encoded_size();

	let hash_cost = T::ByteDeposit::get().saturating_mul(hash_size.saturated_into());

	let name_cost = T::ByteDeposit::get().saturating_mul(name_size.saturated_into());
	let description_cost = match artist.description {
		Some(_) => hash_cost,
		None => 0u32.saturated_into(),
	};
	let assets_cost: BalanceOf<T> = artist
		.assets
		.iter()
		.fold(0u32.saturated_into(), |acc, _| acc.saturating_add(hash_cost));

	T::BaseDeposit::get()
		.saturating_add(name_cost)
		.saturating_add(description_cost)
		.saturating_add(assets_cost)
}

#[test]
fn artist_register_works() {
	new_test_ext().execute_with(|| {
		let artist = tester_artist::<Test>();
		let artist_id = 1u64;

		let old_balance = Balances::free_balance(artist_id);

		assert_ok!(Artists::register(
			RuntimeOrigin::signed(artist_id),
			artist.main_name.clone(),
			artist.main_type,
			artist.extra_types.clone(),
			artist.genres.clone(),
			artist.description.clone(),
			artist.assets.clone(),
		));

		// Verify register cost
		let new_balance = Balances::free_balance(artist_id);

		let expected_cost = expected_artist_cost(&artist);

		assert_eq!(new_balance, old_balance - expected_cost);

		// Can't register a second time if already registered
		assert_noop!(
			Artists::register(
				RuntimeOrigin::signed(artist_id),
				artist.main_name,
				artist.main_type,
				artist.extra_types,
				artist.genres,
				artist.description,
				artist.assets,
			),
			ArtistsError::<Test>::AlreadyRegistered
		);
	})
}

#[test]
fn artist_force_unregister_works() {
	new_test_ext().execute_with(|| {
		let artist = tester_artist::<Test>();
		let artist_id = 1u64;

		let old_balance = Balances::free_balance(artist_id);

		assert_ok!(Artists::register(
			RuntimeOrigin::signed(artist_id),
			artist.main_name.clone(),
			artist.main_type,
			artist.extra_types.clone(),
			artist.genres.clone(),
			artist.description.clone(),
			artist.assets.clone(),
		));

		// Can't force unregister if not Root origin
		assert_noop!(
			Artists::force_unregister(RuntimeOrigin::signed(artist_id), artist_id),
			BadOrigin
		);

		assert_ok!(Artists::force_unregister(RuntimeOrigin::root(), artist_id,));

		// Deposit has been returned
		let new_balance = Balances::free_balance(artist_id);
		let expected_cost = expected_artist_cost(&artist);

		assert_eq!(new_balance, old_balance - expected_cost);
	})
}

#[test]
fn artist_unregister_works() {
	new_test_ext().execute_with(|| {
		let artist = tester_artist::<Test>();
		let artist_id = 1u64;

		// Can't unregister if not registered
		assert_noop!(
			Artists::unregister(RuntimeOrigin::signed(artist_id)),
			Error::<Test>::NotRegistered
		);

		assert_ok!(Artists::register(
			RuntimeOrigin::signed(artist_id),
			artist.main_name.clone(),
			artist.main_type,
			artist.extra_types.clone(),
			artist.genres.clone(),
			artist.description.clone(),
			artist.assets.clone(),
		));

		// Can't unregister if not waited the unregister period
		assert_noop!(
			Artists::unregister(RuntimeOrigin::signed(artist_id)),
			Error::<Test>::PeriodNotPassed
		);

		let unregister_cd: u32 = <Test as Config>::UnregisterPeriod::get();
		frame_system::Pallet::<Test>::set_block_number(unregister_cd.saturated_into());

		let old_balance = Balances::free_balance(artist_id);

		assert_ok!(Artists::unregister(RuntimeOrigin::signed(artist_id)));

		// Deposit has been returned
		let new_balance = Balances::free_balance(artist_id);
		let expected_cost = expected_artist_cost(&artist);

		assert_eq!(new_balance, old_balance + expected_cost);
	})
}
