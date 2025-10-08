# Token Allocation Pallet

A Substrate pallet for managing Allfeat token allocation and vesting across different investment envelopes.

## Overview

This pallet provides a secure and flexible system for token distribution with vesting mechanisms. It supports multiple envelope types (Allfeat specific ones) each with their own vesting parameters and automatically generated accounts.

## Key Features

- **Envelope-based allocation**: Separate pools for different investment rounds
- **Automatic account generation**: Deterministic envelope accounts without private keys
- **Flexible vesting**: Configurable cliff periods, vesting duration, and immediate unlock percentages
- **Manual claiming**: Beneficiaries claim their vested tokens when ready
- **Genesis configuration**: Pre-TGE and post-TGE contributor setup
- **Secure design**: No withdrawal functions, tokens can only be claimed through vesting

## Architecture

### Vesting Configuration

```rust
pub struct EnvelopeConfig<BlockNumber> {
    pub immediate_unlock_percentage: Percent,  // % unlocked immediately
    pub cliff_duration: BlockNumber,           // Blocks before vesting starts
    pub vesting_duration: BlockNumber,         // Total vesting period in blocks
}
```

### Allocation Status

```rust
pub enum AllocationStatus<BlockNumber> {
    ActiveSinceGenesis,    // Activated at block 0 (pre-TGE contributors)
    ActivatedAt(BlockNumber), // Activated at specific block (post-TGE contributors)
    Completed,             // Fully claimed
    Revoked,              // Revoked (future use)
}
```

## Usage

### 1. Genesis Configuration

Configure envelopes and pre-TGE allocations in your runtime:

```rust
use pallet_token_allocation::{EnvelopeType, EnvelopeConfig};

// In your runtime configuration
pallet_token_allocation: PalletTokenAllocationConfig {
    envelope_wallets: vec![
        (
            EnvelopeType::Seed,
            1_000_000 * UNITS, // Initial balance (must be pre-minted)
            EnvelopeConfig {
                immediate_unlock_percentage: Percent::from_percent(10), // 10% immediate
                cliff_duration: 30 * DAYS_IN_BLOCKS, // 30 day cliff
                vesting_duration: 365 * DAYS_IN_BLOCKS, // 1 year vesting
            }
        ),
        (
            EnvelopeType::Private1,
            2_000_000 * UNITS,
            EnvelopeConfig {
                immediate_unlock_percentage: Percent::from_percent(5),
                cliff_duration: 90 * DAYS_IN_BLOCKS, // 3 month cliff
                vesting_duration: 2 * 365 * DAYS_IN_BLOCKS, // 2 year vesting
            }
        ),
        // ... other envelopes
    ],
    allocations: vec![
        // Pre-TGE contributor allocations
        (
            AccountId32::from([1u8; 32]), // beneficiary
            TokenAllocation {
                total_allocation: 50_000 * UNITS,
                envelope_type: EnvelopeType::Seed,
                status: AllocationStatus::ActiveSinceGenesis,
                claimed_amount: 0,
            }
        ),
        // ... more pre-TGE allocations
    ],
}
```

**Important**: You must mint the exact amount of tokens to each envelope account before genesis.

### 2. Getting Envelope Addresses

Envelope accounts are generated deterministically:

```rust
let seed_envelope_account = TokenAllocation::envelope_account_id(&EnvelopeType::Seed);
// or
let seed_envelope_account = EnvelopeType::Seed.address::<Runtime>();
```

### 3. Post-Genesis Operations

#### Allocate Tokens from Envelope

Create new allocations for post-TGE contributors:

```rust
// Only callable by AllocationOrigin (typically governance)
TokenAllocation::allocate_from_envelope(
    origin,
    EnvelopeType::Private1,
    beneficiary_account,
    100_000 * UNITS, // allocation amount
    Some(block_number), // activation block (None for immediate)
)?;
```

#### Claim Vested Tokens

Beneficiaries claim their unlocked tokens:

```rust
// Called by the beneficiary
TokenAllocation::claim_tokens(
    RuntimeOrigin::signed(beneficiary),
    allocation_id, // starts from 0 for each beneficiary
)?;
```

## Security Features

### Over-allocation Prevention

The pallet prevents allocating more tokens than available in an envelope:

```rust
// This will fail if envelope doesn't have enough remaining tokens
let available = current_balance - distributed_amount;
ensure!(available >= amount, Error::InsufficientEnvelopeBalance);
```

### No Withdrawal Functions

Envelope accounts are **one-way**: tokens can only be:

- Added at genesis (via pre-minting)
- Claimed by beneficiaries through the vesting mechanism

There are **no admin withdrawal functions** to prevent fund extraction.

### Immutable Configuration

Once the blockchain launches, envelope configurations cannot be modified. This ensures:

- Predictable vesting schedules
- No parameter manipulation
- Trust in the tokenomics

## Vesting Calculation

The vesting calculation follows this logic:

1. **Immediate unlock**: `immediate_percentage * total_allocation`
2. **Cliff period**: Only immediate unlock available until `activation_block + cliff_duration`
3. **Linear vesting**: After cliff, linear unlock over `vesting_duration` blocks
4. **Full unlock**: At `activation_block + vesting_duration`, entire allocation is available

## Events

```rust
pub enum Event<T: Config> {
    /// An allocation was created [envelope_type, beneficiary, allocation_id, amount]
    AllocationCreated {
        envelope_type: EnvelopeType,
        beneficiary: AccountIdOf<T>,
        allocation_id: u32,
        amount: BalanceOf<T>,
    },

    /// Tokens were claimed [beneficiary, allocation_id, amount]
    TokensClaimed {
        beneficiary: AccountIdOf<T>,
        allocation_id: u32,
        amount: BalanceOf<T>,
    },

    /// An envelope wallet was created [envelope_type, wallet_account, total_allocation]
    EnvelopeWalletCreated {
        envelope_type: EnvelopeType,
        wallet_account: AccountIdOf<T>,
        total_allocation: BalanceOf<T>,
    },
}
```

## Storage

- `EnvelopeWallets`: Tracks distributed amounts per envelope
- `Allocations`: Maps beneficiary → allocation_id → allocation details
- `NextAllocationId`: Next allocation ID per beneficiary
- `EnvelopeConfigs`: Vesting parameters per envelope type

## Testing

Run tests with:

```bash
cargo test -p pallet-token-allocation
```

Key test scenarios:

- Basic envelope setup and allocation
- Vesting calculations and claiming
- Over-allocation prevention
- Genesis validation

## Integration

Add to your runtime's `Cargo.toml`:

```toml
[dependencies]
pallet-token-allocation = { path = "../pallets/token-allocation", default-features = false }
```

Configure in your runtime:

```rust
impl pallet_token_allocation::Config for Runtime {
    type Currency = Balances;
    type AllocationOrigin = EnsureRoot<AccountId>; // or governance
    type WeightInfo = (); // or custom weights
}
```

Add to your `construct_runtime!` macro:

```rust
TokenAllocation: pallet_token_allocation,
```

## License

GPL-3.0-or-later
