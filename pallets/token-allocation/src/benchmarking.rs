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

use super::*;
use crate::Pallet as TokenAllocPallet;
use frame_benchmarking::v2::*;
use frame_system::RawOrigin;
use sp_runtime::traits::Zero;

fn setup_envelope<T: Config>(
    id: EnvelopeId,
    total_cap: BalanceOf<T>,
    upfront_rate: Percent,
    cliff: BlockNumberFor<T>,
    vesting_duration: BlockNumberFor<T>,
    unique_beneficiary: Option<T::AccountId>,
) {
    let cfg = EnvelopeConfig::<BalanceOf<T>, BlockNumberFor<T>, T::AccountId> {
        total_cap,
        upfront_rate,
        cliff,
        vesting_duration,
        unique_beneficiary: unique_beneficiary.clone(),
    };

    Envelopes::<T>::insert(id, cfg.clone());
    EnvelopeDistributed::<T>::insert(id, BalanceOf::<T>::zero());

    let envelope_acc = id.account::<T>();
    if !total_cap.is_zero() {
        T::Currency::mint_into(&envelope_acc, total_cap).expect("mint in benchmark should work");
    }

    if NextPayoutAt::<T>::get().is_zero() {
        NextPayoutAt::<T>::put(T::EpochDuration::get());
        EpochIndex::<T>::put(0u64);
        PayoutCursor::<T>::kill();
    }
}

fn force_allocation<T: Config>(
    id: EnvelopeId,
    who: &T::AccountId,
    total: BalanceOf<T>,
    start: Option<BlockNumberFor<T>>,
    emit_events: bool,
) {
    let cfg = Envelopes::<T>::get(id).expect("env set");
    TokenAllocPallet::<T>::do_add_allocation(id, who, total, start, &cfg, emit_events)
        .expect("allocation in benchmark cannot fail");
}

/// Helper to make an allocation that will *definitely* pay something at `now`
/// and be fully vested after that payout (worst-case route in payout_allocation):
///
/// We do that by:
/// - upfront_rate = 0 (so everything is vested_total)
/// - cliff = 0
/// - vesting_duration = 1
/// - start = Some(0)
///
/// Then at `now = 1`, `claimable_amount` == full vested_total.
/// So payout_allocation() will:
/// - call release(...)
/// - see allocation fully vested
/// - remove it from storage
/// - dec_providers
/// - emit VestedReleased.
///
/// This is the heavy path we want to benchmark.
fn setup_fully_vestable_allocation<T: Config>(
    alloc_idx_seed: u32,
    envelope_id: EnvelopeId,
    per_alloc_amount: BalanceOf<T>,
) -> T::AccountId {
    let who: T::AccountId = account("recipient", 0, alloc_idx_seed);

    force_allocation::<T>(
        envelope_id,
        &who,
        per_alloc_amount,
        Some(Zero::zero()), // start at block 0
        false,              // don't emit events in bench setup
    );

    who
}

#[benchmarks(
    where
        BalanceOf<T>: From<u64> + Into<u128>,
        BlockNumberFor<T>: From<u32> + Into<u128>,
)]
mod benches {
    use super::*;

    /// Benchmark for `add_allocation` extrinsic.
    ///
    /// Worst case:
    /// - caller is AdminOrigin -> we simulate with RawOrigin::Root and assume runtime maps it.
    /// - `AllocationsOf` for `who` is empty (first push so we hit try_push)
    /// - upfront > 0 so it has to release.
    #[benchmark]
    fn add_allocation() -> Result<(), BenchmarkError> {
        let who: T::AccountId = account("recipient", 0, 0u32);
        let ed = T::Currency::minimum_balance();

        // basic envelope config:
        // total_cap = 1_000_000
        // upfront_rate = 20%
        // cliff = 10
        // vesting_duration = 100
        // unique_beneficiary = None (so extrinsic is allowed)
        setup_envelope::<T>(
            EnvelopeId::Founders,
            ed.saturating_mul(100.into()),
            Percent::from_percent(20),
            10u32.into(),
            100u32.into(),
            None,
        );

        #[extrinsic_call]
        _(
            RawOrigin::Root,
            EnvelopeId::Founders,
            who.clone(),
            ed.saturating_mul(50.into()),
            Some(10u32.into()),
        );

        Ok(())
    }

    #[benchmark]
    fn on_initialize_noop() {
        // Manually ensure NextPayoutAt is strictly GREATER than `now`,
        // so that Pallet::<T>::on_initialize(now) returns via the noop branch.
        let now: BlockNumberFor<T> = 1u32.into();
        let future_epoch: BlockNumberFor<T> = 10u32.into();
        NextPayoutAt::<T>::put(future_epoch);
        EpochIndex::<T>::put(0u64);
        PayoutCursor::<T>::kill();

        #[block]
        {
            Pallet::<T>::on_initialize(now);
        }
    }

    /// Benchmark for on_initialize() when we are IN an epoch (now >= NextPayoutAt),
    /// and we hit MaxPayoutsPerBlock so we EXIT EARLY with a cursor (partial scan / pagination).
    ///
    /// We want worst-case per-block cost:
    /// - Have (MaxPayoutsPerBlock + 1) allocations that will all fully vest at `now`,
    ///   so each payout_allocation() triggers the heavy path:
    ///     - release()
    ///     - remove allocation
    ///     - dec_providers()
    ///     - emit VestedReleased(...)
    ///
    /// - After processing `limit = MaxPayoutsPerBlock`, the hook must:
    ///     - leave some allocations unprocessed
    ///     - set PayoutCursor(Some(next_id))
    ///     - emit EpochPayout { cursor: Some(..) }
    ///     - NOT bump EpochIndex
    ///     - NOT bump NextPayoutAt
    #[benchmark]
    fn on_initialize_partial() {
        let ed = T::Currency::minimum_balance();

        // 1. Prepare an envelope with:
        //    upfront_rate = 0
        //    cliff = 0
        //    vesting_duration = 1
        // So at now = 1, 100% is claimable.
        setup_envelope::<T>(
            EnvelopeId::Airdrop,
            ed.saturating_mul(10_000_000u64.into()),
            Percent::from_percent(0),
            0u32.into(),
            1u32.into(), // super short vesting
            None,
        );

        // Initialize epoch bookkeeping so that now >= NextPayoutAt triggers payout logic.
        // We'll run on_initialize at block `now = 1`.
        let now: BlockNumberFor<T> = 1u32.into();
        NextPayoutAt::<T>::put(now); // force payout path
        EpochIndex::<T>::put(0u64);
        PayoutCursor::<T>::kill();
        NextAllocationId::<T>::put(0);

        // 2. Create N = MaxPayoutsPerBlock + 1 allocations that will fully vest at `now`.
        let limit = T::MaxPayoutsPerBlock::get();
        let total_allocs: u32 = limit.saturating_add(1);
        let per_alloc_amount: BalanceOf<T> = ed.saturating_mul(1_000u64.into());

        for i in 0..total_allocs {
            let _benef = setup_fully_vestable_allocation::<T>(
                i, // unique-ish seed for account
                EnvelopeId::Airdrop,
                per_alloc_amount, // same size for all
            );
        }

        // Sanity: we should have exactly total_allocs allocations in storage,
        // with ids [0..total_allocs).
        assert_eq!(NextAllocationId::<T>::get(), total_allocs);

        #[block]
        {
            Pallet::<T>::on_initialize(now);
        }
    }

    /// Benchmark for on_initialize() when we are IN an epoch (now >= NextPayoutAt),
    /// and we FINISH the epoch in this block (no pagination needed).
    ///
    /// We want to exercise:
    /// - process the remaining allocations (strictly fewer than MaxPayoutsPerBlock),
    ///   each using the heavy payout_allocation path (full vest, remove allocation, etc.)
    /// - clear PayoutCursor
    /// - bump EpochIndex
    /// - set NextPayoutAt = now + EpochDuration
    /// - emit EpochPayout { cursor: None }
    #[benchmark]
    fn on_initialize_epoch_finished(x: Linear<1, { T::MaxPayoutsPerBlock::get() - 1 }>) {
        let ed = T::Currency::minimum_balance();

        // Same "fully vest at now=1" envelope config.
        setup_envelope::<T>(
            EnvelopeId::Seed,
            ed.saturating_mul(10_000_000u64.into()),
            Percent::from_percent(0),
            0u32.into(),
            1u32.into(), // vest fully by block 1
            None,
        );

        let now: BlockNumberFor<T> = 1u32.into();
        NextPayoutAt::<T>::put(now);
        EpochIndex::<T>::put(7u64);
        PayoutCursor::<T>::kill();
        NextAllocationId::<T>::put(0);

        // create exactly `payouts` allocations fully vestable at `now`
        let per_alloc_amount: BalanceOf<T> = ed.saturating_mul(5_000u64.into());
        for i in 0..x {
            let _ = setup_fully_vestable_allocation::<T>(
                20_000 + i,
                EnvelopeId::Seed,
                per_alloc_amount,
            );
        }

        // Sanity: after this loop, we should have NextAllocationId == payouts.
        assert_eq!(NextAllocationId::<T>::get(), x);

        #[block]
        {
            Pallet::<T>::on_initialize(now);
        }
    }

    impl_benchmark_test_suite!(
        TokenAllocPallet,
        crate::mock::new_test_ext(
            // empty envelopes/allocations at genesis; we set them manually in each bench
            sp_runtime::Vec::new(),
            sp_runtime::Vec::new()
        ),
        crate::mock::Test,
    );
}
