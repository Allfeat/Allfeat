use crate::{Runtime, genesis::token::tokenomics};
use sp_keyring::Sr25519Keyring;
use sp_runtime::BuildStorage;

pub mod token;

pub fn new_test_ext() -> sp_io::TestExternalities {
    let sudo = Sr25519Keyring::Charlie.to_account_id();
    let token_genesis = tokenomics(sudo, 0);

    let mut t = frame_system::GenesisConfig::<Runtime>::default()
        .build_storage()
        .unwrap();

    token_genesis.balances.assimilate_storage(&mut t).unwrap();
    token_genesis
        .allocations
        .assimilate_storage(&mut t)
        .unwrap();

    sp_io::TestExternalities::new(t)
}
