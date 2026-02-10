use crate::{Runtime, RuntimeBlockWeights, TransactionByteFee, WeightToFee, weights};
use frame_support::weights::{Weight, WeightToFee as WeightToFeeTrait};
use frame_system::limits::BlockWeights;
use shared_runtime::fee_estimator::{
    self, ExtrinsicFeeInfo, FeeEstimate, FeeReportConfig, DEFAULT_AFT_PRICE_USD, estimate_fees,
    gcd,
};

fn base_extrinsic_weight() -> Weight {
    let block_weights: BlockWeights = RuntimeBlockWeights::get();
    block_weights
        .get(frame_support::dispatch::DispatchClass::Normal)
        .base_extrinsic
}

fn weight_to_fee(weight: Weight) -> allfeat_primitives::Balance {
    <WeightToFee as WeightToFeeTrait>::weight_to_fee(&weight)
}

#[test]
fn print_fee_report() {
    use frame_support::sp_runtime::FixedPointNumber;

    super::new_test_ext().execute_with(|| {
        // Read MinimumMultiplier from runtime
        let min_multiplier = shared_runtime::MinimumMultiplier::get();
        // Extract as rational: MinimumMultiplier.into_inner() / DIV
        // FixedU128 stores value * 10^18, so 0.1 = 10^17
        // We express it as num/den for integer arithmetic
        let min_mult_inner = min_multiplier.into_inner();
        let fixed_div = <pallet_transaction_payment::Multiplier as FixedPointNumber>::DIV as u128;
        // Simplify: gcd(min_mult_inner, fixed_div)
        let g = gcd(min_mult_inner, fixed_div);
        let min_num = min_mult_inner / g;
        let min_den = fixed_div / g;

        let config = FeeReportConfig {
            base_weight: base_extrinsic_weight(),
            byte_fee: TransactionByteFee::get(),
            weight_to_fee_fn: weight_to_fee,
            min_multiplier_num: min_num,
            min_multiplier_den: min_den,
            max_multiplier_num: 10,
            max_multiplier_den: 1,
            aft_price_usd: DEFAULT_AFT_PRICE_USD,
        };

        use frame_system::WeightInfo as _;
        use pallet_balances::WeightInfo as _;
        use pallet_multisig::WeightInfo as _;
        use pallet_preimage::WeightInfo as _;
        use pallet_proxy::WeightInfo as _;
        use pallet_scheduler::WeightInfo as _;
        use pallet_sudo::WeightInfo as _;
        use pallet_timestamp::WeightInfo as _;
        use pallet_token_allocation::WeightInfo as _;
        use pallet_treasury::WeightInfo as _;
        use pallet_utility::WeightInfo as _;
        use pallet_validators::WeightInfo as _;

        type SystemW = weights::system::AllfeatWeight<Runtime>;
        type BalancesW = weights::balances::AllfeatWeight<Runtime>;
        type TimestampW = weights::timestamp::AllfeatWeight<Runtime>;
        type UtilityW = weights::utility::AllfeatWeight<Runtime>;
        type SchedulerW = weights::scheduler::AllfeatWeight<Runtime>;
        type PreimageW = weights::preimage::AllfeatWeight<Runtime>;
        type ProxyW = weights::proxy::AllfeatWeight<Runtime>;
        type MultisigW = weights::multisig::AllfeatWeight<Runtime>;
        type TokenAllocationW = weights::token_allocation::AllfeatWeight<Runtime>;
        type TreasuryW = weights::treasury::AllfeatWeight<Runtime>;
        type SudoW = weights::sudo::AllfeatWeight<Runtime>;
        type ValidatorsW = weights::validators::AllfeatWeight<Runtime>;

        let extrinsics = vec![
            // System
            ExtrinsicFeeInfo {
                pallet: "System",
                extrinsic: "remark",
                weight: SystemW::remark(32),
                encoded_len: 100,
                deposit: 0,
            },
            ExtrinsicFeeInfo {
                pallet: "System",
                extrinsic: "set_code",
                weight: SystemW::set_code(),
                encoded_len: 4_000_000,
                deposit: 0,
            },
            // Balances
            ExtrinsicFeeInfo {
                pallet: "Balances",
                extrinsic: "transfer_allow_death",
                weight: BalancesW::transfer_allow_death(),
                encoded_len: 120,
                deposit: 0,
            },
            ExtrinsicFeeInfo {
                pallet: "Balances",
                extrinsic: "transfer_keep_alive",
                weight: BalancesW::transfer_keep_alive(),
                encoded_len: 120,
                deposit: 0,
            },
            // Timestamp
            ExtrinsicFeeInfo {
                pallet: "Timestamp",
                extrinsic: "set",
                weight: TimestampW::set(),
                encoded_len: 50,
                deposit: 0,
            },
            // Utility
            ExtrinsicFeeInfo {
                pallet: "Utility",
                extrinsic: "batch (10 calls)",
                weight: UtilityW::batch(10),
                encoded_len: 500,
                deposit: 0,
            },
            ExtrinsicFeeInfo {
                pallet: "Utility",
                extrinsic: "batch_all (10 calls)",
                weight: UtilityW::batch_all(10),
                encoded_len: 500,
                deposit: 0,
            },
            // Scheduler
            ExtrinsicFeeInfo {
                pallet: "Scheduler",
                extrinsic: "schedule",
                weight: SchedulerW::schedule(10),
                encoded_len: 200,
                deposit: 0,
            },
            ExtrinsicFeeInfo {
                pallet: "Scheduler",
                extrinsic: "cancel",
                weight: SchedulerW::cancel(10),
                encoded_len: 100,
                deposit: 0,
            },
            // Preimage
            ExtrinsicFeeInfo {
                pallet: "Preimage",
                extrinsic: "note_preimage (1KB)",
                weight: PreimageW::note_preimage(1024),
                encoded_len: 1100,
                deposit: 0,
            },
            ExtrinsicFeeInfo {
                pallet: "Preimage",
                extrinsic: "unnote_preimage",
                weight: PreimageW::unnote_preimage(),
                encoded_len: 100,
                deposit: 0,
            },
            // Proxy
            ExtrinsicFeeInfo {
                pallet: "Proxy",
                extrinsic: "proxy",
                weight: ProxyW::proxy(5),
                encoded_len: 150,
                deposit: 0,
            },
            ExtrinsicFeeInfo {
                pallet: "Proxy",
                extrinsic: "add_proxy",
                weight: ProxyW::add_proxy(5),
                encoded_len: 150,
                deposit: 0,
            },
            ExtrinsicFeeInfo {
                pallet: "Proxy",
                extrinsic: "remove_proxy",
                weight: ProxyW::remove_proxy(5),
                encoded_len: 150,
                deposit: 0,
            },
            // Multisig
            ExtrinsicFeeInfo {
                pallet: "Multisig",
                extrinsic: "as_multi (3 signers)",
                weight: MultisigW::as_multi_create(3, 500),
                encoded_len: 600,
                deposit: 0,
            },
            ExtrinsicFeeInfo {
                pallet: "Multisig",
                extrinsic: "approve_as_multi",
                weight: MultisigW::approve_as_multi_create(3),
                encoded_len: 200,
                deposit: 0,
            },
            // TokenAllocation
            ExtrinsicFeeInfo {
                pallet: "TokenAllocation",
                extrinsic: "add_allocation",
                weight: TokenAllocationW::add_allocation(),
                encoded_len: 200,
                deposit: 0,
            },
            // Treasury
            ExtrinsicFeeInfo {
                pallet: "Treasury",
                extrinsic: "spend_local",
                weight: TreasuryW::spend_local(),
                encoded_len: 150,
                deposit: 0,
            },
            // Sudo
            ExtrinsicFeeInfo {
                pallet: "Sudo",
                extrinsic: "sudo",
                weight: SudoW::sudo(),
                encoded_len: 200,
                deposit: 0,
            },
            // Validators
            ExtrinsicFeeInfo {
                pallet: "Validators",
                extrinsic: "add_validator",
                weight: ValidatorsW::add_validator(),
                encoded_len: 100,
                deposit: 0,
            },
            ExtrinsicFeeInfo {
                pallet: "Validators",
                extrinsic: "remove_validator",
                weight: ValidatorsW::remove_validator(),
                encoded_len: 100,
                deposit: 0,
            },
        ];

        let estimates: Vec<FeeEstimate> = extrinsics
            .iter()
            .map(|info| estimate_fees(info, &config))
            .collect();

        fee_estimator::print_fee_report("ALLFEAT MAINNET", &estimates, &config);
    });
}

