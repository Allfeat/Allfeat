#[macro_export]
macro_rules! impl_weight_tests {
	() => {
		mod weight {
			// allfeat
			use super::mock::*;
			use shared_runtime::{weights, WeightToFee};
			// substrate
			use frame_support::{
				dispatch::DispatchClass,
				weights::{Weight, WeightToFee as WeightToFeeT},
			};
			use sp_runtime::traits::Zero;

			// We can fit at least 1000 transfers in a block.
			#[test]
			fn sane_block_weight() {
				// substrate
				use pallet_balances::WeightInfo;

				let block = RuntimeBlockWeights::get().max_block;
				let base = RuntimeBlockWeights::get().get(DispatchClass::Normal).base_extrinsic;
				let transfer =
					base + weights::balances::AllfeatWeight::<Runtime>::transfer_allow_death();
				let fit = block.checked_div_per_component(&transfer).unwrap_or_default();

				assert!(fit >= 1000, "{} should be at least 1000", fit);
			}

			// The fee for one transfer is at most 1 ALFT.
			#[test]
			fn sane_transfer_fee() {
				// substrate
				use pallet_balances::WeightInfo;

				let base = RuntimeBlockWeights::get().get(DispatchClass::Normal).base_extrinsic;
				let transfer =
					base + weights::balances::AllfeatWeight::<Runtime>::transfer_allow_death();
				let fee = WeightToFee::weight_to_fee(&transfer);

				assert!(fee <= ALFT, "{} MILLIALFT should be at most 1000", fee / MILLIALFT);
			}

			// Weight is being charged for both dimensions.
			#[test]
			#[ignore]
			fn weight_charged_for_both_components() {
				let fee = WeightToFee::weight_to_fee(&Weight::from_parts(10_000, 0));
				assert!(!fee.is_zero(), "Charges for ref time");

				let fee = WeightToFee::weight_to_fee(&Weight::from_parts(0, 10_000));
				assert_eq!(fee, ALFT, "10kb maps to ALFT");
			}
		}
	};
}

#[macro_export]
macro_rules! impl_fee_tests {
	() => {
		mod transaction_fee {
			// allfeat
			use super::mock::*;
			use shared_runtime::{
				MinimumMultiplier, SlowAdjustingFeeUpdate, TargetBlockFullness,
				TransactionPaymentGasPrice, NORMAL_DISPATCH_RATIO,
			};
			// substrate
			use frame_support::{
				dispatch::DispatchClass, pallet_prelude::Weight, traits::OnFinalize,
			};
			use pallet_transaction_payment::Multiplier;
			use sp_core::U256;
			use sp_runtime::{traits::Convert, BuildStorage, Perbill};

			fn run_with_system_weight<F>(w: Weight, mut assertions: F)
			where
				F: FnMut(),
			{
				let mut t: sp_io::TestExternalities =
					<frame_system::GenesisConfig<Runtime>>::default()
						.build_storage()
						.unwrap()
						.into();
				t.execute_with(|| {
					System::set_block_consumed_resources(w, 0);
					assertions()
				});
			}

			#[test]
			fn multiplier_can_grow_from_zero() {
				let minimum_multiplier = MinimumMultiplier::get();
				let target = TargetBlockFullness::get()
					* RuntimeBlockWeights::get().get(DispatchClass::Normal).max_total.unwrap();
				// if the min is too small, then this will not change, and we are doomed forever.
				// the weight is 1/100th bigger than target.
				run_with_system_weight(target.saturating_mul(101) / 100, || {
					let next = SlowAdjustingFeeUpdate::<Runtime>::convert(minimum_multiplier);
					assert!(next > minimum_multiplier, "{:?} !>= {:?}", next, minimum_multiplier);
				})
			}
		}
	};
}
