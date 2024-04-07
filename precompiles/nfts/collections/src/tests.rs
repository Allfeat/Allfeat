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

use crate::{mock::*, NftsPrecompileSet, NftsPrecompileSetCall};
use frame_support::{
	assert_err, assert_ok,
	traits::nonfungibles_v2::{Inspect, InspectRole, Mutate, Trading},
};
use frame_system::pallet_prelude::OriginFor;
use pallet_evm_precompile_nfts_tests::{
	ExtBuilder, ALICE, ALICE_COLLECTION_PRECOMPILE_ADDRESS, BOB, CHARLIE,
};
use pallet_evm_precompile_nfts_types::solidity::{
	AttributeNamespace, AttributeNamespaceInfo, CancelAttributesApprovalWitness, CollectionDetails,
	CollectionSettings, ItemSettings, MintInfo, MintSettings, MintType, OptionalAddress,
	OptionalMintWitness, OptionalU256,
};
use pallet_nfts::{CollectionConfigOf, CollectionSetting, Event, MintWitness};
use precompile_utils::{solidity::codec::bytes::BoundedBytesString, testing::*};
use sp_core::U256;
use sp_runtime::BoundedVec;

type PCall = NftsPrecompileSetCall<Runtime>;

fn precompiles() -> NftsPrecompileSet<Runtime> {
	PrecompilesValue::get()
}

#[test]
fn selectors() {
	// Getters
	assert!(PCall::get_details_selectors().contains(&0xb87f86b7));
	// Extrinsics
	assert!(PCall::mint_selectors().contains(&0xcd568c38));
	assert!(PCall::burn_selectors().contains(&0x42966c68));
	assert!(PCall::transfer_selectors().contains(&0xb7760c8f));
	assert!(PCall::lock_item_transfer_selectors().contains(&0x81c2e1e8));
	assert!(PCall::unlock_item_transfer_selectors().contains(&0x3b8413a5));
	assert!(PCall::seal_collection_selectors().contains(&0xa872c4c8));
	assert!(PCall::transfer_ownership_selectors().contains(&0xf0350c04));
	assert!(PCall::set_team_selectors().contains(&0xf8bf8e95));
	assert!(PCall::approve_transfer_selectors().contains(&0x0df4508b));
	assert!(PCall::cancel_approval_selectors().contains(&0x22b856f3));
	assert!(PCall::clear_all_transfer_approvals_selectors().contains(&0x6f83fe8a));
	assert!(PCall::lock_item_properties_selectors().contains(&0x91743611));
	assert!(PCall::set_collection_attribute_selectors().contains(&0xe8971f23));
	assert!(PCall::set_item_attribute_selectors().contains(&0x123ffb18));
	assert!(PCall::clear_collection_attribute_selectors().contains(&0x07ac98df));
	assert!(PCall::clear_item_attribute_selectors().contains(&0x29eaab3f));
	assert!(PCall::approve_item_attributes_selectors().contains(&0x620fea0d));
	assert!(PCall::cancel_item_attributes_approval_selectors().contains(&0xe96389a9));
	assert!(PCall::set_metadata_selectors().contains(&0x914384e8));
	assert!(PCall::clear_metadata_selectors().contains(&0xf7948baa));
	assert!(PCall::set_collection_metadata_selectors().contains(&0xee9b0247));
	assert!(PCall::clear_collection_metadata_selectors().contains(&0x8699f6de));
	assert!(PCall::set_collection_max_supply_selectors().contains(&0x5c59e577));
	assert!(PCall::update_mint_settings_selectors().contains(&0x9f8ca97d));
	assert!(PCall::set_price_selectors().contains(&0xfc019a21));
	assert!(PCall::buy_item_selectors().contains(&0x0a6169cf));
}

#[test]
fn get_details_works() {
	ExtBuilder::<Runtime>::default()
		.with_balances(vec![(ALICE.into(), 1000)])
		.build_with_collections()
		.execute_with(|| {
			precompiles()
				.prepare_test(ALICE, ALICE_COLLECTION_PRECOMPILE_ADDRESS, PCall::get_details {})
				.execute_returns(CollectionDetails {
					owner: ALICE.into(),
					owner_deposit: U256::from(0),
					items: 0,
					item_metadatas: 0,
					item_configs: 0,
					attributes: 0,
				});
		})
}

#[test]
fn mint_works() {
	ExtBuilder::<Runtime>::default()
		.with_balances(vec![(ALICE.into(), 1000)])
		.build_with_collections()
		.execute_with(|| {
			precompiles()
				.prepare_test(
					ALICE,
					ALICE_COLLECTION_PRECOMPILE_ADDRESS,
					PCall::mint {
						item_id: U256::from(0),
						mint_to: ALICE.into(),
						witness_data: OptionalMintWitness::default(),
					},
				)
				.execute_returns(true);

			System::assert_last_event(RuntimeEvent::Nfts(Event::Issued {
				collection: 0,
				item: 0,
				owner: ALICE.into(),
			}));

			assert_eq!(pallet_nfts::Pallet::<Runtime>::owner(0, 0), Some(ALICE.into()));
		})
}

#[test]
fn burn_works() {
	ExtBuilder::<Runtime>::default()
		.with_balances(vec![(ALICE.into(), 1000)])
		.build_with_collections()
		.execute_with(|| {
			assert_ok!(pallet_nfts::Pallet::<Runtime>::mint(
				OriginFor::<Runtime>::signed(ALICE.into()),
				0,
				0,
				ALICE.into(),
				None
			));

			assert_eq!(pallet_nfts::Pallet::<Runtime>::owner(0, 0), Some(ALICE.into()));

			precompiles()
				.prepare_test(
					ALICE,
					ALICE_COLLECTION_PRECOMPILE_ADDRESS,
					PCall::burn { item_id: U256::from(0) },
				)
				.execute_returns(true);

			System::assert_last_event(RuntimeEvent::Nfts(Event::Burned {
				collection: 0,
				item: 0,
				owner: ALICE.into(),
			}));

			assert_eq!(pallet_nfts::Pallet::<Runtime>::owner(0, 0), None);
		})
}

#[test]
fn transfer_works() {
	ExtBuilder::<Runtime>::default()
		.with_balances(vec![(ALICE.into(), 1000)])
		.build_with_collections()
		.execute_with(|| {
			assert_ok!(pallet_nfts::Pallet::<Runtime>::mint(
				OriginFor::<Runtime>::signed(ALICE.into()),
				0,
				0,
				ALICE.into(),
				None
			));

			assert_eq!(pallet_nfts::Pallet::<Runtime>::owner(0, 0), Some(ALICE.into()));

			precompiles()
				.prepare_test(
					ALICE,
					ALICE_COLLECTION_PRECOMPILE_ADDRESS,
					PCall::transfer { item_id: U256::from(0), dest: BOB.into() },
				)
				.execute_returns(true);

			System::assert_last_event(RuntimeEvent::Nfts(Event::Transferred {
				collection: 0,
				item: 0,
				from: ALICE.into(),
				to: BOB.into(),
			}));

			assert_eq!(pallet_nfts::Pallet::<Runtime>::owner(0, 0), Some(BOB.into()));
		})
}

#[test]
fn lock_item_transfer_works() {
	ExtBuilder::<Runtime>::default()
		.with_balances(vec![(ALICE.into(), 1000)])
		.build_with_collections()
		.execute_with(|| {
			assert_ok!(pallet_nfts::Pallet::<Runtime>::mint(
				OriginFor::<Runtime>::signed(ALICE.into()),
				0,
				0,
				ALICE.into(),
				None
			));

			precompiles()
				.prepare_test(
					ALICE,
					ALICE_COLLECTION_PRECOMPILE_ADDRESS,
					PCall::lock_item_transfer { item_id: U256::from(0) },
				)
				.execute_returns(true);

			System::assert_last_event(RuntimeEvent::Nfts(Event::ItemTransferLocked {
				collection: 0,
				item: 0,
			}));

			assert_err!(
				pallet_nfts::Pallet::<Runtime>::transfer(
					OriginFor::<Runtime>::signed(ALICE.into()),
					0,
					0,
					BOB.into(),
				),
				pallet_nfts::Error::<Runtime>::ItemLocked
			);
		})
}

#[test]
fn unlock_item_transfer_works() {
	ExtBuilder::<Runtime>::default()
		.with_balances(vec![(ALICE.into(), 1000)])
		.build_with_collections()
		.execute_with(|| {
			assert_ok!(pallet_nfts::Pallet::<Runtime>::mint(
				OriginFor::<Runtime>::signed(ALICE.into()),
				0,
				0,
				ALICE.into(),
				None
			));

			assert_ok!(pallet_nfts::Pallet::<Runtime>::lock_item_transfer(
				OriginFor::<Runtime>::signed(ALICE.into()),
				0,
				0,
			));

			assert_err!(
				pallet_nfts::Pallet::<Runtime>::transfer(
					OriginFor::<Runtime>::signed(ALICE.into()),
					0,
					0,
					BOB.into(),
				),
				pallet_nfts::Error::<Runtime>::ItemLocked
			);

			precompiles()
				.prepare_test(
					ALICE,
					ALICE_COLLECTION_PRECOMPILE_ADDRESS,
					PCall::unlock_item_transfer { item_id: U256::from(0) },
				)
				.execute_returns(true);

			System::assert_last_event(RuntimeEvent::Nfts(Event::ItemTransferUnlocked {
				collection: 0,
				item: 0,
			}));

			assert_ok!(pallet_nfts::Pallet::<Runtime>::transfer(
				OriginFor::<Runtime>::signed(ALICE.into()),
				0,
				0,
				BOB.into(),
			));
		})
}

#[test]
fn seal_collection_works() {
	ExtBuilder::<Runtime>::default()
		.with_balances(vec![(ALICE.into(), 1000)])
		.build_with_collections()
		.execute_with(|| {
			precompiles()
				.prepare_test(
					ALICE,
					ALICE_COLLECTION_PRECOMPILE_ADDRESS,
					PCall::seal_collection {
						settings: CollectionSettings {
							is_transferable_items: false,
							is_unlocked_metadata: false,
							is_unlocked_attributes: false,
							is_unlocked_max_supply: false,
							is_deposit_required: true,
						},
					},
				)
				.execute_returns(true);

			System::assert_last_event(RuntimeEvent::Nfts(Event::CollectionLocked {
				collection: 0,
			}));

			if let Some(config) = CollectionConfigOf::<Runtime>::get(0) {
				assert!(config.has_disabled_setting(CollectionSetting::DepositRequired)); // Still disabled as it was force_created
				assert!(config.has_disabled_setting(CollectionSetting::UnlockedAttributes));
				assert!(config.has_disabled_setting(CollectionSetting::UnlockedMetadata));
				assert!(config.has_disabled_setting(CollectionSetting::UnlockedMaxSupply));
				assert!(config.has_disabled_setting(CollectionSetting::TransferableItems));
			}
		})
}

#[test]
fn transfer_ownership_works() {
	ExtBuilder::<Runtime>::default()
		.with_balances(vec![(ALICE.into(), 1000), (BOB.into(), 1000)])
		.build_with_collections()
		.execute_with(|| {
			assert_ok!(pallet_nfts::Pallet::<Runtime>::set_accept_ownership(
				OriginFor::<Runtime>::signed(BOB.into()),
				Some(0),
			));

			precompiles()
				.prepare_test(
					ALICE,
					ALICE_COLLECTION_PRECOMPILE_ADDRESS,
					PCall::transfer_ownership { owner: BOB.into() },
				)
				.execute_returns(true);

			System::assert_last_event(RuntimeEvent::Nfts(Event::OwnerChanged {
				collection: 0,
				new_owner: BOB.into(),
			}));

			assert_eq!(pallet_nfts::Pallet::<Runtime>::collection_owner(0), Some(BOB.into()));
		})
}

#[test]
fn set_team_works() {
	ExtBuilder::<Runtime>::default()
		.with_balances(vec![(ALICE.into(), 1000)])
		.build_with_collections()
		.execute_with(|| {
			precompiles()
				.prepare_test(
					ALICE,
					ALICE_COLLECTION_PRECOMPILE_ADDRESS,
					PCall::set_team {
						issuer: ALICE.into(),
						admin: BOB.into(),
						freezer: BOB.into(),
					},
				)
				.execute_returns(true);

			System::assert_last_event(RuntimeEvent::Nfts(Event::TeamChanged {
				collection: 0,
				issuer: Some(ALICE.into()),
				admin: Some(BOB.into()),
				freezer: Some(BOB.into()),
			}));

			assert!(pallet_nfts::Pallet::<Runtime>::is_issuer(&0, &ALICE.into(),));

			assert!(pallet_nfts::Pallet::<Runtime>::is_freezer(&0, &BOB.into(),));

			assert!(pallet_nfts::Pallet::<Runtime>::is_admin(&0, &BOB.into(),));
		})
}

#[test]
fn approve_transfer_works() {
	ExtBuilder::<Runtime>::default()
		.with_balances(vec![(ALICE.into(), 1000)])
		.build_with_collections()
		.execute_with(|| {
			assert_ok!(pallet_nfts::Pallet::<Runtime>::mint(
				OriginFor::<Runtime>::signed(ALICE.into()),
				0,
				0,
				ALICE.into(),
				None
			));

			precompiles()
				.prepare_test(
					ALICE,
					ALICE_COLLECTION_PRECOMPILE_ADDRESS,
					PCall::approve_transfer {
						item: U256::from(0),
						delegate: BOB.into(),
						maybe_deadline: OptionalU256::default(),
					},
				)
				.execute_returns(true);

			System::assert_last_event(RuntimeEvent::Nfts(Event::TransferApproved {
				collection: 0,
				item: 0,
				owner: ALICE.into(),
				delegate: BOB.into(),
				deadline: None,
			}));

			assert_ok!(pallet_nfts::Pallet::<Runtime>::transfer(
				OriginFor::<Runtime>::signed(BOB.into()),
				0,
				0,
				CHARLIE.into(),
			));

			assert_eq!(pallet_nfts::Pallet::<Runtime>::owner(0, 0), Some(CHARLIE.into()));
		})
}

#[test]
fn cancel_approval_works() {
	ExtBuilder::<Runtime>::default()
		.with_balances(vec![(ALICE.into(), 1000)])
		.build_with_collections()
		.execute_with(|| {
			assert_ok!(pallet_nfts::Pallet::<Runtime>::mint(
				OriginFor::<Runtime>::signed(ALICE.into()),
				0,
				0,
				ALICE.into(),
				None
			));

			assert_ok!(pallet_nfts::Pallet::<Runtime>::approve_transfer(
				OriginFor::<Runtime>::signed(ALICE.into()),
				0,
				0,
				BOB.into(),
				None
			));

			precompiles()
				.prepare_test(
					ALICE,
					ALICE_COLLECTION_PRECOMPILE_ADDRESS,
					PCall::cancel_approval { item: U256::from(0), delegate: BOB.into() },
				)
				.execute_returns(true);

			System::assert_last_event(RuntimeEvent::Nfts(Event::ApprovalCancelled {
				collection: 0,
				item: 0,
				owner: ALICE.into(),
				delegate: BOB.into(),
			}));

			assert_err!(
				pallet_nfts::Pallet::<Runtime>::transfer(
					OriginFor::<Runtime>::signed(BOB.into()),
					0,
					0,
					CHARLIE.into(),
				),
				pallet_nfts::Error::<Runtime>::NoPermission
			);
		})
}

#[test]
fn clear_all_transfer_approvals_works() {
	ExtBuilder::<Runtime>::default()
		.with_balances(vec![(ALICE.into(), 1000)])
		.build_with_collections()
		.execute_with(|| {
			assert_ok!(pallet_nfts::Pallet::<Runtime>::mint(
				OriginFor::<Runtime>::signed(ALICE.into()),
				0,
				0,
				ALICE.into(),
				None
			));

			assert_ok!(pallet_nfts::Pallet::<Runtime>::approve_transfer(
				OriginFor::<Runtime>::signed(ALICE.into()),
				0,
				0,
				BOB.into(),
				None
			));

			precompiles()
				.prepare_test(
					ALICE,
					ALICE_COLLECTION_PRECOMPILE_ADDRESS,
					PCall::clear_all_transfer_approvals { item: U256::from(0) },
				)
				.execute_returns(true);

			System::assert_last_event(RuntimeEvent::Nfts(Event::AllApprovalsCancelled {
				collection: 0,
				item: 0,
				owner: ALICE.into(),
			}));

			assert_err!(
				pallet_nfts::Pallet::<Runtime>::transfer(
					OriginFor::<Runtime>::signed(BOB.into()),
					0,
					0,
					CHARLIE.into(),
				),
				pallet_nfts::Error::<Runtime>::NoPermission
			);
		})
}

#[test]
fn lock_item_properties_works() {
	ExtBuilder::<Runtime>::default()
		.with_balances(vec![(ALICE.into(), 1000)])
		.build_with_collections()
		.execute_with(|| {
			assert_ok!(pallet_nfts::Pallet::<Runtime>::mint(
				OriginFor::<Runtime>::signed(ALICE.into()),
				0,
				0,
				ALICE.into(),
				None
			));

			precompiles()
				.prepare_test(
					ALICE,
					ALICE_COLLECTION_PRECOMPILE_ADDRESS,
					PCall::lock_item_properties {
						item: U256::from(0),
						lock_metadata: true,
						lock_attributes: true,
					},
				)
				.execute_returns(true);

			System::assert_last_event(RuntimeEvent::Nfts(Event::ItemPropertiesLocked {
				collection: 0,
				item: 0,
				lock_metadata: true,
				lock_attributes: true,
			}));

			assert_err!(
				pallet_nfts::Pallet::<Runtime>::set_item_metadata(
					Some(&ALICE.into()),
					&0,
					&0,
					&[0],
				),
				pallet_nfts::Error::<Runtime>::LockedItemMetadata
			);

			assert_err!(
				pallet_nfts::Pallet::<Runtime>::set_attribute(
					OriginFor::<Runtime>::signed(ALICE.into()),
					0,
					Some(0),
					pallet_nfts::AttributeNamespace::CollectionOwner,
					BoundedVec::new(),
					BoundedVec::new(),
				),
				pallet_nfts::Error::<Runtime>::LockedItemAttributes
			);
		})
}

#[test]
fn set_collection_attribute_works() {
	ExtBuilder::<Runtime>::default()
		.with_balances(vec![(ALICE.into(), 1000)])
		.build_with_collections()
		.execute_with(|| {
			precompiles()
				.prepare_test(
					ALICE,
					ALICE_COLLECTION_PRECOMPILE_ADDRESS,
					PCall::set_collection_attribute {
						namespace: AttributeNamespaceInfo {
							namespace: AttributeNamespace::CollectionOwner,
							account: ALICE.into(),
						},
						key: "key".into(),
						value: "value".into(),
					},
				)
				.execute_returns(true);

			System::assert_last_event(RuntimeEvent::Nfts(Event::AttributeSet {
				collection: 0,
				maybe_item: None,
				key: BoundedVec::try_from(b"key".to_vec()).unwrap(),
				value: BoundedVec::try_from(b"value".to_vec()).unwrap(),
				namespace: pallet_nfts::AttributeNamespace::CollectionOwner,
			}));

			assert_eq!(
				pallet_nfts::Pallet::<Runtime>::collection_attribute(
					&0,
					"key".to_string().as_bytes()
				),
				Some("value".into())
			);
		})
}

#[test]
fn set_item_attribute_works() {
	ExtBuilder::<Runtime>::default()
		.with_balances(vec![(ALICE.into(), 1000)])
		.build_with_collections()
		.execute_with(|| {
			assert_ok!(pallet_nfts::Pallet::<Runtime>::mint(
				OriginFor::<Runtime>::signed(ALICE.into()),
				0,
				0,
				ALICE.into(),
				None
			));

			precompiles()
				.prepare_test(
					ALICE,
					ALICE_COLLECTION_PRECOMPILE_ADDRESS,
					PCall::set_item_attribute {
						item: U256::from(0),
						namespace: AttributeNamespaceInfo {
							namespace: AttributeNamespace::CollectionOwner,
							account: ALICE.into(),
						},
						key: "key".into(),
						value: "value".into(),
					},
				)
				.execute_returns(true);

			System::assert_last_event(RuntimeEvent::Nfts(Event::AttributeSet {
				collection: 0,
				maybe_item: Some(0),
				key: BoundedVec::try_from(b"key".to_vec()).unwrap(),
				value: BoundedVec::try_from(b"value".to_vec()).unwrap(),
				namespace: pallet_nfts::AttributeNamespace::CollectionOwner,
			}));

			assert_eq!(
				pallet_nfts::Pallet::<Runtime>::attribute(&0, &0, b"key"),
				Some("value".into())
			);
		})
}

#[test]
fn clear_collection_attribute_works() {
	ExtBuilder::<Runtime>::default()
		.with_balances(vec![(ALICE.into(), 1000)])
		.build_with_collections()
		.execute_with(|| {
			let key = BoundedVec::try_from(b"key".to_vec()).unwrap();
			let value = BoundedVec::try_from(b"value".to_vec()).unwrap();

			assert_ok!(pallet_nfts::Pallet::<Runtime>::set_attribute(
				OriginFor::<Runtime>::signed(ALICE.into()),
				0,
				None,
				pallet_nfts::AttributeNamespace::CollectionOwner,
				key.clone(),
				value,
			));

			assert_eq!(
				pallet_nfts::Pallet::<Runtime>::collection_attribute(
					&0,
					"key".to_string().as_bytes()
				),
				Some("value".into())
			);

			precompiles()
				.prepare_test(
					ALICE,
					ALICE_COLLECTION_PRECOMPILE_ADDRESS,
					PCall::clear_collection_attribute {
						namespace: AttributeNamespaceInfo {
							namespace: AttributeNamespace::CollectionOwner,
							account: ALICE.into(),
						},
						key: "key".into(),
					},
				)
				.execute_returns(true);

			System::assert_last_event(RuntimeEvent::Nfts(Event::AttributeCleared {
				collection: 0,
				maybe_item: None,
				key,
				namespace: pallet_nfts::AttributeNamespace::CollectionOwner,
			}));

			assert_eq!(
				pallet_nfts::Pallet::<Runtime>::collection_attribute(
					&0,
					"key".to_string().as_bytes()
				),
				None
			);
		})
}

#[test]
fn clear_item_attribute_works() {
	ExtBuilder::<Runtime>::default()
		.with_balances(vec![(ALICE.into(), 1000)])
		.build_with_collections()
		.execute_with(|| {
			let key = BoundedVec::try_from(b"key".to_vec()).unwrap();
			let value = BoundedVec::try_from(b"value".to_vec()).unwrap();

			assert_ok!(pallet_nfts::Pallet::<Runtime>::mint(
				OriginFor::<Runtime>::signed(ALICE.into()),
				0,
				0,
				ALICE.into(),
				None
			));

			assert_ok!(pallet_nfts::Pallet::<Runtime>::set_attribute(
				OriginFor::<Runtime>::signed(ALICE.into()),
				0,
				Some(0),
				pallet_nfts::AttributeNamespace::CollectionOwner,
				key.clone(),
				value,
			));

			assert_eq!(
				pallet_nfts::Pallet::<Runtime>::attribute(&0, &0, b"key"),
				Some("value".into())
			);

			precompiles()
				.prepare_test(
					ALICE,
					ALICE_COLLECTION_PRECOMPILE_ADDRESS,
					PCall::clear_item_attribute {
						item: U256::from(0),
						namespace: AttributeNamespaceInfo {
							namespace: AttributeNamespace::CollectionOwner,
							account: ALICE.into(),
						},
						key: "key".into(),
					},
				)
				.execute_returns(true);

			System::assert_last_event(RuntimeEvent::Nfts(Event::AttributeCleared {
				collection: 0,
				maybe_item: Some(0),
				key,
				namespace: pallet_nfts::AttributeNamespace::CollectionOwner,
			}));

			assert_eq!(pallet_nfts::Pallet::<Runtime>::attribute(&0, &0, "key".as_bytes()), None,);
		})
}

#[test]
fn approve_item_attributes_works() {
	ExtBuilder::<Runtime>::default()
		.with_balances(vec![(ALICE.into(), 1000), (BOB.into(), 1000)])
		.build_with_collections()
		.execute_with(|| {
			assert_ok!(pallet_nfts::Pallet::<Runtime>::mint(
				OriginFor::<Runtime>::signed(ALICE.into()),
				0,
				0,
				ALICE.into(),
				None
			));

			precompiles()
				.prepare_test(
					ALICE,
					ALICE_COLLECTION_PRECOMPILE_ADDRESS,
					PCall::approve_item_attributes { item: U256::from(0), delegate: BOB.into() },
				)
				.execute_returns(true);

			System::assert_last_event(RuntimeEvent::Nfts(Event::ItemAttributesApprovalAdded {
				collection: 0,
				item: 0,
				delegate: BOB.into(),
			}));

			assert_ok!(pallet_nfts::Pallet::<Runtime>::set_attribute(
				OriginFor::<Runtime>::signed(BOB.into()),
				0,
				Some(0),
				pallet_nfts::AttributeNamespace::Account(BOB.into()),
				BoundedVec::try_from(b"key".to_vec()).unwrap(),
				BoundedVec::try_from(b"value".to_vec()).unwrap(),
			));
		})
}

#[test]
fn cancel_item_attributes_approval_works() {
	ExtBuilder::<Runtime>::default()
		.with_balances(vec![(ALICE.into(), 1000)])
		.build_with_collections()
		.execute_with(|| {
			assert_ok!(pallet_nfts::Pallet::<Runtime>::mint(
				OriginFor::<Runtime>::signed(ALICE.into()),
				0,
				0,
				ALICE.into(),
				None
			));

			assert_ok!(pallet_nfts::Pallet::<Runtime>::approve_item_attributes(
				OriginFor::<Runtime>::signed(ALICE.into()),
				0,
				0,
				BOB.into(),
			));

			precompiles()
				.prepare_test(
					ALICE,
					ALICE_COLLECTION_PRECOMPILE_ADDRESS,
					PCall::cancel_item_attributes_approval {
						item: U256::from(0),
						delegate: BOB.into(),
						witness: CancelAttributesApprovalWitness::default(),
					},
				)
				.execute_returns(true);

			System::assert_last_event(RuntimeEvent::Nfts(Event::ItemAttributesApprovalRemoved {
				collection: 0,
				item: 0,
				delegate: BOB.into(),
			}));

			assert_err!(
				pallet_nfts::Pallet::<Runtime>::set_attribute(
					OriginFor::<Runtime>::signed(BOB.into()),
					0,
					Some(0),
					pallet_nfts::AttributeNamespace::Account(BOB.into()),
					BoundedVec::try_from(b"key".to_vec()).unwrap(),
					BoundedVec::try_from(b"value".to_vec()).unwrap(),
				),
				pallet_nfts::Error::<Runtime>::NoPermission
			);
		})
}

#[test]
fn set_metadata_works() {
	ExtBuilder::<Runtime>::default()
		.with_balances(vec![(ALICE.into(), 1000)])
		.build_with_collections()
		.execute_with(|| {
			assert_ok!(pallet_nfts::Pallet::<Runtime>::mint(
				OriginFor::<Runtime>::signed(ALICE.into()),
				0,
				0,
				ALICE.into(),
				None,
			));

			precompiles()
				.prepare_test(
					ALICE,
					ALICE_COLLECTION_PRECOMPILE_ADDRESS,
					PCall::set_metadata {
						item: U256::from(0),
						data: BoundedBytesString::from(b"metadata"),
					},
				)
				.execute_returns(true);

			System::assert_last_event(RuntimeEvent::Nfts(Event::ItemMetadataSet {
				collection: 0,
				item: 0,
				data: BoundedVec::try_from(b"metadata".to_vec()).unwrap(),
			}));

			assert_eq!(
				pallet_nfts::Pallet::<Runtime>::attribute(&0, &0, &[]),
				Some("metadata".into()),
			);
		})
}

#[test]
fn clear_metadata_works() {
	ExtBuilder::<Runtime>::default()
		.with_balances(vec![(ALICE.into(), 1000)])
		.build_with_collections()
		.execute_with(|| {
			let metadata = BoundedVec::try_from(b"metadata".to_vec()).unwrap();

			assert_ok!(pallet_nfts::Pallet::<Runtime>::mint(
				OriginFor::<Runtime>::signed(ALICE.into()),
				0,
				0,
				ALICE.into(),
				None
			));

			assert_ok!(pallet_nfts::Pallet::<Runtime>::set_metadata(
				OriginFor::<Runtime>::signed(ALICE.into()),
				0,
				0,
				metadata,
			));

			assert_eq!(
				pallet_nfts::Pallet::<Runtime>::attribute(&0, &0, &[]),
				Some("metadata".into()),
			);

			precompiles()
				.prepare_test(
					ALICE,
					ALICE_COLLECTION_PRECOMPILE_ADDRESS,
					PCall::clear_metadata { item: U256::from(0) },
				)
				.execute_returns(true);

			System::assert_last_event(RuntimeEvent::Nfts(Event::ItemMetadataCleared {
				collection: 0,
				item: 0,
			}));

			assert_eq!(pallet_nfts::Pallet::<Runtime>::attribute(&0, &0, &[]), None,);
		})
}

#[test]
fn set_collection_metadata_works() {
	ExtBuilder::<Runtime>::default()
		.with_balances(vec![(ALICE.into(), 1000)])
		.build_with_collections()
		.execute_with(|| {
			precompiles()
				.prepare_test(
					ALICE,
					ALICE_COLLECTION_PRECOMPILE_ADDRESS,
					PCall::set_collection_metadata { data: BoundedBytesString::from(b"metadata") },
				)
				.execute_returns(true);

			System::assert_last_event(RuntimeEvent::Nfts(Event::CollectionMetadataSet {
				collection: 0,
				data: BoundedVec::try_from(b"metadata".to_vec()).unwrap(),
			}));

			assert_eq!(
				pallet_nfts::Pallet::<Runtime>::collection_attribute(&0, &[]),
				Some("metadata".into()),
			);
		})
}

#[test]
fn clear_collection_metadata_works() {
	ExtBuilder::<Runtime>::default()
		.with_balances(vec![(ALICE.into(), 1000)])
		.build_with_collections()
		.execute_with(|| {
			let metadata = BoundedVec::try_from(b"metadata".to_vec()).unwrap();

			assert_ok!(pallet_nfts::Pallet::<Runtime>::set_collection_metadata(
				OriginFor::<Runtime>::signed(ALICE.into()),
				0,
				metadata
			));

			assert_eq!(
				pallet_nfts::Pallet::<Runtime>::collection_attribute(&0, &[]),
				Some("metadata".into()),
			);

			precompiles()
				.prepare_test(
					ALICE,
					ALICE_COLLECTION_PRECOMPILE_ADDRESS,
					PCall::clear_collection_metadata {},
				)
				.execute_returns(true);

			System::assert_last_event(RuntimeEvent::Nfts(Event::CollectionMetadataCleared {
				collection: 0,
			}));

			assert_eq!(pallet_nfts::Pallet::<Runtime>::collection_attribute(&0, &[]), None,);
		})
}

#[test]
fn set_collection_max_supply_works() {
	ExtBuilder::<Runtime>::default()
		.with_balances(vec![(ALICE.into(), 1000)])
		.build_with_collections()
		.execute_with(|| {
			precompiles()
				.prepare_test(
					ALICE,
					ALICE_COLLECTION_PRECOMPILE_ADDRESS,
					PCall::set_collection_max_supply { max_supply: 1 },
				)
				.execute_returns(true);

			System::assert_last_event(RuntimeEvent::Nfts(Event::CollectionMaxSupplySet {
				collection: 0,
				max_supply: 1,
			}));

			assert_ok!(pallet_nfts::Pallet::<Runtime>::mint(
				OriginFor::<Runtime>::signed(ALICE.into()),
				0,
				0,
				ALICE.into(),
				None
			));

			assert_err!(
				pallet_nfts::Pallet::<Runtime>::mint(
					OriginFor::<Runtime>::signed(ALICE.into()),
					0,
					1,
					ALICE.into(),
					None
				),
				pallet_nfts::Error::<Runtime>::MaxSupplyReached
			);
		})
}

#[test]
fn update_mint_settings_works() {
	ExtBuilder::<Runtime>::default()
		.with_balances(vec![(ALICE.into(), 1000)])
		.build_with_collections()
		.execute_with(|| {
			precompiles()
				.prepare_test(
					ALICE,
					ALICE_COLLECTION_PRECOMPILE_ADDRESS,
					PCall::update_mint_settings {
						mint_settings: MintSettings {
							mint_type: MintInfo {
								collection_id: U256::from(0),
								mint_type: MintType::Public,
							},
							price: OptionalU256 { has_value: true, value: U256::from(100) },
							start_block: OptionalU256::default(),
							end_block: OptionalU256::default(),
							default_item_settings: ItemSettings {
								is_transferable: true,
								is_unlocked_metadata: true,
								is_unlocked_attributes: true,
							},
						},
					},
				)
				.execute_returns(true);

			System::assert_last_event(RuntimeEvent::Nfts(Event::CollectionMintSettingsUpdated {
				collection: 0,
			}));

			assert_eq!(
				CollectionConfigOf::<Runtime>::get(0).unwrap().mint_settings,
				pallet_nfts::MintSettings {
					mint_type: pallet_nfts::MintType::Public,
					price: Some(100),
					start_block: None,
					end_block: None,
					default_item_settings: ItemSettings {
						is_transferable: true,
						is_unlocked_metadata: true,
						is_unlocked_attributes: true,
					}
					.into(),
				}
			);
		})
}

#[test]
fn set_price_works() {
	ExtBuilder::<Runtime>::default()
		.with_balances(vec![(ALICE.into(), 1000)])
		.build_with_collections()
		.execute_with(|| {
			assert_ok!(pallet_nfts::Pallet::<Runtime>::mint(
				OriginFor::<Runtime>::signed(ALICE.into()),
				0,
				0,
				ALICE.into(),
				None,
			));

			precompiles()
				.prepare_test(
					ALICE,
					ALICE_COLLECTION_PRECOMPILE_ADDRESS,
					PCall::set_price {
						item: U256::from(0),
						whitelisted_buyer: OptionalAddress::default(),
						price: OptionalU256 { value: U256::from(100), has_value: true },
					},
				)
				.execute_returns(true);

			System::assert_last_event(RuntimeEvent::Nfts(Event::ItemPriceSet {
				collection: 0,
				item: 0,
				whitelisted_buyer: None,
				price: 100,
			}));

			assert_eq!(pallet_nfts::Pallet::<Runtime>::item_price(&0, &0), Some(100));
		})
}

#[test]
fn buy_item_works() {
	ExtBuilder::<Runtime>::default()
		.with_balances(vec![(ALICE.into(), 1000), (BOB.into(), 1000)])
		.build_with_collections()
		.execute_with(|| {
			assert_ok!(pallet_nfts::Pallet::<Runtime>::mint(
				OriginFor::<Runtime>::signed(ALICE.into()),
				0,
				0,
				ALICE.into(),
				Some(MintWitness { owned_item: None, mint_price: Some(100) }),
			));

			assert_ok!(pallet_nfts::Pallet::<Runtime>::set_price(
				OriginFor::<Runtime>::signed(ALICE.into()),
				0,
				0,
				Some(100),
				None,
			));

			precompiles()
				.prepare_test(
					BOB,
					ALICE_COLLECTION_PRECOMPILE_ADDRESS,
					PCall::buy_item { item: U256::from(0), bid_price: U256::from(100) },
				)
				.execute_returns(true);

			System::assert_last_event(RuntimeEvent::Nfts(Event::ItemBought {
				collection: 0,
				item: 0,
				price: 100,
				seller: ALICE.into(),
				buyer: BOB.into(),
			}));

			assert_eq!(pallet_nfts::Pallet::<Runtime>::owner(0, 0,), Some(BOB.into()));
		})
}
