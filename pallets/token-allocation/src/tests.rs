// tests.rs

use crate::{EnvelopeConfig, EnvelopeId, Error, mock::*};
use frame_support::{
    assert_noop, assert_ok,
    traits::fungible::{InspectHold, Mutate},
};
use sp_runtime::Percent;

// --- HELPER ---
// Reduces test verbosity by centralizing configuration.
fn setup_and_fund_envelope(
    id: EnvelopeId,
    cap: u128,
    upfront: u8,
    cliff: u64,
    duration: u64,
    unique: Option<u128>,
) {
    let config = EnvelopeConfig {
        total_cap: cap,
        upfront_rate: Percent::from_percent(upfront),
        cliff,
        vesting_duration: duration,
        unique_beneficiary: unique,
    };

    // Direct insertion into storage (simulates Genesis or prior setup).
    crate::Envelopes::<Test>::insert(id, config);
    crate::EnvelopeDistributed::<Test>::insert(id, 0);

    // Mint funds to the envelope account so it can distribute them.
    let envelope_acc = id.account::<Test>();
    let _ = Balances::mint_into(&envelope_acc, cap);
}

// --- TESTS ---

#[test]
fn full_lifecycle_works() {
    // Tests the standard scenario: Upfront -> Cliff -> Progressive Vesting -> Finish.
    new_test_ext(vec![], vec![]).execute_with(|| {
        // Config: 1000 tokens, 10% upfront, Cliff at block 10, Duration 100 blocks.
        setup_and_fund_envelope(EnvelopeId::Public2, 1000, 10, 10, 100, None);
        let ben = 1u128;

        // 1. Create allocation
        assert_ok!(TokenAllocation::add_allocation(
            RuntimeOrigin::root(),
            EnvelopeId::Public2,
            ben,
            1000,
            None // Start default (Cliff)
        ));

        // CHECK 1: Upfront paid immediately.
        // 10% of 1000 = 100 Free. The rest (900) is Held.
        assert_eq!(
            Balances::free_balance(ben),
            100,
            "Upfront should be free immediately"
        );
        assert_eq!(
            Balances::total_balance_on_hold(&ben),
            900,
            "Remaining should be held"
        );

        // 2. Advance before the Cliff (Block 5).
        run_to_block(5);
        // Nothing should change.
        assert_eq!(Balances::free_balance(ben), 100);

        // 3. Advance to middle of vesting (Block 60: 10 cliff + 50 duration).
        // Elapsed time = 50 blocks out of 100 = 50% of the remainder.
        // Remaining to vest = 900. 50% of 900 = 450.
        // Total Free = 100 (upfront) + 450 (vested) = 550.
        run_to_block(60);
        assert_eq!(Balances::free_balance(ben), 550, "Upfront + 50% vested");
        assert_eq!(Balances::total_balance_on_hold(&ben), 450);

        // 4. Finish vesting (Block 120).
        run_to_block(120);
        assert_eq!(
            Balances::free_balance(ben),
            1000,
            "All funds should be free"
        );
        assert_eq!(Balances::total_balance_on_hold(&ben), 0, "No funds held");

        // CHECK 2: Storage cleanup.
        assert_eq!(
            crate::Allocations::<Test>::iter_keys().count(),
            0,
            "Allocation should be removed after full vesting"
        );
    });
}

#[test]
fn pagination_logic_handles_overflowing_allocations() {
    // Tests the 'Push' mechanism when there are more allocations than the block limit.
    new_test_ext(vec![], vec![]).execute_with(|| {
        // Mock Config: MaxPayoutPerBlock = 5 (defined in mock.rs).

        // Config: Instant vesting (duration 1) to force payout at every tick.
        setup_and_fund_envelope(EnvelopeId::Airdrop, 1_000_000, 0, 0, 1, None);

        // Create 7 allocations (7 > 5) that must all pay out at block 100.
        for i in 0..7 {
            assert_ok!(TokenAllocation::add_allocation(
                RuntimeOrigin::root(),
                EnvelopeId::Airdrop,
                i + 100, // Unique IDs
                100,
                Some(99) // Start vesting at block 99
            ));
        }

        // Force next payout at block 100.
        crate::NextPayoutAt::<Test>::put(100);
        crate::EpochIndex::<Test>::put(0);

        // --- EXECUTION BLOCK 100 ---
        run_to_block(100);

        // CHECK 1: Pagination active.
        // Should have 2 remaining (7 - 5 processed).
        assert_eq!(
            crate::Allocations::<Test>::iter_keys().count(),
            2,
            "Should have 2 allocations remaining"
        );
        // Cursor must be set to resume in next block.
        assert!(
            crate::PayoutCursor::<Test>::get().is_some(),
            "Cursor should be set"
        );
        // Epoch should NOT have changed yet.
        assert_eq!(
            crate::EpochIndex::<Test>::get(),
            0,
            "Epoch should not increment yet"
        );

        // --- EXECUTION BLOCK 101 ---
        run_to_block(101);

        // CHECK 2: Processing finished.
        assert_eq!(
            crate::Allocations::<Test>::iter_keys().count(),
            0,
            "All allocations processed"
        );
        assert!(
            crate::PayoutCursor::<Test>::get().is_none(),
            "Cursor cleared"
        );
        // Epoch changed, next payout scheduled.
        assert_eq!(crate::EpochIndex::<Test>::get(), 1);
    });
}

#[test]
fn math_is_safe_with_u256() {
    // Tests protection against mathematical overflow.
    new_test_ext(vec![], vec![]).execute_with(|| {
        let huge_cap = u128::MAX / 10;
        // Very long duration.
        setup_and_fund_envelope(EnvelopeId::Reserve, huge_cap, 0, 0, 1_000_000, None);

        let ben = 1u128;
        assert_ok!(TokenAllocation::add_allocation(
            RuntimeOrigin::root(),
            EnvelopeId::Reserve,
            ben,
            huge_cap,
            Some(0)
        ));

        assert_eq!(
            Balances::total_balance_on_hold(&ben),
            huge_cap,
            "Funds should be held"
        );

        // Advance to half duration.
        // If code did (Amount * Time), it would be u128::MAX * 500_000 -> Immediate Panic in pure u128.
        // With U256, this must pass.
        run_to_block(500_000);

        // We should receive approximately half.
        let free = Balances::free_balance(ben);
        let expected = huge_cap / 2;

        // Minimal rounding error tolerance.
        let diff = free.abs_diff(expected);
        assert!(diff <= 1, "Math should result in ~50% of u128::MAX");
    });
}

#[test]
fn constraints_are_enforced() {
    // Tests limits (Caps) and business rules.
    new_test_ext(vec![], vec![]).execute_with(|| {
        // 1. Test CAP
        setup_and_fund_envelope(EnvelopeId::Public1, 1000, 0, 0, 100, None);

        // Allocation OK (1000 <= 1000).
        assert_ok!(TokenAllocation::add_allocation(
            RuntimeOrigin::root(),
            EnvelopeId::Public1,
            1,
            1000,
            None
        ));

        // Allocation fails (Cap exceeded because 1000 already distributed).
        assert_noop!(
            TokenAllocation::add_allocation(RuntimeOrigin::root(), EnvelopeId::Public1, 2, 1, None),
            Error::<Test>::EnvelopeCapExceeded
        );

        // 2. Test Unique Beneficiary
        // Config with enforced beneficiary (e.g., ID 99).
        setup_and_fund_envelope(EnvelopeId::Teams, 1000, 0, 0, 100, Some(99));

        // Try to allocate to someone else.
        assert_noop!(
            TokenAllocation::add_allocation(
                RuntimeOrigin::root(),
                EnvelopeId::Teams,
                50,
                100,
                None
            ),
            Error::<Test>::AllocationDisabled
        );

        // Even if we try to allocate manually to the correct user,
        // the current code returns AllocationDisabled if unique_beneficiary is set
        // (assuming it's handled by genesis or special logic).
        assert_noop!(
            TokenAllocation::add_allocation(
                RuntimeOrigin::root(),
                EnvelopeId::Teams,
                99,
                100,
                None
            ),
            Error::<Test>::AllocationDisabled
        );
    });
}

#[test]
fn permission_checks() {
    new_test_ext(vec![], vec![]).execute_with(|| {
        setup_and_fund_envelope(EnvelopeId::Public2, 1000, 0, 0, 100, None);

        // A normal user cannot create an allocation.
        assert_noop!(
            TokenAllocation::add_allocation(
                RuntimeOrigin::signed(1), // Not root
                EnvelopeId::Public2,
                2,
                100,
                None
            ),
            sp_runtime::DispatchError::BadOrigin
        );
    });
}
