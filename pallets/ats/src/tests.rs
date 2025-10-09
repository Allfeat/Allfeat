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

use crate::{AtsByOwner, AtsIdByHash, AtsVersions, AtsWorks, Error, LatestVersion, mock::*};
use frame_support::{pallet_prelude::TypedGet, testing_prelude::*, traits::fungible::InspectHold};

fn hex_to_vec(s: &str) -> Vec<u8> {
    let s = s.strip_prefix("0x").unwrap_or(s);
    hex::decode(s).expect("valid hex")
}

fn hex_to_pub(s: &str) -> [u8; 32] {
    let v = hex_to_vec(s);
    assert_eq!(v.len(), 32, "public input must be 32 bytes");
    let mut out = [0u8; 32];
    out.copy_from_slice(&v);
    out
}

// ============================
// Global Test Data
// ============================

const VK_HEX: &str = "0x0dad748a7ef4a81fc022070d1d92142ce7dfe8565c4852fcd25fbcaf7906759f9b724b742bb839ebb319eb346ed517faf82e7e66276219c37cfa8d9d0b5cef0a3494b9dfc76fe7406f71be3fe5c2a72f04d85d13d113f0d2926f9b44f7876a29cf9f3060f8e58114518eb3d3ae033419dca17765b66b106dac5647cabc47da13169bc4a3e626f2200ba189f9f17f548cb66d6ef1da7c3e9db81388ae2f834d895d789bb4d21c35d8257991b0a339bbbd488328a7ee70358265b35bb3181aef02899afeaf67b693aa04828aa929998d0152527f2e67f901fab54f8717709e9faa0700000000000000e9e1273293c1a32aa27705729bb1f2e0293e1cb744a087c70d369d25cddff2a4ef73d88aec5f058ac2de61635a380211e49276e772c7926edb5264069101b106c91a5c9405a7b26c9bc188cd29d1275b141fdda0d766fbf019c2563b73c6d8ae2f6652677d17fc5f2e9c49ede6df9b01fe3ed1992a50c0d7c645a1852ce68f197fb033f9073337dbdf7645ad8efe51b9cbacb4726984a41fa00fadf2f73a080bf528732cf871bcc682a10d6a5973464b35e8589fe33a37d08748f8e4adc4470c60d97cbb85e99ff481168bda0d45c68e10a7433cea5287523ec800292cf94c95";
const PROOF_HEX: &str = "0x2e2008dc99bbc214438279dc6c527abf5d3b544d6535e2e1a8240eff60e3528524009ffa9f7dd9582f4aea6d64ee999dcbc068d84293f15ab7ee8121d4b5e812970acdff96b8371b2b75a194f591a0cb5c104aef6ad3523376f11cf17e13f7af3ba5ca7ff69cd5262c34092becafc3e44df7be4a830388640d8fd1821687d3a4";
const PUBS_HEX: [&str; 6] = [
    "0x26d273f7c73a635f6eaeb904e116ec4cd887fb5a87fc7427c95279e6053e5bf0",
    "0x175eeef716d52cf8ee972c6fefd60e47df5084efde3c188c40a81a42e72dfb04",
    "0x017ac5e7a52bec07ca8ee344a9979aa083b7713f1196af35310de21746985079",
    "0x2a6dda925d7af47190183415517709278c73a94b40ab39f56d058c0bf0a84c68",
    "0x0000000000000000000000000000000000000000000000000000000000002710",
    "0x17c57750af41a2dc524ba01dd95bf7876d738eac80936fe96f374086ed91391d",
];


// ============================
// Register Tests
// ============================

#[test]
fn register_ats_successfully() {
    sp_tracing::init_for_tests();

    new_test_ext().execute_with(|| {
        let provider = 1;
        let hash_commitment = hex_to_pub(PUBS_HEX[3]);

        // Get the expected lock cost (fixed registration cost)
        let expected_lock_cost = <<Test as crate::Config>::AtsRegistrationCost as TypedGet>::get();

        assert_ok!(MockAts::register(
            RuntimeOrigin::signed(provider),
            hash_commitment
        ));

        // Check that funds are held
        assert_eq!(
            expected_lock_cost,
            Balances::balance_on_hold(&crate::HoldReason::AtsRegistration.into(), &provider)
        );

        // Check that ATS work is stored
        let ats_id = 0; // First ATS ID should be 0
        let stored_work = AtsWorks::<Test>::get(ats_id).expect("ATS work should be stored");
        assert_eq!(stored_work.owner, provider);
        assert_eq!(stored_work.id, ats_id);

        // Check that ATS version is stored
        let stored_version =
            AtsVersions::<Test>::get(ats_id, 1).expect("ATS version should be stored");
        assert_eq!(stored_version.version, 1);
        assert_eq!(stored_version.hash_commitment, hash_commitment);

        // Check that hash commitment maps to ATS ID
        let mapped_id = AtsIdByHash::<Test>::get(hash_commitment).expect("Hash should map to ID");
        assert_eq!(mapped_id, ats_id);

        // Check that latest version is 1
        let latest = LatestVersion::<Test>::get(ats_id);
        assert_eq!(latest, 1);

        // Check that ATS ID is added to owner's list
        let owner_list = AtsByOwner::<Test>::get(provider).expect("Owner should have list");
        assert!(owner_list.contains(&ats_id));
    });
}

#[test]
fn register_without_enough_funds_fail() {
    sp_tracing::init_for_tests();

    new_test_ext().execute_with(|| {
        let provider = 5; // This account has 0 balance in mock
        let hash_commitment = hex_to_pub(PUBS_HEX[3]);

        assert_err!(
            MockAts::register(RuntimeOrigin::signed(provider), hash_commitment),
            Error::<Test>::CantHoldFunds
        );
    });
}

#[test]
fn register_same_hash_commitment_fail() {
    sp_tracing::init_for_tests();

    new_test_ext().execute_with(|| {
        let provider = 1;
        let hash_commitment = hex_to_pub(PUBS_HEX[3]);

        // Register once - should succeed
        assert_ok!(MockAts::register(
            RuntimeOrigin::signed(provider),
            hash_commitment
        ));

        // Try to register again with same hash commitment - should fail
        assert_err!(
            MockAts::register(RuntimeOrigin::signed(provider), hash_commitment),
            Error::<Test>::AtsDataAlreadyExist
        );
    });
}

// ============================
// Claim Tests
// ============================

#[test]
fn claim_ats_successfully() {
    sp_tracing::init_for_tests();

    new_test_ext().execute_with(|| {
        let original_owner = 1;
        let new_owner = 2;
        let vk = hex_to_vec(VK_HEX);
        let proof = hex_to_vec(PROOF_HEX);
        let pubs: Vec<[u8; 32]> = PUBS_HEX.iter().map(|h| hex_to_pub(h)).collect();
        let hash_commitment = hex_to_pub(PUBS_HEX[3]);

        // First register the ATS with original owner
        assert_ok!(MockAts::register(
            RuntimeOrigin::signed(original_owner),
            hash_commitment
        ));

        // Verify original owner has it
        let ats_id = 0; // First ATS ID should be 0
        let ats_work = AtsWorks::<Test>::get(ats_id).expect("ATS work should be stored");
        assert_eq!(ats_work.owner, original_owner);

        // Now claim it with new owner
        assert_ok!(MockAts::claim(
            RuntimeOrigin::signed(new_owner),
            vk,
            pubs,
            proof
        ));

        // Verify ownership transfer
        let ats_work = AtsWorks::<Test>::get(ats_id).expect("ATS work should still be stored");
        assert_eq!(ats_work.owner, new_owner);

        // Verify ATS ID removed from original owner's list
        let original_owner_list = AtsByOwner::<Test>::get(original_owner).unwrap_or_default();
        assert!(!original_owner_list.contains(&ats_id));

        // Verify ATS ID added to new owner's list
        let new_owner_list =
            AtsByOwner::<Test>::get(new_owner).expect("New owner should have list");
        assert!(new_owner_list.contains(&ats_id));
    });
}

#[test]
fn claim_non_existent_ats_fail() {
    sp_tracing::init_for_tests();

    new_test_ext().execute_with(|| {
        let claimer = 2;
        let vk = hex_to_vec(VK_HEX);
        let proof = hex_to_vec(PROOF_HEX);
        let pubs: Vec<[u8; 32]> = PUBS_HEX.iter().map(|h| hex_to_pub(h)).collect();

        // Try to claim without registering first - should fail
        assert_err!(
            MockAts::claim(RuntimeOrigin::signed(claimer), vk, pubs, proof),
            Error::<Test>::AtsNotFound
        );
    });
}

#[test]
fn claim_with_invalid_proof_fail() {
    sp_tracing::init_for_tests();

    new_test_ext().execute_with(|| {
        let original_owner = 1;
        let claimer = 2;
        let vk = hex_to_vec(VK_HEX);
        let proof = hex_to_vec(PROOF_HEX);
        let invalid_proof = hex_to_vec(INVALID_PROOF_HEX);
        let pubs: Vec<[u8; 32]> = PUBS_HEX.iter().map(|h| hex_to_pub(h)).collect();

        // First register the ATS with original owner
        assert_ok!(MockAts::register(
            RuntimeOrigin::signed(original_owner),
            vk.clone(),
            pubs.clone(),
            proof
        ));

        // Try to claim with invalid proof - should fail
        assert_err!(
            MockAts::claim(RuntimeOrigin::signed(claimer), vk, pubs, invalid_proof),
            Error::<Test>::VerificationFailed
        );
    });
}

// ============================
// Update Tests
// ============================

#[test]
fn update_ats_successfully() {
    sp_tracing::init_for_tests();

    new_test_ext().execute_with(|| {
        let owner = 1;
        let vk = hex_to_vec(VK_HEX);
        let proof = hex_to_vec(PROOF_HEX);
        let pubs: Vec<[u8; 32]> = PUBS_HEX.iter().map(|h| hex_to_pub(h)).collect();
        let hash_commitment_v1 = hex_to_pub(PUBS_HEX[3]);

        // First register the ATS
        assert_ok!(MockAts::register(
            RuntimeOrigin::signed(owner),
            hash_commitment_v1
        ));

        let ats_id = 0;

        // Verify version 1 is stored
        let version_1 = AtsVersions::<Test>::get(ats_id, 1).expect("Version 1 should be stored");
        assert_eq!(version_1.version, 1);
        assert_eq!(version_1.hash_commitment, hash_commitment_v1);
        assert_eq!(LatestVersion::<Test>::get(ats_id), 1);

        // Update with the same valid proof and hash commitment (in reality, this would be a new proof for a new version)
        // Calculate expected cost for the new version (fixed registration cost per update)
        let version_cost = <<Test as crate::Config>::AtsRegistrationCost as TypedGet>::get();

        let initial_hold =
            Balances::balance_on_hold(&crate::HoldReason::AtsRegistration.into(), &owner);

        // Update to version 2 (using same proof for test simplicity)
        assert_ok!(MockAts::update(
            RuntimeOrigin::signed(owner),
            ats_id,
            vk,
            pubs,
            proof
        ));

        // Verify version 2 is stored
        let version_2 = AtsVersions::<Test>::get(ats_id, 2).expect("Version 2 should be stored");
        assert_eq!(version_2.version, 2);
        assert_eq!(version_2.hash_commitment, hash_commitment_v1);

        // Verify latest version is updated
        assert_eq!(LatestVersion::<Test>::get(ats_id), 2);

        // Verify hash lookup still points to this ATS ID
        let mapped_id =
            AtsIdByHash::<Test>::get(hash_commitment_v1).expect("Hash should still map to ID");
        assert_eq!(mapped_id, ats_id);

        // Verify version 1 is still accessible
        let version_1_still =
            AtsVersions::<Test>::get(ats_id, 1).expect("Version 1 should still exist");
        assert_eq!(version_1_still.hash_commitment, hash_commitment_v1);

        // Check that additional funds are held for the new version
        let final_hold =
            Balances::balance_on_hold(&crate::HoldReason::AtsRegistration.into(), &owner);
        assert_eq!(final_hold, initial_hold + version_cost);
    });
}

#[test]
fn update_ats_non_owner_fails() {
    sp_tracing::init_for_tests();

    new_test_ext().execute_with(|| {
        let owner = 1;
        let non_owner = 2;
        let vk = hex_to_vec(VK_HEX);
        let proof = hex_to_vec(PROOF_HEX);
        let pubs: Vec<[u8; 32]> = PUBS_HEX.iter().map(|h| hex_to_pub(h)).collect();
        let hash_commitment = hex_to_pub(PUBS_HEX[3]);

        // First register the ATS
        assert_ok!(MockAts::register(
            RuntimeOrigin::signed(owner),
            hash_commitment
        ));

        let ats_id = 0;

        // Try to update as non-owner with valid proof (ownership check happens after ZKP verification)
        assert_err!(
            MockAts::update(RuntimeOrigin::signed(non_owner), ats_id, vk, pubs, proof),
            Error::<Test>::VerificationFailed
        );
    });
}

#[test]
fn update_non_existent_ats_fails() {
    sp_tracing::init_for_tests();

    new_test_ext().execute_with(|| {
        let owner = 1;
        let vk = hex_to_vec(VK_HEX);
        let proof = hex_to_vec(PROOF_HEX);
        let pubs: Vec<[u8; 32]> = PUBS_HEX.iter().map(|h| hex_to_pub(h)).collect();

        // Try to update non-existent ATS
        let non_existent_id = 999;
        assert_err!(
            MockAts::update(
                RuntimeOrigin::signed(owner),
                non_existent_id,
                vk,
                pubs,
                proof
            ),
            Error::<Test>::AtsNotFound
        );
    });
}
