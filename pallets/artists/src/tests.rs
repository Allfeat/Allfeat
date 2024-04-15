// This file is part of Allfeat.

// Copyright (C) Allfeat (FR) Ltd.
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! # Artists tests.

#![cfg(test)]

use super::*;
use crate::{
	mock::*,
	types::{ArtistAliasOf, UpdatableData},
	Error as ArtistsError,
};
use frame_support::{assert_noop, assert_ok, pallet_prelude::Get};
use genres_registry::ElectronicSubtype;
use parity_scale_codec::{Encode, MaxEncodedLen};
use sp_runtime::{DispatchError::BadOrigin, Saturating};
use sp_std::prelude::Vec;

struct ArtistMock<T: Config> {
	pub main_name: BoundedVec<u8, <T as Config>::MaxNameLen>,
	pub alias: Option<BoundedVec<u8, <T as Config>::MaxNameLen>>,
	pub genres: BoundedVec<MusicGenre, T::MaxGenres>,
	pub description: Option<Vec<u8>>,
	pub assets: BoundedVec<Vec<u8>, T::MaxAssets>,
}

fn to_bounded_alias(str: String) -> ArtistAliasOf<Test> {
	ArtistAliasOf::<Test>::try_from(str.as_bytes().to_vec()).expect("invalid alias test string")
}

fn tester_artist<T: Config>() -> ArtistMock<T> {
	let mut genres = Vec::new();
	genres.push(MusicGenre::Electronic(Some(ElectronicSubtype::House)));

	ArtistMock {
		main_name: b"Tester".to_vec().try_into().unwrap(),
		alias: Some(b"Dark Singer".to_vec().try_into().unwrap()),
		genres: genres.try_into().unwrap(),
		description: Some(b"A simple tester artist.".to_vec()),
		assets: Default::default(),
	}
}

fn expected_artist_cost<T: Config>(artist: &ArtistMock<T>) -> BalanceOf<T> {
	let hash_size = T::Hash::max_encoded_len();

	let name_size = artist.main_name.encoded_size();
	let alias_size = artist.alias.encoded_size();

	let hash_cost = T::ByteDeposit::get().saturating_mul(hash_size.saturated_into());

	let name_cost = T::ByteDeposit::get().saturating_mul(name_size.saturated_into());
	let alias_cost = T::ByteDeposit::get().saturating_mul(alias_size.saturated_into());
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
		.saturating_add(alias_cost)
		.saturating_add(description_cost)
		.saturating_add(assets_cost)
}

#[test]
fn artist_register_works() {
	new_test_ext().execute_with(|| {
		let artist = tester_artist::<Test>();
		let artist_id = 1u64;

		let old_balance = Balances::free_balance(&artist_id);

		assert_ok!(Artists::register(
			RuntimeOrigin::signed(artist_id),
			artist.main_name.clone(),
			artist.alias.clone(),
			artist.genres.clone(),
			artist.description.clone(),
			artist.assets.clone(),
		));

		// Verify register cost
		let new_balance = Balances::free_balance(&artist_id);

		let expected_cost = expected_artist_cost(&artist);

		assert_eq!(new_balance, old_balance - expected_cost);

		// Can't register a second time if already registered
		assert_noop!(
			Artists::register(
				RuntimeOrigin::signed(artist_id),
				artist.main_name,
				artist.alias,
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

		let old_balance = Balances::free_balance(&artist_id);

		assert_ok!(Artists::register(
			RuntimeOrigin::signed(artist_id),
			artist.main_name.clone(),
			artist.alias.clone(),
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
		let new_balance = Balances::free_balance(&artist_id);
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
			artist.alias.clone(),
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

		let old_balance = Balances::free_balance(&artist_id);

		assert_ok!(Artists::unregister(RuntimeOrigin::signed(artist_id)));

		// Deposit has been returned
		let new_balance = Balances::free_balance(&artist_id);
		let expected_cost = expected_artist_cost(&artist);

		assert_eq!(new_balance, old_balance + expected_cost);
	})
}

#[test]
fn artist_update_alias_works() {
	new_test_ext().execute_with(|| {
		let artist = tester_artist::<Test>();
		let artist_id = 1u64;

		assert_ok!(Artists::register(
			RuntimeOrigin::signed(artist_id),
			artist.main_name.clone(),
			artist.alias.clone(),
			artist.genres.clone(),
			artist.description.clone(),
			artist.assets.clone(),
		));

		let new_alias = to_bounded_alias(String::from("new artist alias"));

		assert_ok!(Artists::update(
			RuntimeOrigin::signed(artist_id),
			UpdatableData::<ArtistAliasOf<Test>>::Alias(Some(new_alias)),
		));

		// Can't update if the caller is not a registered artist
		assert_noop!(
			Artists::update(
				RuntimeOrigin::signed(2),
				UpdatableData::<ArtistAliasOf<Test>>::Alias(None),
			),
			Error::<Test>::NotRegistered
		);

		assert_ok!(Artists::update(
			RuntimeOrigin::signed(artist_id),
			UpdatableData::<ArtistAliasOf<Test>>::Alias(None),
		));
	})
}
