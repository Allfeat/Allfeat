use crate::{mock::*, NftsPrecompileSet, NftsPrecompileSetCall};

type PCall = NftsPrecompileSetCall<Runtime>;

fn precompiles() -> NftsPrecompileSet<Runtime> {
	PrecompilesValue::get()
}

#[test]
fn selectors() {
	// Getters
	assert!(PCall::get_details_selectors().contains(&0xb87f86b7));
	// Extrinsics
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
	assert!(PCall::clear_all_transfer_approvals_selectors().contains(&0x6f83fe8a));
	assert!(PCall::lock_item_properties_selectors().contains(&0x91743611));
	assert!(PCall::set_collection_attribute_selectors().contains(&0xe8971f23));
	assert!(PCall::set_item_attribute_selectors().contains(&0x123ffb18));
	assert!(PCall::clear_collection_attribute_selectors().contains(&0x07ac98df));
	assert!(PCall::clear_item_attribute_selectors().contains(&0x29eaab3f));
	assert!(PCall::approve_item_attributes_selectors().contains(&0x620fea0d));
	assert!(PCall::cancel_item_attributes_approval_selectors().contains(&0xe96389a9));
	assert!(PCall::set_metadata_selectors().contains(&0x914384e8));
	assert!(PCall::clear_metadata_selectors().contains(&0xf7948baa));
	assert!(PCall::set_collection_metadata_selectors().contains(&0xee9b0247));
	assert!(PCall::clear_collection_metadata_selectors().contains(&0x8699f6de));
	assert!(PCall::set_collection_max_supply_selectors().contains(&0x5c59e577));
	assert!(PCall::update_mint_settings_selectors().contains(&0x9f8ca97d));
	assert!(PCall::set_price_selectors().contains(&0xfc019a21));
	assert!(PCall::buy_item_selectors().contains(&0x0a6169cf));
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
