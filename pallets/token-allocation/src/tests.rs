use super::*;
use crate::pallet::{Allocations, EnvelopeDistributed, Envelopes, Event as PalletEvent};
use frame_support::{assert_noop, assert_ok, traits::Currency};
use sp_runtime::Percent;

use crate::mock::{
    Balances, RuntimeEvent as TestEvent, System, Test, TokenAllocation, new_test_ext,
};

fn set_block(n: u64) {
    System::set_block_number(n);
}

#[test]
fn add_allocation_pays_upfront_and_stores_allocation() {
    new_test_ext().execute_with(|| {
        let id = EnvelopeId::Founders;
        let total_cap: u64 = 1_000_000;
        let upfront_rate: Percent = Percent::from_percent(10);
        let cliff: u64 = 10;
        let duration: u64 = 100;

        // fund envelope source account sufficiently: upfront + vested_total == total
        let source = id.account::<Test>();
        // give source large balance
        Balances::make_free_balance_be(&source, 500_000);

        // Insert envelope config in storage (as done by genesis build normally)
        Envelopes::<Test>::insert(
            id,
            EnvelopeConfig::<u64, u64> {
                total_cap,
                upfront_rate,
                cliff,
                vesting_duration: duration,
            },
        );
        EnvelopeDistributed::<Test>::insert(id, 0u64);

        let who = 1u128;
        let total: u64 = 100_000;

        let before_who = Balances::free_balance(who);

        assert_ok!(TokenAllocation::add_allocation(
            mock::RuntimeOrigin::root(),
            id,
            who,
            total
        ));

        // upfront should be 10_000 (10%)
        let upfront = 10_000u64;
        assert_eq!(Balances::free_balance(who), before_who + upfront);

        // allocation stored
        let alloc = Allocations::<Test>::get(id, who).expect("allocation exists");
        assert_eq!(alloc.total, total);
        assert_eq!(alloc.upfront, upfront);
        assert_eq!(alloc.vested_total, total - upfront);
        assert_eq!(alloc.released, 0);

        // events include UpfrontPaid then AllocationAdded
        let events: Vec<TestEvent> = System::events().into_iter().map(|r| r.event).collect();
        assert!(events.iter().any(|e| matches!(e,
            TestEvent::TokenAllocation(PalletEvent::UpfrontPaid(eid, account, amount)) if eid == &id && account == &who && amount == &upfront
        )));
        assert!(events.iter().any(|e| matches!(e,
            TestEvent::TokenAllocation(PalletEvent::AllocationAdded(eid, account, amount)) if eid == &id && account == &who && amount == &total
        )));
    });
}

#[test]
fn cannot_over_cap_envelope() {
    new_test_ext().execute_with(|| {
        let id = EnvelopeId::KoL;
        let total_cap: u64 = 150;
        let upfront_rate: Percent = Percent::from_percent(0);
        let cliff: u64 = 0;
        let duration: u64 = 0;

        let source = id.account::<Test>();
        Balances::make_free_balance_be(&source, 1_000_000);

        Envelopes::<Test>::insert(
            id,
            EnvelopeConfig::<u64, u64> {
                total_cap,
                upfront_rate,
                cliff,
                vesting_duration: duration,
            },
        );
        EnvelopeDistributed::<Test>::insert(id, 0u64);

        // First allocation within cap
        assert_ok!(TokenAllocation::add_allocation(
            mock::RuntimeOrigin::root(),
            id,
            1u128,
            100u64
        ));
        // Second would exceed cap
        assert_noop!(
            TokenAllocation::add_allocation(mock::RuntimeOrigin::root(), id, 2u128, 60u64),
            Error::<Test>::EnvelopeCapExceeded
        );
    });
}

#[test]
fn cannot_add_duplicate_allocation() {
    new_test_ext().execute_with(|| {
        let id = EnvelopeId::Private1;
        Envelopes::<Test>::insert(
            id,
            EnvelopeConfig::<u64, u64> {
                total_cap: 1_000,
                upfront_rate: Percent::from_percent(0),
                cliff: 0,
                vesting_duration: 0,
            },
        );
        EnvelopeDistributed::<Test>::insert(id, 0u64);
        let source = id.account::<Test>();
        Balances::make_free_balance_be(&source, 1_000_000);

        assert_ok!(TokenAllocation::add_allocation(
            mock::RuntimeOrigin::root(),
            id,
            1u128,
            100u64
        ));
        assert_noop!(
            TokenAllocation::add_allocation(mock::RuntimeOrigin::root(), id, 1u128, 50u64),
            Error::<Test>::AllocationExists
        );
    });
}

#[test]
fn add_allocation_fails_if_source_insufficient() {
    new_test_ext().execute_with(|| {
        let id = EnvelopeId::Private2;
        Envelopes::<Test>::insert(
            id,
            EnvelopeConfig::<u64, u64> {
                total_cap: 1_000_000,
                upfront_rate: Percent::from_percent(50),
                cliff: 0,
                vesting_duration: 100,
            },
        );
        EnvelopeDistributed::<Test>::insert(id, 0u64);
        let source = id.account::<Test>();
        // Provide less than required (required == total), set only 99
        Balances::make_free_balance_be(&source, 99);

        assert_noop!(
            TokenAllocation::add_allocation(mock::RuntimeOrigin::root(), id, 1u128, 100u64),
            Error::<Test>::EnvelopeCapExceeded
        );
    });
}

#[test]
fn claim_before_cliff_is_zero() {
    new_test_ext().execute_with(|| {
        let id = EnvelopeId::ICO1;
        let total_cap = 1_000_000u64;
        let upfront_rate: Percent = Percent::from_percent(0);
        let cliff = 100u64;
        let duration = 1_000u64;
        let who = 1u128;
        let total = 1_000u64;
        let source = id.account::<Test>();
        Balances::make_free_balance_be(&source, total_cap);

        Envelopes::<Test>::insert(
            id,
            EnvelopeConfig {
                total_cap,
                upfront_rate,
                cliff,
                vesting_duration: duration,
            },
        );
        EnvelopeDistributed::<Test>::insert(id, 0u64);
        assert_ok!(TokenAllocation::add_allocation(
            mock::RuntimeOrigin::root(),
            id,
            who,
            total
        ));

        set_block(50);
        assert_noop!(
            TokenAllocation::claim(mock::RuntimeOrigin::signed(who), id),
            Error::<Test>::NothingToClaim
        );
    });
}

#[test]
fn linear_vesting_claims_over_time() {
    new_test_ext().execute_with(|| {
        let id = EnvelopeId::Seed;
        let total_cap = 1_000_000u64;
        let upfront_rate: Percent = Percent::from_percent(0);
        let cliff = 10u64;
        let duration = 100u64;
        let who = 1u128;
        let total = 1_000u64;
        let source = id.account::<Test>();
        Balances::make_free_balance_be(&source, total_cap);

        Envelopes::<Test>::insert(id, EnvelopeConfig { total_cap, upfront_rate, cliff, vesting_duration: duration });
        EnvelopeDistributed::<Test>::insert(id, 0u64);
        assert_ok!(TokenAllocation::add_allocation(mock::RuntimeOrigin::root(), id, who, total));

        // At cliff exactly, claimable == 0 because now <= cliff returns 0
        set_block(cliff);
        assert_noop!(TokenAllocation::claim(mock::RuntimeOrigin::signed(who), id), Error::<Test>::NothingToClaim);

        // After some time: 50% of duration
        set_block(cliff + duration / 2);
        let before = Balances::free_balance(who);
        assert_ok!(TokenAllocation::claim(mock::RuntimeOrigin::signed(who), id));
        let after = Balances::free_balance(who);
        // vested_total == total (no upfront). 50% of 1000 = 500
        assert_eq!(after - before, 500);

        // After end, remaining all can be claimed
        set_block(cliff + duration + 1);
        let before2 = Balances::free_balance(who);
        assert_ok!(TokenAllocation::claim(mock::RuntimeOrigin::signed(who), id));
        let after2 = Balances::free_balance(who);
        // remaining 500
        assert_eq!(after2 - before2, 500);

        // No more to claim
        assert_noop!(TokenAllocation::claim(mock::RuntimeOrigin::signed(who), id), Error::<Test>::NothingToClaim);

        // Event emitted on claims
        let events: Vec<TestEvent> = System::events().into_iter().map(|r| r.event).collect();
        assert!(events.iter().any(|e| matches!(e,
            TestEvent::TokenAllocation(PalletEvent::VestedReleased(eid, account, amount)) if eid == &id && account == &who && amount == &500u64
        )));
    });
}

#[test]
fn upfront_zero_skips_transfer_and_event() {
    new_test_ext().execute_with(|| {
        let id = EnvelopeId::ICO2;
        let total_cap = 1_000_000u64;
        let upfront_rate: Percent = Percent::from_percent(0); // 0%
        let cliff = 0u64;
        let duration = 100u64;
        let who = 2u128;
        let total = 1_000u64;
        let source = id.account::<Test>();
        Balances::make_free_balance_be(&source, total_cap);

        Envelopes::<Test>::insert(
            id,
            EnvelopeConfig {
                total_cap,
                upfront_rate,
                cliff,
                vesting_duration: duration,
            },
        );
        EnvelopeDistributed::<Test>::insert(id, 0u64);

        let before_events = System::events().len();
        let before_balance = Balances::free_balance(who);
        assert_ok!(TokenAllocation::add_allocation(
            mock::RuntimeOrigin::root(),
            id,
            who,
            total
        ));
        let after_balance = Balances::free_balance(who);

        assert_eq!(after_balance, before_balance); // no upfront received
        let events = &System::events()[before_events..];
        // Only AllocationAdded should be present for this call (no UpfrontPaid)
        assert!(events.iter().any(|rec| matches!(
            rec.event,
            TestEvent::TokenAllocation(PalletEvent::AllocationAdded(..))
        )));
        assert!(!events.iter().any(|rec| matches!(
            rec.event,
            TestEvent::TokenAllocation(PalletEvent::UpfrontPaid(..))
        )));
    });
}

#[test]
fn claim_fails_when_allocation_missing_or_envelope_unknown() {
    new_test_ext().execute_with(|| {
        let id = EnvelopeId::SerieA;
        // Claim fails if envelope unknown
        assert_noop!(
            TokenAllocation::claim(mock::RuntimeOrigin::signed(1u128), id),
            Error::<Test>::EnvelopeUnknown
        );

        // Claim fails if allocation missing
        Envelopes::<Test>::insert(
            id,
            EnvelopeConfig::<u64, u64> {
                total_cap: 1000,
                upfront_rate: Percent::from_percent(0),
                cliff: 0,
                vesting_duration: 0,
            },
        );
        EnvelopeDistributed::<Test>::insert(id, 0u64);
        assert_noop!(
            TokenAllocation::claim(mock::RuntimeOrigin::signed(1u128), id),
            Error::<Test>::EnvelopeUnknown
        );
    });
}

#[test]
fn claim_all_at_exact_end_block() {
    new_test_ext().execute_with(|| {
        // At now == cliff + duration, full remaining vested should be claimable
        let id = EnvelopeId::Founders;
        let total_cap = 1_000_000u64;
        let upfront_rate = Percent::from_percent(0);
        let cliff = 10u64;
        let duration = 90u64;
        let who = 1u128;
        let total = 1_000u64;
        let source = id.account::<Test>();
        Balances::make_free_balance_be(&source, total_cap);
        Envelopes::<Test>::insert(
            id,
            EnvelopeConfig {
                total_cap,
                upfront_rate,
                cliff,
                vesting_duration: duration,
            },
        );
        EnvelopeDistributed::<Test>::insert(id, 0u64);
        assert_ok!(TokenAllocation::add_allocation(
            mock::RuntimeOrigin::root(),
            id,
            who,
            total
        ));

        set_block(cliff + duration);
        let before = Balances::free_balance(who);
        assert_ok!(TokenAllocation::claim(mock::RuntimeOrigin::signed(who), id));
        let after = Balances::free_balance(who);
        assert_eq!(after - before, total);
    });
}

#[test]
fn multiple_claims_track_released_correctly() {
    new_test_ext().execute_with(|| {
        // Claim in two steps: mid-vesting and at end; released should accumulate
        let id = EnvelopeId::Seed;
        let total_cap = 1_000_000u64;
        let upfront_rate = Percent::from_percent(0);
        let cliff = 10u64;
        let duration = 100u64;
        let who = 1u128;
        let total = 1_000u64;
        let source = id.account::<Test>();
        Balances::make_free_balance_be(&source, total_cap);
        Envelopes::<Test>::insert(
            id,
            EnvelopeConfig {
                total_cap,
                upfront_rate,
                cliff,
                vesting_duration: duration,
            },
        );
        EnvelopeDistributed::<Test>::insert(id, 0u64);
        assert_ok!(TokenAllocation::add_allocation(
            mock::RuntimeOrigin::root(),
            id,
            who,
            total
        ));

        set_block(cliff + duration / 4);
        let b0 = Balances::free_balance(who);
        assert_ok!(TokenAllocation::claim(mock::RuntimeOrigin::signed(who), id));
        let c1 = Balances::free_balance(who) - b0; // 25% of 1000 = 250
        assert_eq!(c1, 250);

        set_block(cliff + duration);
        let b1 = Balances::free_balance(who);
        assert_ok!(TokenAllocation::claim(mock::RuntimeOrigin::signed(who), id));
        let c2 = Balances::free_balance(who) - b1; // remaining 750
        assert_eq!(c2, 750);

        // No more to claim
        assert_noop!(
            TokenAllocation::claim(mock::RuntimeOrigin::signed(who), id),
            Error::<Test>::NothingToClaim
        );
    });
}

#[test]
fn upfront_rounds_down_and_linear_floor() {
    new_test_ext().execute_with(|| {
        // Percent 1% with total 99 => upfront 0; vesting floors over time
        let id = EnvelopeId::ICO1;
        let total_cap = 10_000u64;
        let upfront_rate = Percent::from_percent(1); // 1%
        let cliff = 0u64;
        let duration = 10u64;
        let who = 3u128;
        let total = 99u64; // upfront should floor to 0
        let source = id.account::<Test>();
        Balances::make_free_balance_be(&source, total_cap);
        Envelopes::<Test>::insert(
            id,
            EnvelopeConfig {
                total_cap,
                upfront_rate,
                cliff,
                vesting_duration: duration,
            },
        );
        EnvelopeDistributed::<Test>::insert(id, 0u64);
        assert_ok!(TokenAllocation::add_allocation(
            mock::RuntimeOrigin::root(),
            id,
            who,
            total
        ));

        // No upfront event expected
        let before = Balances::free_balance(who);
        set_block(duration / 2); // at half duration, vested = floor(99*5/10)=49
        assert_ok!(TokenAllocation::claim(mock::RuntimeOrigin::signed(who), id));
        let mid = Balances::free_balance(who);
        assert_eq!(mid - before, 49);

        set_block(duration);
        assert_ok!(TokenAllocation::claim(mock::RuntimeOrigin::signed(who), id));
        let after = Balances::free_balance(who);
        assert_eq!(after - mid, 50); // remaining
    });
}

#[test]
fn vesting_duration_zero_unlocks_after_cliff() {
    new_test_ext().execute_with(|| {
        // If duration == 0, all vested becomes claimable right after cliff
        let id = EnvelopeId::Private2;
        let total_cap = 1_000_000u64;
        let upfront_rate = Percent::from_percent(0);
        let cliff = 5u64;
        let duration = 0u64;
        let who = 4u128;
        let total = 1_000u64;
        let source = id.account::<Test>();
        Balances::make_free_balance_be(&source, total_cap);
        Envelopes::<Test>::insert(
            id,
            EnvelopeConfig {
                total_cap,
                upfront_rate,
                cliff,
                vesting_duration: duration,
            },
        );
        EnvelopeDistributed::<Test>::insert(id, 0u64);
        assert_ok!(TokenAllocation::add_allocation(
            mock::RuntimeOrigin::root(),
            id,
            who,
            total
        ));

        set_block(cliff);
        assert_noop!(
            TokenAllocation::claim(mock::RuntimeOrigin::signed(who), id),
            Error::<Test>::NothingToClaim
        );

        set_block(cliff + 1);
        let b0 = Balances::free_balance(who);
        assert_ok!(TokenAllocation::claim(mock::RuntimeOrigin::signed(who), id));
        let after = Balances::free_balance(who);
        assert_eq!(after - b0, total);
    });
}

#[test]
fn funding_must_cover_upfront_and_vested_total() {
    new_test_ext().execute_with(|| {
        // If source covers only upfront but not vested_total, allocation must fail
        let id = EnvelopeId::KoL;
        let total_cap = 1_000_000u64;
        let upfront_rate = Percent::from_percent(30);
        let cliff = 0u64;
        let duration = 100u64;
        let who = 5u128;
        let total = 1_000u64; // upfront 300, vested 700
        let source = id.account::<Test>();
        Balances::make_free_balance_be(&source, 299); // less than required 1000
        Envelopes::<Test>::insert(
            id,
            EnvelopeConfig {
                total_cap,
                upfront_rate,
                cliff,
                vesting_duration: duration,
            },
        );
        EnvelopeDistributed::<Test>::insert(id, 0u64);

        assert_noop!(
            TokenAllocation::add_allocation(mock::RuntimeOrigin::root(), id, who, total),
            Error::<Test>::EnvelopeCapExceeded
        );
    });
}

#[test]
fn exact_fill_of_cap_with_multiple_beneficiaries() {
    new_test_ext().execute_with(|| {
        // Two allocations exactly fill cap; third should fail
        let id = EnvelopeId::ICO2;
        let total_cap = 1_000u64;
        let upfront_rate = Percent::from_percent(0);
        let cliff = 0u64;
        let duration = 10u64;
        let source = id.account::<Test>();
        Balances::make_free_balance_be(&source, 10_000);
        Envelopes::<Test>::insert(
            id,
            EnvelopeConfig {
                total_cap,
                upfront_rate,
                cliff,
                vesting_duration: duration,
            },
        );
        EnvelopeDistributed::<Test>::insert(id, 0u64);

        assert_ok!(TokenAllocation::add_allocation(
            mock::RuntimeOrigin::root(),
            id,
            10u128,
            400u64
        ));
        assert_ok!(TokenAllocation::add_allocation(
            mock::RuntimeOrigin::root(),
            id,
            11u128,
            600u64
        ));
        assert_noop!(
            TokenAllocation::add_allocation(mock::RuntimeOrigin::root(), id, 12u128, 1u64),
            Error::<Test>::EnvelopeCapExceeded
        );
    });
}

#[test]
fn repeated_claim_when_nothing_new_vested_is_noop() {
    new_test_ext().execute_with(|| {
        // Claim twice in same block: second yields NothingToClaim
        let id = EnvelopeId::Seed;
        let total_cap = 1_000_000u64;
        let upfront_rate = Percent::from_percent(0);
        let cliff = 0u64;
        let duration = 100u64;
        let who = 6u128;
        let total = 1_000u64;
        let source = id.account::<Test>();
        Balances::make_free_balance_be(&source, total_cap);
        Envelopes::<Test>::insert(
            id,
            EnvelopeConfig {
                total_cap,
                upfront_rate,
                cliff,
                vesting_duration: duration,
            },
        );
        EnvelopeDistributed::<Test>::insert(id, 0u64);
        assert_ok!(TokenAllocation::add_allocation(
            mock::RuntimeOrigin::root(),
            id,
            who,
            total
        ));

        set_block(duration / 2);
        assert_ok!(TokenAllocation::claim(mock::RuntimeOrigin::signed(who), id));
        assert_noop!(
            TokenAllocation::claim(mock::RuntimeOrigin::signed(who), id),
            Error::<Test>::NothingToClaim
        );
    });
}

#[test]
fn large_values_no_overflow_in_mul_div() {
    new_test_ext().execute_with(|| {
        // Exercise mul_div with large values via long vesting and large totals
        let id = EnvelopeId::Founders;
        let total_cap: u64 = u64::MAX / 4; // big but safe
        let upfront_rate = Percent::from_percent(0);
        let cliff = 0u64;
        let duration: u64 = u32::MAX as u64; // large duration
        let who = 7u128;
        let total: u64 = (u64::MAX / 8) & !1; // even
        let source = id.account::<Test>();
        Balances::make_free_balance_be(&source, total_cap);
        Envelopes::<Test>::insert(
            id,
            EnvelopeConfig {
                total_cap,
                upfront_rate,
                cliff,
                vesting_duration: duration,
            },
        );
        EnvelopeDistributed::<Test>::insert(id, 0u64);
        assert_ok!(TokenAllocation::add_allocation(
            mock::RuntimeOrigin::root(),
            id,
            who,
            total
        ));

        set_block(duration / 2);
        let before = Balances::free_balance(who);
        assert_ok!(TokenAllocation::claim(mock::RuntimeOrigin::signed(who), id));
        let after = Balances::free_balance(who);
        // Roughly half should be claimable (floor); ensure non-zero and <= total/2
        assert!(after > before);
        assert!(after - before <= total / 2);
    });
}
