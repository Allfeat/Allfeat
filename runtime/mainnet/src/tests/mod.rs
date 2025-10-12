#![cfg(test)]

use crate::{Runtime, genesis::token::tokenomics};
use sp_runtime::BuildStorage;

pub mod token;

pub fn new_test_ext() -> sp_io::TestExternalities {
    let token_genesis = tokenomics();

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
