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

use crate::{
    mock::*,
    types::{EnvelopeConfig, EnvelopeType, EnvelopeWallet},
};
use frame_support::{
    assert_ok,
    sp_runtime::Percent,
    traits::fungible::{Inspect, Mutate},
};

#[test]
fn basic_envelope_setup_works() {
    new_test_ext().execute_with(|| {
        let total_allocation = 1000;

        let config = EnvelopeConfig {
            immediate_unlock_percentage: Percent::from_percent(10),
            cliff_duration: 100,
            vesting_duration: 1000,
        };

        // Generate the account automatically and mint tokens to it
        let envelope_account = crate::Pallet::<Test>::envelope_account_id(&EnvelopeType::Seed);
        assert_ok!(Balances::mint_into(&envelope_account, total_allocation));

        // Setup envelope wallet
        let envelope_wallet = EnvelopeWallet {
            distributed_amount: 0,
        };

        crate::EnvelopeWallets::<Test>::insert(&EnvelopeType::Seed, &envelope_wallet);
        crate::EnvelopeConfigs::<Test>::insert(&EnvelopeType::Seed, &config);

        // Verify storage
        assert_eq!(
            crate::EnvelopeWallets::<Test>::get(&EnvelopeType::Seed),
            Some(envelope_wallet)
        );
        assert_eq!(
            crate::EnvelopeConfigs::<Test>::get(&EnvelopeType::Seed),
            Some(config)
        );

        // Verify envelope account has the tokens
        assert_eq!(Balances::balance(&envelope_account), total_allocation);
    });
}

#[test]
fn allocate_from_envelope_works() {
    new_test_ext().execute_with(|| {
        let beneficiary = 2;
        let total_allocation = 1000;
        let allocation_amount = 100;

        let config = EnvelopeConfig {
            immediate_unlock_percentage: Percent::from_percent(10),
            cliff_duration: 100,
            vesting_duration: 1000,
        };

        // Setup
        let envelope_account = crate::Pallet::<Test>::envelope_account_id(&EnvelopeType::Seed);
        assert_ok!(Balances::mint_into(&envelope_account, total_allocation));

        let envelope_wallet = EnvelopeWallet {
            distributed_amount: 0,
        };

        crate::EnvelopeWallets::<Test>::insert(&EnvelopeType::Seed, &envelope_wallet);
        crate::EnvelopeConfigs::<Test>::insert(&EnvelopeType::Seed, &config);

        // Allocate from envelope
        assert_ok!(crate::Pallet::<Test>::allocate_from_envelope(
            RuntimeOrigin::root(),
            EnvelopeType::Seed,
            beneficiary,
            allocation_amount,
            None, // start immediately
        ));

        // Verify allocation was created
        let allocation = crate::Allocations::<Test>::get(beneficiary, 0);
        assert!(allocation.is_some());
        let allocation = allocation.unwrap();
        assert_eq!(allocation.total_allocation, allocation_amount);
        assert_eq!(allocation.envelope_type, EnvelopeType::Seed);

        // Verify wallet state updated
        let updated_wallet = crate::EnvelopeWallets::<Test>::get(&EnvelopeType::Seed).unwrap();
        assert_eq!(updated_wallet.distributed_amount, allocation_amount);

        // Verify envelope account still has all tokens (no immediate transfer)
        let envelope_account = crate::Pallet::<Test>::envelope_account_id(&EnvelopeType::Seed);
        let remaining_balance = Balances::balance(&envelope_account);
        assert_eq!(remaining_balance, total_allocation); // Tokens stay for vesting
    });
}

#[test]
fn claim_tokens_works() {
    new_test_ext().execute_with(|| {
        let beneficiary = 2;
        let total_allocation = 1000;
        let allocation_amount = 100;

        let config = EnvelopeConfig {
            immediate_unlock_percentage: Percent::from_percent(50), // 50% immediate
            cliff_duration: 0,                                      // No cliff to simplify
            vesting_duration: 100,
        };

        // Setup
        let envelope_account = crate::Pallet::<Test>::envelope_account_id(&EnvelopeType::Seed);
        assert_ok!(Balances::mint_into(&envelope_account, total_allocation));

        let envelope_wallet = EnvelopeWallet {
            distributed_amount: 0,
        };

        crate::EnvelopeWallets::<Test>::insert(&EnvelopeType::Seed, &envelope_wallet);
        crate::EnvelopeConfigs::<Test>::insert(&EnvelopeType::Seed, &config);

        // Allocate
        assert_ok!(crate::Pallet::<Test>::allocate_from_envelope(
            RuntimeOrigin::root(),
            EnvelopeType::Seed,
            beneficiary,
            allocation_amount,
            None,
        ));

        // Claim tokens
        assert_ok!(crate::Pallet::<Test>::claim_tokens(
            RuntimeOrigin::signed(beneficiary),
            0, // allocation_id
        ));

        // Verify claimed amount updated
        let allocation = crate::Allocations::<Test>::get(beneficiary, 0).unwrap();
        assert_eq!(allocation.claimed_amount, 50); // 50% of 100
    });
}

#[test]
fn prevent_over_allocation() {
    new_test_ext().execute_with(|| {
        let total_allocation = 20; // Small envelope
        let first_allocation = 10;
        let second_allocation = 11; // This should fail

        let config = EnvelopeConfig {
            immediate_unlock_percentage: Percent::from_percent(10),
            cliff_duration: 100,
            vesting_duration: 1000,
        };

        // Setup envelope with only 20 tokens
        let envelope_account = crate::Pallet::<Test>::envelope_account_id(&EnvelopeType::Seed);
        assert_ok!(Balances::mint_into(&envelope_account, total_allocation));

        let envelope_wallet = EnvelopeWallet {
            distributed_amount: 0,
        };

        crate::EnvelopeWallets::<Test>::insert(&EnvelopeType::Seed, &envelope_wallet);
        crate::EnvelopeConfigs::<Test>::insert(&EnvelopeType::Seed, &config);

        // First allocation should work (10 out of 20)
        assert_ok!(crate::Pallet::<Test>::allocate_from_envelope(
            RuntimeOrigin::root(),
            EnvelopeType::Seed,
            2,
            first_allocation,
            None,
        ));

        // Second allocation should fail (11 out of remaining 10)
        assert!(
            crate::Pallet::<Test>::allocate_from_envelope(
                RuntimeOrigin::root(),
                EnvelopeType::Seed,
                3,
                second_allocation,
                None,
            )
            .is_err()
        );

        // Verify only first allocation was created
        assert!(crate::Allocations::<Test>::get(2, 0).is_some());
        assert!(crate::Allocations::<Test>::get(3, 0).is_none());

        // Verify distributed amount is only from first allocation
        let wallet = crate::EnvelopeWallets::<Test>::get(&EnvelopeType::Seed).unwrap();
        assert_eq!(wallet.distributed_amount, first_allocation);
    });
}

#[test]
fn update_envelope_config_affects_claim_schedule() {
    new_test_ext().execute_with(|| {
        let beneficiary = 2;
        let total = 1_000;
        let amount = 200;

        let config_initial = EnvelopeConfig {
            immediate_unlock_percentage: Percent::from_percent(0),
            cliff_duration: 100,
            vesting_duration: 200,
        };

        let envelope_account = crate::Pallet::<Test>::envelope_account_id(&EnvelopeType::Seed);
        assert_ok!(Balances::mint_into(&envelope_account, total));
        let wallet = EnvelopeWallet {
            distributed_amount: 0,
        };
        crate::EnvelopeWallets::<Test>::insert(&EnvelopeType::Seed, &wallet);
        crate::EnvelopeConfigs::<Test>::insert(&EnvelopeType::Seed, &config_initial);

        assert_ok!(crate::Pallet::<Test>::allocate_from_envelope(
            RuntimeOrigin::root(),
            EnvelopeType::Seed,
            beneficiary,
            amount,
            None,
        ));

        let allocation = crate::Allocations::<Test>::get(beneficiary, 0).unwrap();
        assert_eq!(allocation.claimed_amount, 0);

        let config_updated = EnvelopeConfig {
            immediate_unlock_percentage: Percent::from_percent(50),
            cliff_duration: 0,
            vesting_duration: 100,
        };
        assert_ok!(crate::Pallet::<Test>::update_envelope_config(
            RuntimeOrigin::root(),
            EnvelopeType::Seed,
            config_updated.clone(),
        ));

        assert_ok!(crate::Pallet::<Test>::claim_tokens(
            RuntimeOrigin::signed(beneficiary),
            0
        ));
        let allocation = crate::Allocations::<Test>::get(beneficiary, 0).unwrap();
        assert_eq!(allocation.claimed_amount, 100);
    });
}

#[test]
fn update_allocation_adjusts_reservation_and_start() {
    new_test_ext().execute_with(|| {
        let beneficiary = 2;
        let total = 200;
        let config = EnvelopeConfig {
            immediate_unlock_percentage: Percent::from_percent(0),
            cliff_duration: 0,
            vesting_duration: 100,
        };
        let envelope_account = crate::Pallet::<Test>::envelope_account_id(&EnvelopeType::Seed);
        assert_ok!(Balances::mint_into(&envelope_account, total));
        let wallet = EnvelopeWallet {
            distributed_amount: 0,
        };
        crate::EnvelopeWallets::<Test>::insert(&EnvelopeType::Seed, &wallet);
        crate::EnvelopeConfigs::<Test>::insert(&EnvelopeType::Seed, &config);

        assert_ok!(crate::Pallet::<Test>::allocate_from_envelope(
            RuntimeOrigin::root(),
            EnvelopeType::Seed,
            beneficiary,
            100,
            None
        ));
        let wallet = crate::EnvelopeWallets::<Test>::get(&EnvelopeType::Seed).unwrap();
        assert_eq!(wallet.distributed_amount, 100);

        assert_ok!(crate::Pallet::<Test>::update_allocation(
            RuntimeOrigin::root(),
            beneficiary,
            0,
            150,
            None
        ));
        let wallet = crate::EnvelopeWallets::<Test>::get(&EnvelopeType::Seed).unwrap();
        assert_eq!(wallet.distributed_amount, 150);

        assert_ok!(crate::Pallet::<Test>::update_allocation(
            RuntimeOrigin::root(),
            beneficiary,
            0,
            120,
            Some(1)
        ));
        let wallet = crate::EnvelopeWallets::<Test>::get(&EnvelopeType::Seed).unwrap();
        assert_eq!(wallet.distributed_amount, 120);

        let alloc = crate::Allocations::<Test>::get(beneficiary, 0).unwrap();
        match alloc.status {
            crate::types::AllocationStatus::ActiveSinceGenesis => (),
            crate::types::AllocationStatus::ActivatedAt(b) if b <= 1 => (),
            _ => panic!(),
        }
    });
}

#[test]
fn revoke_allocation_releases_unclaimed() {
    new_test_ext().execute_with(|| {
        let beneficiary = 2;
        let total = 300;
        let config = EnvelopeConfig {
            immediate_unlock_percentage: Percent::from_percent(50),
            cliff_duration: 0,
            vesting_duration: 100,
        };
        let envelope_account = crate::Pallet::<Test>::envelope_account_id(&EnvelopeType::Seed);
        assert_ok!(Balances::mint_into(&envelope_account, total));
        let wallet = EnvelopeWallet {
            distributed_amount: 0,
        };
        crate::EnvelopeWallets::<Test>::insert(&EnvelopeType::Seed, &wallet);
        crate::EnvelopeConfigs::<Test>::insert(&EnvelopeType::Seed, &config);

        assert_ok!(crate::Pallet::<Test>::allocate_from_envelope(
            RuntimeOrigin::root(),
            EnvelopeType::Seed,
            beneficiary,
            200,
            None
        ));
        let wallet = crate::EnvelopeWallets::<Test>::get(&EnvelopeType::Seed).unwrap();
        assert_eq!(wallet.distributed_amount, 200);

        assert_ok!(crate::Pallet::<Test>::claim_tokens(
            RuntimeOrigin::signed(beneficiary),
            0
        ));
        let alloc = crate::Allocations::<Test>::get(beneficiary, 0).unwrap();
        assert_eq!(alloc.claimed_amount, 100);

        assert_ok!(crate::Pallet::<Test>::revoke_allocation(
            RuntimeOrigin::root(),
            beneficiary,
            0
        ));
        let wallet = crate::EnvelopeWallets::<Test>::get(&EnvelopeType::Seed).unwrap();
        assert_eq!(wallet.distributed_amount, 100);
        let alloc = crate::Allocations::<Test>::get(beneficiary, 0).unwrap();
        match alloc.status {
            crate::types::AllocationStatus::Revoked => (),
            _ => panic!(),
        }
        assert_eq!(alloc.total_allocation, 100);
    });
}
