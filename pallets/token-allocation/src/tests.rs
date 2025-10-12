use super::*;
use crate::{
    mock::{EpochDuration, RuntimeHoldReason, RuntimeOrigin, run_to_block},
    pallet::{Allocations, Event as PalletEvent, PayoutCursor},
};
use frame_support::{
    assert_noop, assert_ok,
    traits::{Currency, fungible::InspectHold},
};
use sp_runtime::Percent;

use crate::mock::{
    Balances, RuntimeEvent as TestEvent, System, Test, TokenAllocation, new_test_ext,
};

fn hold_reason() -> RuntimeHoldReason {
    RuntimeHoldReason::TokenAllocation(HoldReason::TokenAllocation)
}

#[test]
fn add_allocation_pays_upfront_and_holds_rest() {
    let total_cap: u64 = 1_000_000;
    let env = EnvelopeConfig {
        total_cap,
        upfront_rate: Percent::from_percent(10),
        cliff: 10,
        vesting_duration: 100,
        unique_beneficiary: None,
    };

    let mut ext = new_test_ext(vec![(EnvelopeId::Seed, env.clone())], vec![]);

    ext.execute_with(|| {
        let alice = 100u128;
        let alloc = 500_000u64;

        let src = EnvelopeId::Seed.account::<Test>();
        pallet_balances::Pallet::<Test>::make_free_balance_be(&src, total_cap);

        // add_allocation (Root)
        assert_ok!(TokenAllocation::add_allocation(
            RuntimeOrigin::root(),
            EnvelopeId::Seed,
            alice,
            alloc,
            Some(0),
        ));

        // upfront = 10% of 500_000 = 50_000
        let upfront = env.upfront_rate.mul_floor(alloc);
        assert_eq!(Balances::free_balance(alice), upfront);

        // the rest is held
        let held = pallet_balances::Pallet::<Test>::balance_on_hold(&hold_reason(), &alice);
        assert_eq!(held, alloc - upfront);

        // allocation stored correctly
        let a = Allocations::<Test>::get((EnvelopeId::Seed, alice)).expect("alloc");
        assert_eq!(a.total, alloc);
        assert_eq!(a.upfront, upfront);
        assert_eq!(a.vested_total, alloc - upfront);
        assert_eq!(a.released, 0);
        assert_eq!(a.start, 0);
    });
}

// -----------------------------------------------------------------------------
// 2) Epoch payout releases linearly and completes (allocation removed)
// -----------------------------------------------------------------------------
#[test]
fn epoch_payout_releases_linearly_and_completes() {
    let total_cap: u64 = 2_000_000;
    let env = EnvelopeConfig {
        total_cap,
        upfront_rate: Percent::from_percent(20),
        cliff: 2,
        vesting_duration: 10,
        unique_beneficiary: None,
    };

    let mut ext = new_test_ext(vec![(EnvelopeId::ICO1, env.clone())], vec![]);
    ext.execute_with(|| {
        let alice = 200u128;
        let alloc = 1_000_000u64;

        // Fund the envelope sub-account
        let src = EnvelopeId::ICO1.account::<Test>();
        pallet_balances::Pallet::<Test>::make_free_balance_be(&src, total_cap);

        // Add allocation (Root)
        assert_ok!(TokenAllocation::add_allocation(
            RuntimeOrigin::root(),
            EnvelopeId::ICO1,
            alice,
            alloc,
            Some(0),
        ));

        // Before cliff: nothing released
        run_to_block(1);
        let a1 = Allocations::<Test>::get((EnvelopeId::ICO1, alice)).unwrap();
        assert_eq!(a1.released, 0);

        // Reach first epoch (EpochDuration=5 in mock) after cliff
        run_to_block(5);
        let a2 = Allocations::<Test>::get((EnvelopeId::ICO1, alice)).unwrap();
        assert!(
            a2.released > 0 && a2.released < a2.vested_total,
            "should be partially released"
        );

        // Go far enough so vesting completes and allocation is pruned
        run_to_block(30);
        assert!(
            Allocations::<Test>::get((EnvelopeId::ICO1, alice)).is_none(),
            "completed allocation must be removed"
        );

        // Sanity: free balance increased beyond upfront
        let upfront = env.upfront_rate.mul_floor(alloc);
        assert!(Balances::free_balance(alice) > upfront);

        // Optional: ensure at least one EpochPayout event was emitted
        let has_epoch_event = System::events().iter().any(|e| {
            matches!(
                e.event,
                TestEvent::TokenAllocation(PalletEvent::EpochPayout { .. })
            )
        });
        assert!(has_epoch_event, "should emit EpochPayout at least once");
    });
}

// -----------------------------------------------------------------------------
// 3) Pagination respects MaxPayoutPerBlock and cursor doesn't reprocess
// -----------------------------------------------------------------------------
#[test]
fn pagination_continues_next_block_not_next_epoch() {
    // MaxPayoutPerBlock = 5 (mock). Create 6 allocations so we need 2 blocks within the same epoch.
    let total_cap: u64 = 10_000_000;
    let env = EnvelopeConfig {
        total_cap,
        upfront_rate: Percent::from_percent(0),
        cliff: 0,
        vesting_duration: 20,
        unique_beneficiary: None,
    };

    let mut ext = new_test_ext(vec![(EnvelopeId::Airdrop, env.clone())], vec![]);
    ext.execute_with(|| {
        let src = EnvelopeId::Airdrop.account::<Test>();
        pallet_balances::Pallet::<Test>::make_free_balance_be(&src, total_cap);

        // 6 allocations
        for i in 0..6u128 {
            let who = 3000u128 + i;
            assert_ok!(TokenAllocation::add_allocation(
                RuntimeOrigin::root(),
                EnvelopeId::Airdrop,
                who,
                1_000u64,
                Some(0),
            ));
        }

        // 1) First epoch tick at block 5 processes at most 5 items and leaves a cursor.
        run_to_block(5);
        let cur1 = PayoutCursor::<Test>::get();
        assert!(
            cur1.is_some(),
            "cursor should exist when there is remaining work"
        );

        // Snapshot released after block 5 (some accounts advanced, one still pending).
        let released_after_5 = |acc: u128| {
            Allocations::<Test>::get((EnvelopeId::Airdrop, acc))
                .map(|a| a.released)
                .unwrap_or(0)
        };
        let r5: Vec<u64> = (0..6u128).map(|i| released_after_5(3000 + i)).collect();

        // 2) Next block (6) must continue processing (same epoch), finish the last item,
        //    close the epoch, clear cursor, and push NextPayoutAt to 6 + EpochDuration (= 11).
        run_to_block(6);
        let cur2 = PayoutCursor::<Test>::get();
        assert!(
            cur2.is_none(),
            "cursor must be cleared once the scan completes"
        );

        // Check NextPayoutAt moved to 6 + EpochDuration (EpochDuration = 5 in mock)
        let expected_next = 6 + EpochDuration::get();
        let next_at = crate::pallet::NextPayoutAt::<Test>::get();
        assert_eq!(
            next_at, expected_next,
            "NextPayoutAt should be set right after completion"
        );

        // Snapshot released after block 6.
        let r6: Vec<u64> = (0..6u128).map(|i| released_after_5(3000 + i)).collect();

        // Everyone should be >= their values at block 5 (monotonic).
        for (a5, a6) in r5.iter().zip(r6.iter()) {
            assert!(a6 >= a5, "released must be monotonic between 5 -> 6");
        }

        // 3) Between blocks 7..10, on_initialize should *not* run (we're before NextPayoutAt=11),
        //    so released amounts must remain unchanged.
        run_to_block(10);
        let r10: Vec<u64> = (0..6u128).map(|i| released_after_5(3000 + i)).collect();
        assert_eq!(r10, r6, "released should not change before NextPayoutAt");

        // 4) At block 11 (new epoch), payouts resume and released must increase (if vesting remains).
        run_to_block(11);
        let r11: Vec<u64> = (0..6u128).map(|i| released_after_5(3000 + i)).collect();
        let any_increased = r11.iter().zip(r10.iter()).any(|(n, p)| n > p);
        assert!(
            any_increased,
            "released should increase again at the next epoch start (block 11)"
        );
    });
}

// -----------------------------------------------------------------------------
// 4) Upfront 100% finishes immediately and allocation disappears
// -----------------------------------------------------------------------------
#[test]
fn upfront_100_percent_finishes_immediately_and_disappears() {
    let total_cap: u64 = 1_000_000;
    let env = EnvelopeConfig {
        total_cap,
        upfront_rate: Percent::from_percent(100),
        cliff: 0,
        vesting_duration: 0,
        unique_beneficiary: None,
    };

    let mut ext = new_test_ext(vec![(EnvelopeId::Exchanges, env.clone())], vec![]);
    ext.execute_with(|| {
        let alice = 909u128;
        let src = EnvelopeId::Exchanges.account::<Test>();
        pallet_balances::Pallet::<Test>::make_free_balance_be(&src, total_cap);

        assert_ok!(TokenAllocation::add_allocation(
            RuntimeOrigin::root(),
            EnvelopeId::Exchanges,
            alice,
            500_000u64,
            Some(0),
        ));

        // With 100% upfront, vesting_total == 0, allocation should be removed on first epoch pass
        run_to_block(5);
        assert!(
            Allocations::<Test>::get((EnvelopeId::Exchanges, alice)).is_none(),
            "100% upfront allocation must not persist"
        );
    });
}

// -----------------------------------------------------------------------------
// 5) Envelope cap is enforced
// -----------------------------------------------------------------------------
#[test]
fn envelope_cap_enforced() {
    let total_cap: u64 = 50_000;
    let env = EnvelopeConfig {
        total_cap,
        upfront_rate: Percent::from_percent(0),
        cliff: 0,
        vesting_duration: 10,
        unique_beneficiary: None,
    };

    let mut ext = new_test_ext(vec![(EnvelopeId::Private1, env.clone())], vec![]);
    ext.execute_with(|| {
        let bob = 777u128;
        let charlie = 888u128;
        let src = EnvelopeId::Private1.account::<Test>();
        pallet_balances::Pallet::<Test>::make_free_balance_be(&src, total_cap);

        // First allocation fills the cap
        assert_ok!(TokenAllocation::add_allocation(
            RuntimeOrigin::root(),
            EnvelopeId::Private1,
            bob,
            total_cap,
            Some(0),
        ));

        // Second allocation should fail with EnvelopeCapExceeded
        let err = TokenAllocation::add_allocation(
            RuntimeOrigin::root(),
            EnvelopeId::Private1,
            charlie,
            1u64,
            Some(0),
        )
        .unwrap_err();

        // Match pallet error
        assert_eq!(
            err,
            sp_runtime::DispatchError::from(pallet::Error::<Test>::EnvelopeCapExceeded)
        );
    });
}

// -----------------------------------------------------------------------------
// 6) Unique beneficiary disables runtime allocations
// -----------------------------------------------------------------------------
#[test]
fn unique_beneficiary_disables_runtime_allocations() {
    let total_cap: u64 = 100_000;
    let enforced = 42u128;

    let env = EnvelopeConfig {
        total_cap,
        upfront_rate: Percent::from_percent(0),
        cliff: 0,
        vesting_duration: 10,
        unique_beneficiary: Some(enforced),
    };

    let mut ext = new_test_ext(vec![(EnvelopeId::Reserve, env.clone())], vec![]);
    ext.execute_with(|| {
        // Fund envelope
        let src = EnvelopeId::Reserve.account::<Test>();
        pallet_balances::Pallet::<Test>::make_free_balance_be(&src, total_cap);

        // Any runtime add_allocation must be disabled when unique_beneficiary is set
        assert_noop!(
            TokenAllocation::add_allocation(
                RuntimeOrigin::root(),
                EnvelopeId::Reserve,
                999u128, // different from enforced
                10_000u64,
                Some(0),
            ),
            pallet::Error::<Test>::AllocationDisabled
        );

        // Even for the same enforced account, runtime is disabled (genesis-only)
        assert_noop!(
            TokenAllocation::add_allocation(
                RuntimeOrigin::root(),
                EnvelopeId::Reserve,
                enforced,
                10_000u64,
                Some(0),
            ),
            pallet::Error::<Test>::AllocationDisabled
        );
    });
}
