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

use crate::{mock::*, NftsSwapPrecompileCall};
use frame_support::assert_ok;
use frame_system::pallet_prelude::OriginFor;
use pallet_evm_precompile_nfts_tests::{ExtBuilder, ALICE, BOB};
use pallet_evm_precompile_nfts_types::solidity::{
	OptionalPriceWithDirection, OptionalU256, PriceDirection, PriceWithDirection,
};
use pallet_nfts::Pallet as NftsPallet;
use precompile_utils::testing::*;
use sp_core::U256;

type PCall = NftsSwapPrecompileCall<Runtime>;

fn precompiles() -> Precompiles<Runtime> {
	PrecompilesValue::get()
}

fn mint_each_collections() {
	assert_ok!(NftsPallet::<Runtime>::mint(
		OriginFor::<Runtime>::signed(ALICE.into()),
		0,
		1,
		ALICE.into(),
		None
	));
	assert_ok!(NftsPallet::<Runtime>::mint(
		OriginFor::<Runtime>::signed(BOB.into()),
		1,
		1,
		BOB.into(),
		None
	));
	assert_eq!(get_owner_of_item(0, 1), Some(ALICE.into()));
	assert_eq!(get_owner_of_item(1, 1), Some(BOB.into()));
}

fn get_owner_of_item(collection_id: u128, item_id: u128) -> Option<AccountId> {
	let owner_id: Option<AccountId> = NftsPallet::<Runtime>::owner(collection_id, item_id);

	owner_id
}

#[test]
fn selectors() {
	assert!(PCall::create_swap_selectors().contains(&0xf93f143a));
	assert!(PCall::cancel_swap_selectors().contains(&0x83698d19));
	assert!(PCall::claim_swap_selectors().contains(&0x0406ab6c));
}

#[test]
fn create_swap_works() {
	ExtBuilder::<Runtime>::default()
		.with_balances(vec![(ALICE.into(), 1000), (BOB.into(), 1000)])
		.build_with_collections()
		.execute_with(|| {
			let collection_id = 0;
			let item_id = 1;
			let desired_collection: u128 = 1;
			let maybe_desired_item = OptionalU256::from(Some(U256::from(1)));
			let maybe_price = OptionalPriceWithDirection::from(Some(PriceWithDirection::new(
				U256::from(100),
				PriceDirection::Receive,
			)));
			let duration = 100;
			let expect_deadline = 101;

			mint_each_collections();
			assert!(pallet_nfts::PendingSwapOf::<Runtime>::get(collection_id, item_id).is_none(),);
			precompiles()
				.prepare_test(
					ALICE,
					Precompile1,
					PCall::create_swap {
						offered_collection: collection_id.into(),
						offered_item: item_id.into(),
						desired_collection: desired_collection.into(),
						maybe_desired_item: maybe_desired_item.clone(),
						maybe_price: maybe_price.clone(),
						duration: duration.into(),
					},
				)
				.execute_returns(true);

			assert_eq!(get_owner_of_item(0, 1), Some(ALICE.into())); // check that the item is still owned by ALICE

			let swap = pallet_nfts::PendingSwapOf::<Runtime>::get(collection_id, item_id);
			assert!(swap.is_some());

			let swap = swap.unwrap();
			assert_eq!(*swap.desired_collection(), desired_collection);
			assert_eq!(swap.desired_item().unwrap(), maybe_desired_item.value.try_into().unwrap());
			assert_eq!(swap.price().as_ref().unwrap(), &maybe_price.value.try_into().unwrap());
			assert_eq!(*swap.deadline(), expect_deadline);
		});
}

#[test]
fn cancel_swap_works() {
	ExtBuilder::<Runtime>::default()
		.with_balances(vec![(ALICE.into(), 1000), (BOB.into(), 1000)])
		.build_with_collections()
		.execute_with(|| {
			let collection_id = 0;
			let item_id = 1;
			let desired_collection = 1;
			let maybe_desired_item = Some(1);
			let maybe_price = OptionalPriceWithDirection::from(Some(PriceWithDirection::new(
				U256::from(100),
				PriceDirection::Receive,
			)));
			let duration = 100;

			mint_each_collections();
			assert!(pallet_nfts::PendingSwapOf::<Runtime>::get(collection_id, item_id).is_none(),);
			assert_ok!(NftsPallet::<Runtime>::create_swap(
				OriginFor::<Runtime>::signed(ALICE.into()),
				collection_id,
				item_id,
				desired_collection,
				maybe_desired_item,
				maybe_price.try_into().unwrap(),
				duration
			));
			assert!(pallet_nfts::PendingSwapOf::<Runtime>::get(collection_id, item_id).is_some(),);

			precompiles()
				.prepare_test(
					ALICE,
					Precompile1,
					PCall::cancel_swap { offered_collection: 0.into(), offered_item: 1.into() },
				)
				.execute_returns(true);

			assert!(pallet_nfts::PendingSwapOf::<Runtime>::get(collection_id, item_id).is_none(),);
			assert_eq!(get_owner_of_item(0, 1), Some(ALICE.into()));
		});
}

#[test]
fn claim_swap_works() {
	ExtBuilder::<Runtime>::default()
		.with_balances(vec![(ALICE.into(), 1000), (BOB.into(), 1000)])
		.build_with_collections()
		.execute_with(|| {
			let collection_id = 0;
			let item_id = 1;
			let desired_collection = 1;
			let maybe_desired_item = Some(1);
			let maybe_price = OptionalPriceWithDirection::from(Some(PriceWithDirection::new(
				U256::from(100),
				PriceDirection::Receive,
			)));
			let duration = 100;

			mint_each_collections();
			assert_ok!(NftsPallet::<Runtime>::create_swap(
				OriginFor::<Runtime>::signed(ALICE.into()),
				collection_id,
				item_id,
				desired_collection,
				maybe_desired_item,
				maybe_price.try_into().unwrap(),
				duration
			));

			precompiles()
				.prepare_test(
					BOB,
					Precompile1,
					PCall::claim_swap {
						send_collection: 1.into(),
						send_item: 1.into(),
						receive_collection: 0.into(),
						receive_item: 1.into(),
						witness_price: OptionalPriceWithDirection::from(Some(
							PriceWithDirection::new(U256::from(100), PriceDirection::Receive),
						)),
					},
				)
				.execute_returns(true);

			assert!(pallet_nfts::PendingSwapOf::<Runtime>::get(collection_id, item_id).is_none(),);
			assert_eq!(get_owner_of_item(0, 1), Some(BOB.into()));
		});
}
