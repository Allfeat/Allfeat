use allfeat_primitives::{AccountId, Balance};
use alloc::vec;
use pallet_token_allocation::{EnvelopeConfig, EnvelopeId};
use shared_runtime::currency::AFT;
use sp_runtime::Percent;

use crate::{MONTHS, Runtime, Treasury};

pub const SUDO_AMOUNT: Balance = 1_000 * AFT;
pub const VALIDATOR_ENDOWMENT: Balance = 10 * AFT;

pub struct TokenGenesis {
    pub balances: pallet_balances::GenesisConfig<Runtime>,
    pub allocations: pallet_token_allocation::GenesisConfig<Runtime>,
}

pub fn tokenomics(sudo_key: AccountId, num_validators: u128) -> TokenGenesis {
    TokenGenesis {
        balances: pallet_balances::GenesisConfig {
            balances: vec![(sudo_key, SUDO_AMOUNT)],
            dev_accounts: None,
        },
        allocations: pallet_token_allocation::GenesisConfig {
            envelopes: vec![
                // Envelope name: Teams
                // Total amount: 67 000 000 AFT
                // 0% Upfront rate, 12 months of cliff (locked)
                // Vesting on 36 months
                (
                    EnvelopeId::Teams,
                    EnvelopeConfig {
                        total_cap: 67_000_000 * AFT,
                        upfront_rate: Percent::from_percent(0),
                        cliff: 12 * MONTHS,
                        vesting_duration: 36 * MONTHS,
                        unique_beneficiary: None,
                    },
                ),
                // Envelope name: KoL
                // Total amount: 3 000 000 AFT
                // 0% Upfront rate, 9 months of cliff
                // Vesting on 9 months
                (
                    EnvelopeId::KoL,
                    EnvelopeConfig {
                        total_cap: 3_000_000 * AFT,
                        upfront_rate: Percent::from_percent(0),
                        cliff: 9 * MONTHS,
                        vesting_duration: 9 * MONTHS,
                        unique_beneficiary: None,
                    },
                ),
                // Envelope name: Private1
                // Total amount: 120 000 000 AFT
                // 5% Upfront rate, 8 months of cliff
                // Vesting on 38 months
                (
                    EnvelopeId::Private1,
                    EnvelopeConfig {
                        total_cap: 120_000_000 * AFT,
                        upfront_rate: Percent::from_percent(5),
                        cliff: 8 * MONTHS,
                        vesting_duration: 38 * MONTHS,
                        unique_beneficiary: None,
                    },
                ),
                // Envelope name: Private2
                // Total amount: 80 000 000 AFT
                // 5% Upfront rate, 3 months of cliff
                // Vesting on 36 months
                (
                    EnvelopeId::Private2,
                    EnvelopeConfig {
                        total_cap: 80_000_000 * AFT,
                        upfront_rate: Percent::from_percent(5),
                        cliff: 3 * MONTHS,
                        vesting_duration: 36 * MONTHS,
                        unique_beneficiary: None,
                    },
                ),
                // Envelope name: Public1
                // Total amount: 30 000 000 AFT
                // 0% Upfront rate, no cliff
                // Vesting on 6 months
                (
                    EnvelopeId::Public1,
                    EnvelopeConfig {
                        total_cap: 30_000_000 * AFT,
                        upfront_rate: Percent::from_percent(0),
                        cliff: 0u32,
                        vesting_duration: 6 * MONTHS,
                        unique_beneficiary: None,
                    },
                ),
                // Envelope name: Public3
                // Total amount: 30 000 000 AFT
                // 0% Upfront rate, no cliff
                // Vesting on 6 months
                (
                    EnvelopeId::Public3,
                    EnvelopeConfig {
                        total_cap: 30_000_000 * AFT,
                        upfront_rate: Percent::from_percent(0),
                        cliff: 0u32,
                        vesting_duration: 6 * MONTHS,
                        unique_beneficiary: None,
                    },
                ),
                // Envelope name: Public2
                // Total amount: 75 000 000 AFT
                // 0% Upfront rate, 18 months of cliff
                // Vesting on 12 months
                (
                    EnvelopeId::Public2,
                    EnvelopeConfig {
                        total_cap: 75_000_000 * AFT,
                        upfront_rate: Percent::from_percent(0),
                        cliff: 18 * MONTHS,
                        vesting_duration: 12 * MONTHS,
                        unique_beneficiary: None,
                    },
                ),
                // Envelope name: Public4
                // Total amount: 80 000 000 AFT
                // 0% Upfront rate, 12 months of cliff
                // Vesting on 12 months
                (
                    EnvelopeId::Public4,
                    EnvelopeConfig {
                        total_cap: 80_000_000 * AFT,
                        upfront_rate: Percent::from_percent(0),
                        cliff: 12 * MONTHS,
                        vesting_duration: 12 * MONTHS,
                        unique_beneficiary: None,
                    },
                ),
                // Envelope name: Airdrop
                // Total amount: 10 000 000 AFT
                // 100% Upfront rate
                // No cliff, no vesting
                (
                    EnvelopeId::Airdrop,
                    EnvelopeConfig {
                        total_cap: 10_000_000 * AFT,
                        upfront_rate: Percent::from_percent(100),
                        cliff: 0u32,
                        vesting_duration: 0u32,
                        unique_beneficiary: None,
                    },
                ),
                // Envelope name: Community Rewards
                // Total amount: 260 000 000 AFT
                // 0% Upfront rate, 5 months of cliff
                // Vesting on 46 months
                // Beneficiary: Treasury
                (
                    EnvelopeId::CommunityRewards,
                    EnvelopeConfig {
                        total_cap: 260_000_000 * AFT,
                        upfront_rate: Percent::from_percent(0),
                        cliff: 5 * MONTHS,
                        vesting_duration: 46 * MONTHS,
                        unique_beneficiary: Some(Treasury::account_id()),
                    },
                ),
                // Envelope name: Listing
                // Total amount: 100 000 000 AFT
                // 0% Upfront rate, 4 months of cliff
                // Vesting on 12 months
                // Beneficiary: Treasury
                (
                    EnvelopeId::Listing, // CEX/DEX listings
                    EnvelopeConfig {
                        total_cap: 100_000_000 * AFT,
                        upfront_rate: Percent::from_percent(0),
                        cliff: 4 * MONTHS,
                        vesting_duration: 12 * MONTHS,
                        unique_beneficiary: Some(Treasury::account_id()),
                    },
                ),
                // Envelope name: Research & Development
                // Total amount: 125 000 000 AFT
                // 20% Upfront rate, no cliff
                // Vesting on 26 months
                // Beneficiary: Treasury
                //
                // Note: Available sudo key amount balance is taken from this envelope.
                (
                    EnvelopeId::ResearchDevelopment,
                    EnvelopeConfig {
                        total_cap: (125_000_000 * AFT)
                            .saturating_sub(SUDO_AMOUNT)
                            .saturating_sub(num_validators * VALIDATOR_ENDOWMENT),
                        upfront_rate: Percent::from_percent(20),
                        cliff: 0u32,
                        vesting_duration: 26 * MONTHS,
                        unique_beneficiary: Some(Treasury::account_id()),
                    },
                ),
                // Envelope name: Reserve
                // Total amount: 20 000 000 AFT
                // 100% Upfront rate
                // No cliff, no vesting
                // Beneficiary: Treasury
                (
                    EnvelopeId::Reserve,
                    EnvelopeConfig {
                        total_cap: 20_000_000 * AFT,
                        upfront_rate: Percent::from_percent(100),
                        cliff: 0u32,
                        vesting_duration: 0u32,
                        unique_beneficiary: Some(Treasury::account_id()),
                    },
                ),
            ],
            initial_allocations: vec![],
        },
    }
}
