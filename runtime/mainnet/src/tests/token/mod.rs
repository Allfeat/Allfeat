#![cfg(test)]

use crate::{tests::new_test_ext, *};
use frame_support::{
    assert_noop, assert_ok,
    traits::{Currency, Hooks, fungible::InspectHold},
};
use frame_system::pallet_prelude::BlockNumberFor;
use pallet_token_allocation::{Allocations, EnvelopeId, HoldReason};
use shared_runtime::currency::AFT;
use sp_keyring::Sr25519Keyring;
use sp_runtime::TokenError;

// --- HELPERS ---

// Helper: read “held” balance on treasury under this pallet’s hold reason
fn held_on_treasury() -> Balance {
    let reason = RuntimeHoldReason::TokenAllocation(HoldReason::TokenAllocation);
    pallet_balances::Pallet::<Runtime>::balance_on_hold(&reason, &Treasury::account_id())
}

fn jump_to(n: BlockNumberFor<Runtime>) {
    frame_system::Pallet::<Runtime>::set_block_number(n);
    let _ = pallet_token_allocation::Pallet::<Runtime>::on_initialize(n);
}

#[test]
fn genesis_issuance_integrity() {
    new_test_ext().execute_with(|| {
        let total_issuance: Balance = pallet_balances::Pallet::<Runtime>::total_issuance();
        let expected: Balance = 1_000_000_000 * AFT;

        assert_eq!(
            total_issuance, expected,
            "CRITICAL: Total issuance at genesis must be exactly 1 Billion AFT"
        );
    })
}

#[test]
fn treasury_allocations_are_correctly_locked() {
    new_test_ext().execute_with(|| {
        // 1. Verify Upfront (Liquid)
        // ResearchDevelopment (20% of 125M) + Reserve (100% of 20M) = 25M + 20M = 45M
        let expected_upfront = (25_000_000 + 20_000_000) * AFT;
        let free_treasury =
            pallet_balances::Pallet::<Runtime>::free_balance(Treasury::account_id());

        assert_eq!(
            free_treasury, expected_upfront,
            "Treasury liquid balance (upfront) mismatch"
        );

        // 2. Verify Locked (Held)
        // Total Treasury Allocations:
        // Community (260) + Exchanges (100) + R&D (125) + Reserve (20) = 505M
        // Minus Upfront (45M) = 460M locked.
        let expected_held = 460_000_000 * AFT;
        let held_treasury = held_on_treasury();

        assert_eq!(
            held_treasury, expected_held,
            "Treasury held balance mismatch"
        );

        // 3. SECURITY CHECK: Treasury cannot spend locked funds
        // Try to transfer more than free balance (e.g. free + 1 AFT from locked)
        let attempt_amount = free_treasury + (AFT);
        let bob = Sr25519Keyring::Bob.to_account_id();

        assert_noop!(
            pallet_balances::Pallet::<Runtime>::transfer_allow_death(
                RuntimeOrigin::signed(Treasury::account_id()),
                bob.into(),
                attempt_amount
            ),
            TokenError::FundsUnavailable
        );
    });
}

#[test]
fn e2e_vesting_schedule_private2() {
    new_test_ext().execute_with(|| {
        let alice = Sr25519Keyring::Alice.to_account_id();

        let alloc_total: u128 = 1_000_000 * AFT;

        let alloc_id = pallet_token_allocation::NextAllocationId::<Runtime>::get();

        // Create Allocation
        assert_ok!(pallet_token_allocation::Pallet::<Runtime>::add_allocation(
            RuntimeOrigin::root(),
            EnvelopeId::Private2,
            alice.clone(),
            alloc_total,
            Some(0),
        ));

        // --- Check Upfront (5%) ---
        let expected_upfront = alloc_total * 5 / 100;
        assert_eq!(
            pallet_balances::Pallet::<Runtime>::free_balance(&alice),
            expected_upfront,
            "Upfront calculation incorrect"
        );

        // --- Check Cliff ---
        let cliff_block = 3 * MONTHS;
        jump_to(cliff_block - 1);

        let alloc = pallet_token_allocation::Allocations::<Runtime>::get(alloc_id)
            .expect("Allocation must exist");

        assert_eq!(alloc.released, 0, "Nothing released before cliff");

        // --- Check Vesting Progress (15 months after cliff) ---
        let check_block = 3 * MONTHS + 15 * MONTHS;
        jump_to(check_block);

        // Force manual payout trigger
        pallet_token_allocation::NextPayoutAt::<Runtime>::put(check_block);
        pallet_token_allocation::Pallet::<Runtime>::on_initialize(check_block);

        // Reload allocation
        let alloc_updated = pallet_token_allocation::Allocations::<Runtime>::get(alloc_id).unwrap();

        let remaining_total = alloc_total - expected_upfront;

        // Math: We expect (15/36) of the remaining amount
        let expected_vested_part = remaining_total
            .saturating_mul(15 * MONTHS as u128)
            .saturating_div(36 * MONTHS as u128);

        let released = alloc_updated.released;

        let diff = released.abs_diff(expected_vested_part);

        let tolerance = AFT;

        assert!(
            diff < tolerance,
            "Vesting math deviation too high. Diff: {diff} raw units (Tolerance: {tolerance} raw units)"
        );

        // --- Check Completion ---
        let end_block = 3 * MONTHS + 36 * MONTHS + MONTHS;

        pallet_token_allocation::NextPayoutAt::<Runtime>::put(end_block);
        pallet_token_allocation::Pallet::<Runtime>::on_initialize(end_block);

        assert!(
            pallet_token_allocation::Allocations::<Runtime>::get(alloc_id).is_none(),
            "Allocation should be pruned from storage"
        );

        let final_balance = pallet_balances::Pallet::<Runtime>::free_balance(&alice);

        let dust = alloc_total.saturating_sub(final_balance);
        assert!(
            dust <= 1,
            "Alice should have full amount at the end. Missing: {dust} raw units"
        );
    });
}

#[test]
fn ensure_no_unexpected_allocations() {
    new_test_ext().execute_with(|| {
        let count = Allocations::<Runtime>::iter_keys().count();
        assert_eq!(
            count, 3,
            "Should have exactly 3 auto-allocations for Treasury"
        );

        for (_, alloc) in Allocations::<Runtime>::iter() {
            assert_eq!(alloc.beneficiary, Treasury::account_id());
        }
    });
}

#[test]
fn ensure_vesting_really_locks_funds_for_users() {
    new_test_ext().execute_with(|| {
        let alice = Sr25519Keyring::Alice.to_account_id();
        let bob = Sr25519Keyring::Bob.to_account_id();

        let alloc_total: u128 = 1_000 * AFT;

        // Allocation : Private2 (5% Upfront = 50 AFT)
        assert_ok!(pallet_token_allocation::Pallet::<Runtime>::add_allocation(
            RuntimeOrigin::root(),
            EnvelopeId::Private2,
            alice.clone(),
            alloc_total,
            Some(0),
        ));

        let total_balance = pallet_balances::Pallet::<Runtime>::total_balance(&alice);
        assert_eq!(total_balance, alloc_total, "Alice should see total balance");

        let free_balance = pallet_balances::Pallet::<Runtime>::free_balance(&alice);
        let expected_free = 50 * AFT;
        assert_eq!(free_balance, expected_free);

        assert_noop!(
            pallet_balances::Pallet::<Runtime>::transfer_allow_death(
                RuntimeOrigin::signed(alice.clone()),
                bob.clone().into(),
                expected_free + (AFT)
            ),
            TokenError::FundsUnavailable
        );

        assert_ok!(pallet_balances::Pallet::<Runtime>::transfer_allow_death(
            RuntimeOrigin::signed(alice.clone()),
            bob.clone().into(),
            expected_free
        ));

        assert_eq!(pallet_balances::Pallet::<Runtime>::free_balance(&alice), 0);
        assert_eq!(
            pallet_balances::Pallet::<Runtime>::total_balance(&alice),
            950 * AFT
        );
    });
}
