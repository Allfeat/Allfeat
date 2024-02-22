use crate::{mock::*, NftsPrecompileSet, SELECTOR_LOG_COLLECTION_CREATED};
use precompile_utils::{prelude::log3, solidity, testing::*};
use sp_core::U256;

fn precompiles() -> NftsPrecompileSet<Runtime> {
	PrecompilesValue::get()
}

#[test]
fn selectors() {
	// assert!(PCall::create_selectors().contains(&0x28d66e67));
	assert!(PCall::mint_selectors().contains(&0xcd568c38));
	assert!(PCall::burn_selectors().contains(&0x42966c68));
	assert!(PCall::transfer_selectors().contains(&0xb7760c8f));
	assert!(PCall::lock_item_transfer_selectors().contains(&0x81c2e1e8));
	assert!(PCall::unlock_item_transfer_selectors().contains(&0x3b8413a5));
	assert!(PCall::seal_collection_selectors().contains(&0xa872c4c8));
	assert!(PCall::transfer_ownership_selectors().contains(&0xf0350c04));
	assert!(PCall::set_team_selectors().contains(&0xf8bf8e95));
	assert!(PCall::approve_transfer_selectors().contains(&0x0df4508b));
	assert!(PCall::cancel_approval_selectors().contains(&0x22b856f3));
	assert!(PCall::clear_all_transfer_approvals_selectors().contains(&0x6f83fe8a))
}

/*#[test]
fn create_works() {
	ExtBuilder::default()
		.with_balances(vec![(ALICE.into(), 1000)])
		.build()
		.execute_with(|| {
			precompiles()
				.prepare_test(
					ALICE,
					Precompile1,
					PCall::create { admin: ALICE.into(), config: mock_collection_config() },
				)
				.expect_log(log3(
					Precompile1,
					SELECTOR_LOG_COLLECTION_CREATED,
					ALICE,
					ALICE,
					solidity::encode_event_data(U256::from(0)),
				))
				.execute_returns(true);
		})
}*/

// TODO mint_works
// TODO burn_works
