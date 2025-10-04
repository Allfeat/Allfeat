# pallet-token-allocation

FRAME pallet for managing token allocations grouped by logical envelopes, with upfront payment, cliff, and linear vesting over blocks.

## Overview

This pallet lets an admin define "envelopes" (budget buckets) with:
- total_cap: maximum total tokens that can ever be allocated from the envelope
- upfront_rate: Percent (0..=100%) paid immediately upon allocation
- cliff: block number until which nothing vests
- vesting_duration: number of blocks linearly vesting after the cliff

Each allocation to a beneficiary is split into:
- upfront: paid instantly (mul_floor of total by upfront_rate)
- vested_total: linearly unlockable after cliff over vesting_duration

Claiming transfers the vested portion that is currently available but not yet released.

## Concepts

- EnvelopeId: fixed set of logical envelopes (e.g., Founders, Seed).
- Sub-account per envelope: derived from PalletId and EnvelopeId; source of transfers.
- total_cap: hard budget limit per envelope (sum of all allocations cannot exceed it).
- upfront_rate: Percent; use Percent::from_percent(n).
- Cliff and vesting on block numbers: vesting logic is deterministic and block-based.

## Storage

- Envelopes: EnvelopeId -> EnvelopeConfig { total_cap, upfront_rate, cliff, vesting_duration }
- EnvelopeDistributed: EnvelopeId -> Balance (cumulative allocated)
- Allocations: (EnvelopeId, AccountId) -> Allocation { total, upfront, vested_total, released }

## Events

- AllocationAdded(EnvelopeId, AccountId, Balance)
- UpfrontPaid(EnvelopeId, AccountId, Balance)
- VestedReleased(EnvelopeId, AccountId, Balance)

## Errors

- EnvelopeUnknown: unknown envelope
- AllocationExists: cannot create a second allocation for same (envelope, account)
- EnvelopeCapExceeded: would exceed total_cap or insufficient funded balance on envelope account
- NothingToClaim: no vested amount available to claim
- ArithmeticOverflow: overflow in intermediate math

## Extrinsics

- add_allocation(origin: AdminOrigin, id: EnvelopeId, who: AccountId, total: Balance)
  - Validates envelope exists, cap not exceeded, and source account has enough free balance for upfront + vested_total.
  - Pays upfront immediately if > 0.

- claim(origin: Signed, id: EnvelopeId)
  - Computes claimable vested amount based on current block number.
  - Transfers claimable and updates released.

## Vesting formula

At block `now`:
- If now <= cliff: 0 is claimable
- Else let elapsed = now - cliff
  - If elapsed >= vesting_duration: claimable = vested_total - released
  - Else: claimable = floor(vested_total * elapsed / vesting_duration) - released

## Genesis configuration

You can pre-load envelopes via GenesisConfig, providing pairs of (EnvelopeId, EnvelopeConfig<Balance, u64>);
`cliff` and `vesting_duration` are given as u64 and converted to the chain's BlockNumber at build time.

Example snippet (pseudo-code):

```
GenesisConfig {
  envelopes: vec![
    (EnvelopeId::Seed, EnvelopeConfig {
      total_cap: 1_000_000 * UNIT,
      upfront_rate: Percent::from_percent(0),
      cliff: 10,                  // blocks
      vesting_duration: 100,      // blocks
    }),
  ],
}
```

## Rounding and precision

- upfront uses Percent.mul_floor(total) to avoid over-crediting.
- Linear vesting uses integer arithmetic with widening to u128 and checked division.

## Security and invariants

- Sum of all allocations for an envelope cannot exceed total_cap.
- Source sub-account must be sufficiently funded for upfront + vested_total at allocation time.
- No panics in runtime; all errors use DispatchError.

## Testing

- Unit tests cover upfront payment, cap enforcement, duplicate allocations, claiming before/after cliff, and end-of-vesting behavior.
- Tests advance block number via `System::set_block_number`.

## Integration notes

- Ensure the pallet's PalletId has a funded sub-account for each used EnvelopeId.
- Consider governance-controlled calls to top up sub-accounts; total_cap still enforces the logical budget.
- If finer precision than Percent is needed for upfront, replace Percent with Permill/Perbill accordingly.
