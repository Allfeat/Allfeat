use crate::{RuntimeBlockWeights, TransactionByteFee, WeightToFee};
use frame_support::weights::{Weight, WeightToFee as WeightToFeeTrait};
use frame_system::limits::BlockWeights;
use shared_runtime::fee_estimator::{
    self, DEFAULT_AFT_PRICE_USD, ExtrinsicFeeInfo, FeeEstimate, FeeReportConfig, aft_to_usd,
    balance_to_aft, estimate_fees, format_balance, format_usd, gcd,
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
        let min_mult_inner = min_multiplier.into_inner();
        let fixed_div = <pallet_transaction_payment::Multiplier as FixedPointNumber>::DIV;
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

        use pallet_ats::WeightInfo as AtsWeightInfo;
        use pallet_midds::WeightInfo as MiddsWeightInfo;

        let ats_registration_cost = crate::AtsRegistrationCost::get();
        let midds_byte_deposit = crate::musical_works::ByteDepositCost::get();

        // Estimated encoded sizes for MIDDS data
        let musical_work_size: u32 = 500;
        let recording_size: u32 = 400;
        let release_size: u32 = 600;

        let extrinsics = vec![
            // System
            ExtrinsicFeeInfo {
                pallet: "System",
                extrinsic: "remark",
                weight: <() as frame_system::WeightInfo>::remark(32),
                encoded_len: 100,
                deposit: 0,
            },
            ExtrinsicFeeInfo {
                pallet: "System",
                extrinsic: "set_code",
                weight: <() as frame_system::WeightInfo>::set_code(),
                encoded_len: 4_000_000,
                deposit: 0,
            },
            // Balances
            ExtrinsicFeeInfo {
                pallet: "Balances",
                extrinsic: "transfer_allow_death",
                weight: <() as pallet_balances::WeightInfo>::transfer_allow_death(),
                encoded_len: 120,
                deposit: 0,
            },
            ExtrinsicFeeInfo {
                pallet: "Balances",
                extrinsic: "transfer_keep_alive",
                weight: <() as pallet_balances::WeightInfo>::transfer_keep_alive(),
                encoded_len: 120,
                deposit: 0,
            },
            // Timestamp
            ExtrinsicFeeInfo {
                pallet: "Timestamp",
                extrinsic: "set",
                weight: <() as pallet_timestamp::WeightInfo>::set(),
                encoded_len: 50,
                deposit: 0,
            },
            // Utility
            ExtrinsicFeeInfo {
                pallet: "Utility",
                extrinsic: "batch (10 calls)",
                weight: <() as pallet_utility::WeightInfo>::batch(10),
                encoded_len: 500,
                deposit: 0,
            },
            ExtrinsicFeeInfo {
                pallet: "Utility",
                extrinsic: "batch_all (10 calls)",
                weight: <() as pallet_utility::WeightInfo>::batch_all(10),
                encoded_len: 500,
                deposit: 0,
            },
            // Scheduler
            ExtrinsicFeeInfo {
                pallet: "Scheduler",
                extrinsic: "schedule",
                weight: <() as pallet_scheduler::WeightInfo>::schedule(10),
                encoded_len: 200,
                deposit: 0,
            },
            ExtrinsicFeeInfo {
                pallet: "Scheduler",
                extrinsic: "cancel",
                weight: <() as pallet_scheduler::WeightInfo>::cancel(10),
                encoded_len: 100,
                deposit: 0,
            },
            // Preimage
            ExtrinsicFeeInfo {
                pallet: "Preimage",
                extrinsic: "note_preimage (1KB)",
                weight: <() as pallet_preimage::WeightInfo>::note_preimage(1024),
                encoded_len: 1100,
                deposit: 0,
            },
            ExtrinsicFeeInfo {
                pallet: "Preimage",
                extrinsic: "unnote_preimage",
                weight: <() as pallet_preimage::WeightInfo>::unnote_preimage(),
                encoded_len: 100,
                deposit: 0,
            },
            // Proxy
            ExtrinsicFeeInfo {
                pallet: "Proxy",
                extrinsic: "proxy",
                weight: <() as pallet_proxy::WeightInfo>::proxy(5),
                encoded_len: 150,
                deposit: 0,
            },
            ExtrinsicFeeInfo {
                pallet: "Proxy",
                extrinsic: "add_proxy",
                weight: <() as pallet_proxy::WeightInfo>::add_proxy(5),
                encoded_len: 150,
                deposit: 0,
            },
            ExtrinsicFeeInfo {
                pallet: "Proxy",
                extrinsic: "remove_proxy",
                weight: <() as pallet_proxy::WeightInfo>::remove_proxy(5),
                encoded_len: 150,
                deposit: 0,
            },
            // Multisig
            ExtrinsicFeeInfo {
                pallet: "Multisig",
                extrinsic: "as_multi (3 signers)",
                weight: <() as pallet_multisig::WeightInfo>::as_multi_create(3, 500),
                encoded_len: 600,
                deposit: 0,
            },
            ExtrinsicFeeInfo {
                pallet: "Multisig",
                extrinsic: "approve_as_multi",
                weight: <() as pallet_multisig::WeightInfo>::approve_as_multi_create(3),
                encoded_len: 200,
                deposit: 0,
            },
            // Sudo
            ExtrinsicFeeInfo {
                pallet: "Sudo",
                extrinsic: "sudo",
                weight: <() as pallet_sudo::WeightInfo>::sudo(),
                encoded_len: 200,
                deposit: 0,
            },
            // Validators
            ExtrinsicFeeInfo {
                pallet: "Validators",
                extrinsic: "add_validator",
                weight: <() as pallet_validators::WeightInfo>::add_validator(),
                encoded_len: 100,
                deposit: 0,
            },
            ExtrinsicFeeInfo {
                pallet: "Validators",
                extrinsic: "remove_validator",
                weight: <() as pallet_validators::WeightInfo>::remove_validator(),
                encoded_len: 100,
                deposit: 0,
            },
            // Identity
            ExtrinsicFeeInfo {
                pallet: "Identity",
                extrinsic: "set_identity",
                weight: <() as pallet_identity::WeightInfo>::set_identity(10),
                encoded_len: 500,
                deposit: 0,
            },
            ExtrinsicFeeInfo {
                pallet: "Identity",
                extrinsic: "clear_identity",
                weight: <() as pallet_identity::WeightInfo>::clear_identity(10, 5),
                encoded_len: 100,
                deposit: 0,
            },
            // ATS
            ExtrinsicFeeInfo {
                pallet: "ATS",
                extrinsic: "register",
                weight: <() as AtsWeightInfo>::register(500),
                encoded_len: 600,
                deposit: ats_registration_cost,
            },
            ExtrinsicFeeInfo {
                pallet: "ATS",
                extrinsic: "update",
                weight: <() as AtsWeightInfo>::update(500),
                encoded_len: 600,
                deposit: ats_registration_cost,
            },
            ExtrinsicFeeInfo {
                pallet: "ATS",
                extrinsic: "claim",
                weight: <() as AtsWeightInfo>::claim(),
                encoded_len: 150,
                deposit: 0,
            },
            ExtrinsicFeeInfo {
                pallet: "ATS",
                extrinsic: "set_verification_key",
                weight: <() as AtsWeightInfo>::set_verification_key(),
                encoded_len: 100,
                deposit: 0,
            },
            // MIDDS - MusicalWorks
            ExtrinsicFeeInfo {
                pallet: "MIDDS (Works)",
                extrinsic: "register",
                weight: <() as MiddsWeightInfo>::register(musical_work_size),
                encoded_len: musical_work_size + 100,
                deposit: midds_byte_deposit.saturating_mul(musical_work_size as u128),
            },
            ExtrinsicFeeInfo {
                pallet: "MIDDS (Works)",
                extrinsic: "unregister",
                weight: <() as MiddsWeightInfo>::unregister(),
                encoded_len: 100,
                deposit: 0,
            },
            // MIDDS - Recordings
            ExtrinsicFeeInfo {
                pallet: "MIDDS (Rec.)",
                extrinsic: "register",
                weight: <() as MiddsWeightInfo>::register(recording_size),
                encoded_len: recording_size + 100,
                deposit: midds_byte_deposit.saturating_mul(recording_size as u128),
            },
            ExtrinsicFeeInfo {
                pallet: "MIDDS (Rec.)",
                extrinsic: "unregister",
                weight: <() as MiddsWeightInfo>::unregister(),
                encoded_len: 100,
                deposit: 0,
            },
            // MIDDS - Releases
            ExtrinsicFeeInfo {
                pallet: "MIDDS (Rel.)",
                extrinsic: "register",
                weight: <() as MiddsWeightInfo>::register(release_size),
                encoded_len: release_size + 100,
                deposit: midds_byte_deposit.saturating_mul(release_size as u128),
            },
            ExtrinsicFeeInfo {
                pallet: "MIDDS (Rel.)",
                extrinsic: "unregister",
                weight: <() as MiddsWeightInfo>::unregister(),
                encoded_len: 100,
                deposit: 0,
            },
        ];

        let estimates: Vec<FeeEstimate> = extrinsics
            .iter()
            .map(|info| estimate_fees(info, &config))
            .collect();

        fee_estimator::print_fee_report("ALLFEAT MELODIE (TESTNET)", &estimates, &config);

        // Print additional ATS cost summary
        println!("=== ATS Registration Total Cost Summary ===");
        println!(
            "  ATS Registration Deposit:     {}",
            format_balance(ats_registration_cost)
        );
        let ats_reg_est = estimates
            .iter()
            .find(|e| e.pallet == "ATS" && e.extrinsic == "register")
            .unwrap();
        println!(
            "  + Min Transaction Fee:        {}",
            format_balance(ats_reg_est.min_fee)
        );
        println!(
            "  + Max Transaction Fee:        {}",
            format_balance(ats_reg_est.max_fee)
        );
        println!(
            "  = Total Min:                  {} ({})",
            format_balance(ats_reg_est.total_min),
            format_usd(aft_to_usd(
                balance_to_aft(ats_reg_est.total_min),
                DEFAULT_AFT_PRICE_USD
            ))
        );
        println!(
            "  = Total Max:                  {} ({})",
            format_balance(ats_reg_est.total_max),
            format_usd(aft_to_usd(
                balance_to_aft(ats_reg_est.total_max),
                DEFAULT_AFT_PRICE_USD
            ))
        );

        println!();
        println!("=== MIDDS Registration Deposit Summary ===");
        println!(
            "  ByteDepositCost:              {} / byte",
            format_balance(midds_byte_deposit)
        );
        println!(
            "  MusicalWork (~{} bytes):     {}",
            musical_work_size,
            format_balance(midds_byte_deposit.saturating_mul(musical_work_size as u128))
        );
        println!(
            "  Recording (~{} bytes):       {}",
            recording_size,
            format_balance(midds_byte_deposit.saturating_mul(recording_size as u128))
        );
        println!(
            "  Release (~{} bytes):         {}",
            release_size,
            format_balance(midds_byte_deposit.saturating_mul(release_size as u128))
        );
        println!();
    });
}
