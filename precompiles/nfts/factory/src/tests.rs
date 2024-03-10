use crate::{mock::*, NftsFactoryPrecompileCall};
use pallet_evm_precompile_nfts_tests::{mock_collection_config, ExtBuilder, ALICE};
use precompile_utils::testing::*;

type PCall = NftsFactoryPrecompileCall<Runtime>;
fn precompiles() -> Precompiles<Runtime> {
	PrecompilesValue::get()
}

#[test]
fn selectors() {
	assert!(PCall::create_selectors().contains(&0x28d66e67));
	assert!(PCall::set_accept_ownership_selectors().contains(&0x8c462cc0));
}

#[test]
fn create_works() {
	ExtBuilder::<Runtime>::default()
		.with_balances(vec![(ALICE.into(), 1000)])
		.build()
		.execute_with(|| {
			precompiles()
				.prepare_test(
					ALICE,
					Precompile1,
					PCall::create { admin: ALICE.into(), config: mock_collection_config() },
				)
				.execute_returns(true);
		})
}
