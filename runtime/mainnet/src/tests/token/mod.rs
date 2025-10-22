#![cfg(test)]

use crate::{tests::new_test_ext, *};
use frame_support::{
    assert_ok,
    traits::{Hooks, fungible::InspectHold},
};
use frame_system::pallet_prelude::BlockNumberFor;
use pallet_token_allocation::{AllocationsOf, HoldReason};
use shared_runtime::currency::AFT;
use sp_keyring::Sr25519Keyring;

// Helper: read “held” balance on treasury under this pallet’s hold reason
fn held_on_treasury() -> Balance {
    let reason = RuntimeHoldReason::TokenAllocation(HoldReason::TokenAllocation);
    pallet_balances::Pallet::<Runtime>::balance_on_hold(&reason, &Treasury::account_id())
}

fn advance_to(n: BlockNumberFor<Runtime>) {
    frame_system::Pallet::<Runtime>::set_block_number(n);
    let _ = pallet_token_allocation::Pallet::<Runtime>::on_initialize(n);
}

#[test]
fn total_issuance_is_one_billion_at_genesis() {
    new_test_ext().execute_with(|| {
        let total_issuance: Balance = pallet_balances::Pallet::<Runtime>::total_issuance();

        let expected: Balance = 1_000_000_000 * AFT;
        assert_eq!(
            total_issuance as u128, expected,
            "Unexpected total issuance at genesis"
        );

        advance_to(10_000_000);

        let total_issuance_future: Balance = pallet_balances::Pallet::<Runtime>::total_issuance();
        assert_eq!(
            total_issuance_future as u128, expected,
            "Unexpected total issuance in the future."
        );
    })
}

#[test]
fn foundation_receives_correct_upfront_at_genesis() {
    new_test_ext().execute_with(|| {
        // Expected upfront per enveloppes:
        // - ResearchDevelopment: 125M * 20% = 25M
        // - Reserve:            20M  *100% = 20M
        // Total upfront = 45M
        let expected_upfront = (25_000_000u128 + 20_000_000u128) * AFT;

        // Expected held: (260 + 100 + 125 + 20) - 45 = 460M
        let expected_held = 460_000_000u128 * AFT;

        let free = pallet_balances::Pallet::<Runtime>::free_balance(Treasury::account_id());
        assert_eq!(
            free, expected_upfront,
            "Treasury free balance (upfront) at genesis is wrong"
        );

        let held = held_on_treasury() as u128;
        assert_eq!(
            held, expected_held,
            "Treasury held balance at genesis is wrong"
        );
    });
}

/// End-to-end integrity test:
/// - add a runtime allocation (no unique beneficiary) to a user,
/// - upfront is paid immediately,
/// - nothing releases before the cliff,
/// - release starts only at the next epoch after the cliff,
/// - allocation is removed when fully vested.
#[test]
fn e2e_add_beneficiary_and_distribute_until_completion() {
    // GIVEN the full runtime genesis (tokenomics already applied by new_test_ext()).
    new_test_ext().execute_with(|| {
        // Constants from the runtime tokenomics for `Private2`
        // Private2: upfront 5%, cliff 3 * MONTHS, vesting 36 * MONTHS
        let epoch: BlockNumber = <Runtime as pallet_token_allocation::Config>::EpochDuration::get();

        // Choose a test account for the runtime allocation (must not be the foundation account).
        let alice: AccountId = Sr25519Keyring::Alice.to_account_id();

        // Pick an allocation size that is small but visible.
        let alloc_total: Balance = 1_000_000 * AFT;

        // WHEN we add a runtime allocation on an envelope that allows it (no unique beneficiary).
        assert_ok!(pallet_token_allocation::Pallet::<Runtime>::add_allocation(
            RuntimeOrigin::root(),
            pallet_token_allocation::EnvelopeId::Private2,
            alice.clone(),
            alloc_total,
            Some(0), // start=0 → effective_start = max(0, cliff) = cliff
        ));

        // THEN upfront (5%) is paid immediately and the rest is held.
        let upfront_rate = sp_runtime::Percent::from_percent(5);
        let upfront = upfront_rate.mul_floor(alloc_total);
        let free_0 = pallet_balances::Pallet::<Runtime>::free_balance(&alice);
        assert_eq!(
            free_0, upfront,
            "upfront must be credited at allocation time"
        );

        // Allocation exists in storage with proper fields.
        let mut alloc = AllocationsOf::<Runtime>::get(alice.clone());
        assert_eq!(alloc.first().unwrap().total, alloc_total);
        assert_eq!(alloc.first().unwrap().upfront, upfront);
        assert_eq!(alloc.first().unwrap().vested_total, alloc_total - upfront);
        assert_eq!(alloc.first().unwrap().released, 0);

        // --- BEFORE CLIFF: no release, even if epochs tick before the cliff.
        // Move to just before the cliff.
        let before_cliff = 3 * MONTHS - 1;
        advance_to(before_cliff);

        alloc = AllocationsOf::<Runtime>::get(alice.clone());
        let free_before_cliff = pallet_balances::Pallet::<Runtime>::free_balance(&alice);
        assert_eq!(
            alloc.first().unwrap().released,
            0,
            "no vested release before the cliff (even with epochs)"
        );
        assert_eq!(
            free_before_cliff, upfront,
            "free balance must stay at upfront before the cliff"
        );

        // --- AT CLIFF (exact): still nothing until the next epoch tick after the cliff.
        let at_cliff = 3 * MONTHS;
        advance_to(at_cliff);
        alloc = AllocationsOf::<Runtime>::get(alice.clone());
        let free_at_cliff = pallet_balances::Pallet::<Runtime>::free_balance(&alice);
        assert_eq!(
            alloc.first().unwrap().released,
            0,
            "no release exactly at cliff unless epoch fires here"
        );
        assert_eq!(free_at_cliff, upfront);

        // --- FIRST RELEASE: at the next epoch after the cliff.
        let first_release_block = at_cliff + epoch;
        advance_to(first_release_block);

        alloc = AllocationsOf::<Runtime>::get(alice.clone());
        let free_first_release = pallet_balances::Pallet::<Runtime>::free_balance(&alice);
        assert!(
            alloc.first().unwrap().released > 0,
            "first release must happen on the first epoch after the cliff"
        );
        assert!(
            free_first_release > upfront,
            "free balance should increase after first epoch post-cliff"
        );

        // --- CONTINUE UNTIL COMPLETION:
        // Jump far beyond vesting end to ensure the allocation finishes and is pruned.
        let vest_duration = 36 * MONTHS;
        let after_vest = at_cliff + vest_duration + epoch * 2;
        advance_to(after_vest);

        // Allocation should be removed from storage.
        let finished = AllocationsOf::<Runtime>::get(alice.clone());
        assert!(
            finished.is_empty(),
            "allocation must be pruned once fully released"
        );

        // And free balance must be initial upfront + full vested_total.
        let free_end = pallet_balances::Pallet::<Runtime>::free_balance(&alice);
        assert!(
            free_end == alloc_total, // == alloc_total in the ideal case
            "by the end, beneficiary should have received the whole allocation"
        );
    });
}
