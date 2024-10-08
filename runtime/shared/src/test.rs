#[macro_export]
macro_rules! impl_evm_tests {
	() => {
		mod evm {
			// allfeat
			use super::mock::*;
			// substrate
			use frame_support::assert_err;
			use sp_core::{H160, U256};
			use sp_runtime::{DispatchError, ModuleError};

			#[test]
			fn configured_base_extrinsic_weight_is_evm_compatible() {
				let min_ethereum_transaction_weight = WeightPerGas::get() * 21_000;
				let base_extrinsic = <Runtime as frame_system::Config>::BlockWeights::get()
					.get(frame_support::dispatch::DispatchClass::Normal)
					.base_extrinsic;

				assert!(base_extrinsic.ref_time() <= min_ethereum_transaction_weight.ref_time());
			}

			#[test]
			fn evm_constants_are_correctly() {
				assert_eq!(BlockGasLimit::get(), U256::from(500_000_000));
				assert_eq!(WeightPerGas::get().ref_time(), 6000);
			}

			#[test]
			fn pallet_evm_calls_only_callable_by_root() {
				ExtBuilder::default().build().execute_with(|| {
					// https://github.com/darwinia-network/darwinia/blob/5923b2e0526b67fe05cee6e4e592ceca80e8f2ff/runtime/darwinia/src/pallets/evm.rs#L136
					assert_err!(
						EVM::call(
							RuntimeOrigin::signed(H160::default().into()),
							H160::default(),
							H160::default(),
							vec![],
							U256::default(),
							1000000,
							U256::from(1_000_000),
							None,
							None,
							vec![],
						),
						DispatchError::BadOrigin
					);

					if let Err(dispatch_info_with_err) = EVM::call(
						RuntimeOrigin::root(),
						H160::default(),
						H160::default(),
						vec![],
						U256::default(),
						1000000,
						U256::from(1_000_000),
						None,
						None,
						vec![],
					) {
						assert_eq!(
							dispatch_info_with_err.error,
							DispatchError::Module(ModuleError {
								index: 51,
								error: [4, 0, 0, 0],
								message: Some("GasPriceTooLow")
							})
						);
					}
				});
			}
		}
	};
}

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

			// The fee for one transfer is at most 1 AFT.
			#[test]
			fn sane_transfer_fee() {
				// substrate
				use pallet_balances::WeightInfo;

				let base = RuntimeBlockWeights::get().get(DispatchClass::Normal).base_extrinsic;
				let transfer =
					base + weights::balances::AllfeatWeight::<Runtime>::transfer_allow_death();
				let fee = WeightToFee::weight_to_fee(&transfer);

				assert!(fee <= AFT, "{} MILLIAFT should be at most 1000", fee / MILLIAFT);
			}

			// Weight is being charged for both dimensions.
			#[test]
			#[ignore]
			fn weight_charged_for_both_components() {
				let fee = WeightToFee::weight_to_fee(&Weight::from_parts(10_000, 0));
				assert!(!fee.is_zero(), "Charges for ref time");

				let fee = WeightToFee::weight_to_fee(&Weight::from_parts(0, 10_000));
				assert_eq!(fee, AFT, "10kb maps to AFT");
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
			// frontier
			use fp_evm::FeeCalculator;
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
				let target = TargetBlockFullness::get() *
					RuntimeBlockWeights::get().get(DispatchClass::Normal).max_total.unwrap();
				// if the min is too small, then this will not change, and we are doomed forever.
				// the weight is 1/100th bigger than target.
				run_with_system_weight(target.saturating_mul(101) / 100, || {
					let next = SlowAdjustingFeeUpdate::<Runtime>::convert(minimum_multiplier);
					assert!(next > minimum_multiplier, "{:?} !>= {:?}", next, minimum_multiplier);
				})
			}

			#[test]
			fn initial_evm_gas_fee_is_correct() {
				ExtBuilder::default().build().execute_with(|| {
					assert_eq!(TransactionPayment::next_fee_multiplier(), Multiplier::from(1u128));
					assert_eq!(
						TransactionPaymentGasPrice::<Runtime, WeightPerGas>::min_gas_price().0,
						U256::from(723_391_258_219u128)
					);
				})
			}

			#[test]
			fn test_evm_fee_adjustment() {
				ExtBuilder::default().build().execute_with(|| {
					let sim = |fullness: Perbill, num_blocks: u64| -> U256 {
						let block_weight = NORMAL_DISPATCH_RATIO * MAXIMUM_BLOCK_WEIGHT * fullness;
						for i in 0..num_blocks {
							System::set_block_number(i as u32);
							System::set_block_consumed_resources(block_weight, 0);
							TransactionPayment::on_finalize(i as u32);
						}
						TransactionPaymentGasPrice::<Runtime, WeightPerGas>::min_gas_price().0
					};

					assert_eq!(sim(Perbill::from_percent(0), 1), U256::from(723_377_694_760u128));
					assert_eq!(sim(Perbill::from_percent(25), 1), U256::from(723_377_694_760u128));
					assert_eq!(sim(Perbill::from_percent(50), 1), U256::from(723_391_258_219u128));
					assert_eq!(sim(Perbill::from_percent(100), 1), U256::from(723_431_950_121u128));

					// 1 "real" hour (at 12-second blocks)
					assert_eq!(sim(Perbill::from_percent(0), 300), U256::from(719_374_068_892u128));
					assert_eq!(
						sim(Perbill::from_percent(25), 300),
						U256::from(719_374_068_892u128)
					);
					assert_eq!(
						sim(Perbill::from_percent(50), 300),
						U256::from(723_431_950_121u128)
					);
					assert_eq!(
						sim(Perbill::from_percent(100), 300),
						U256::from(735_743_450_400u128)
					);

					// 1 "real" day (at 12-second blocks)
					assert_eq!(
						sim(Perbill::from_percent(0), 7200),
						U256::from(642_830_759_540u128)
					);
					assert_eq!(
						sim(Perbill::from_percent(25), 7200),
						U256::from(642_830_759_540u128)
					);
					assert_eq!(
						sim(Perbill::from_percent(50), 7200),
						U256::from(735_743_450_400u128)
					);
					assert_eq!(
						sim(Perbill::from_percent(100), 7200),
						U256::from(1_103_101_994_350u128)
					);

					// 7 "real" day (at 12-second blocks)
					assert_eq!(
						sim(Perbill::from_percent(0), 50400),
						U256::from(428_753_209_848u128)
					);
					assert_eq!(
						sim(Perbill::from_percent(25), 50400),
						U256::from(428_753_209_848u128)
					);
					assert_eq!(
						sim(Perbill::from_percent(50), 50400),
						U256::from(1_103_101_994_350u128)
					);
					assert_eq!(
						sim(Perbill::from_percent(100), 50400),
						U256::from(18_786_268_507_874u128)
					);

					// 30 "real" day (at 12-second blocks)
					assert_eq!(
						sim(Perbill::from_percent(0), 259200),
						U256::from(145_602_671_486u128)
					);
					assert_eq!(
						sim(Perbill::from_percent(25), 259200),
						U256::from(145_602_671_486u128)
					);
				});
			}
		}
	};
}
