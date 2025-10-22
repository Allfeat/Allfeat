use alloc::vec;
use pallet_token_allocation::{EnvelopeConfig, EnvelopeId};
use shared_runtime::currency::AFT;
use sp_runtime::Percent;

use crate::{MONTHS, Runtime, Treasury};

pub struct TokenGenesis {
    pub balances: pallet_balances::GenesisConfig<Runtime>,
    pub allocations: pallet_token_allocation::GenesisConfig<Runtime>,
}

pub fn tokenomics() -> TokenGenesis {
    TokenGenesis {
        balances: pallet_balances::GenesisConfig {
            balances: vec![],
            dev_accounts: None,
        },
        allocations: pallet_token_allocation::GenesisConfig {
            envelopes: vec![
                (
                    EnvelopeId::Founders,
                    EnvelopeConfig {
                        total_cap: 67_000_000 * AFT,
                        upfront_rate: Percent::from_percent(0),
                        cliff: 12 * MONTHS,
                        vesting_duration: 36 * MONTHS,
                        unique_beneficiary: None,
                    },
                ),
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
                (
                    EnvelopeId::ICO1,
                    EnvelopeConfig {
                        total_cap: 30_000_000 * AFT,
                        upfront_rate: Percent::from_percent(0),
                        cliff: 0u32,
                        vesting_duration: 6 * MONTHS,
                        unique_beneficiary: None,
                    },
                ),
                (
                    EnvelopeId::ICO2,
                    EnvelopeConfig {
                        total_cap: 30_000_000 * AFT,
                        upfront_rate: Percent::from_percent(0),
                        cliff: 0u32,
                        vesting_duration: 6 * MONTHS,
                        unique_beneficiary: None,
                    },
                ),
                (
                    EnvelopeId::Seed,
                    EnvelopeConfig {
                        total_cap: 75_000_000 * AFT,
                        upfront_rate: Percent::from_percent(0),
                        cliff: 18 * MONTHS,
                        vesting_duration: 12 * MONTHS,
                        unique_beneficiary: None,
                    },
                ),
                (
                    EnvelopeId::SerieA,
                    EnvelopeConfig {
                        total_cap: 80_000_000 * AFT,
                        upfront_rate: Percent::from_percent(0),
                        cliff: 12 * MONTHS,
                        vesting_duration: 12 * MONTHS,
                        unique_beneficiary: None,
                    },
                ),
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
                (
                    EnvelopeId::Exchanges, // CEX/DEX listings.
                    EnvelopeConfig {
                        total_cap: 100_000_000 * AFT,
                        upfront_rate: Percent::from_percent(0),
                        cliff: 4 * MONTHS,
                        vesting_duration: 12 * MONTHS,
                        unique_beneficiary: Some(Treasury::account_id()),
                    },
                ),
                (
                    EnvelopeId::ResearchDevelopment,
                    EnvelopeConfig {
                        total_cap: 125_000_000 * AFT,
                        upfront_rate: Percent::from_percent(20),
                        cliff: 0u32,
                        vesting_duration: 26 * MONTHS,
                        unique_beneficiary: Some(Treasury::account_id()),
                    },
                ),
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
