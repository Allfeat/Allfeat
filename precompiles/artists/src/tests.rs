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

use crate::{mock::*, ArtistData, ArtistOf, ArtistType, DescriptionPreimage};
use frame_support::assert_ok;
// use pallet_evm::Call as EvmCall;
use precompile_utils::testing::*;
use sp_core::H160;
use sp_runtime::traits::Hash;

fn precompiles() -> Precompiles<Runtime> {
	PrecompilesValue::get()
}

/*
fn evm_call(source: impl Into<H160>, input: Vec<u8>) -> EvmCall<Runtime> {
	EvmCall::call {
		source: source.into(),
		target: Precompile1.into(),
		input,
		value: U256::zero(),
		gas_limit: u64::max_value(),
		max_fee_per_gas: 0.into(),
		max_priority_fee_per_gas: Some(U256::zero()),
		nonce: None,
		access_list: Vec::new(),
	}
}
*/

#[test]
fn test_solidity_interface_has_all_function_selectors_documented_and_implemented() {
	check_precompile_implements_solidity_interfaces(&["Artists.sol"], PCall::supports_selector)
}

#[test]
fn test_artists_returns_none_if_not_an_artist() {
	ExtBuilder::default()
		.with_balances(vec![(Alice.into(), 100_000), (Bob.into(), 100_000)])
		.build()
		.execute_with(|| {
			precompiles()
				.prepare_test(
					Alice,
					Precompile1,
					PCall::get_artist { account: H160::from(Alice).into() },
				)
				.expect_no_logs()
				.execute_returns(ArtistOf::<Runtime>::default());
		})
}

#[test]
fn test_artists_returns_valid_data_for_artist_data() {
	ExtBuilder::default()
		.with_balances(vec![(Alice.into(), 100_000), (Bob.into(), 100_000)])
		.build()
		.execute_with(|| {
			let mut extra_types = pallet_artists::types::ExtraArtistTypes::default();
			extra_types.0.insert(pallet_artists::types::ArtistTypeFlag::Director);
			extra_types.0.insert(pallet_artists::types::ArtistTypeFlag::Producer);

			assert_ok!(Artists::register(
				RuntimeOrigin::signed(Bob.into()),
				vec![0x01].try_into().expect("succeeds"),
				pallet_artists::types::ArtistType::Singer,
				extra_types,
				vec![].try_into().expect("succeeds"),
				Some(vec![0x03]),
				vec![vec![0x04], vec![0x05]].try_into().expect("succeeds")
			));

			precompiles()
				.prepare_test(
					Bob,
					Precompile1,
					PCall::get_artist { account: H160::from(Bob).into() },
				)
				.expect_no_logs()
				.execute_returns(ArtistOf::<Runtime> {
					is_artist: true,
					data: ArtistData {
						owner: H160::from(Bob).into(),
						registered_at: 1,
						main_name: vec![0x01].try_into().expect("succeeds"),
						main_type: ArtistType::Singer,
						extra_types: vec![ArtistType::Producer, ArtistType::Director],
						genres: vec![],
						description: DescriptionPreimage {
							has_preimage: true,
							preimage: <Runtime as frame_system::Config>::Hashing::hash(
								vec![0x03u8].as_slice(),
							)
							.into(),
						},
						assets: vec![
							<Runtime as frame_system::Config>::Hashing::hash(
								vec![0x04u8].as_slice(),
							)
							.into(),
							<Runtime as frame_system::Config>::Hashing::hash(
								vec![0x05u8].as_slice(),
							)
							.into(),
						],
					},
				})
		})
}
