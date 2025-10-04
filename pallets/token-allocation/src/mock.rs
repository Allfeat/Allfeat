// This file is part of Allfeat.
// Copyright (C) 2022-2025 Allfeat.
// SPDX-License-Identifier: GPL-3.0-or-later

use crate as pallet_token_allocation;
use frame_support::{PalletId, derive_impl, parameter_types, sp_runtime::BuildStorage};
use frame_system::EnsureRoot;
use sp_runtime::traits::IdentityLookup;

type Block = frame_system::mocking::MockBlock<Test>;

#[frame_support::runtime]
mod runtime {
    #[runtime::runtime]
    #[runtime::derive(
        RuntimeCall,
        RuntimeEvent,
        RuntimeError,
        RuntimeOrigin,
        RuntimeFreezeReason,
        RuntimeTask,
        RuntimeHoldReason
    )]
    pub struct Test;

    #[runtime::pallet_index(0)]
    pub type System = frame_system;

    #[runtime::pallet_index(1)]
    pub type Balances = pallet_balances;

    #[runtime::pallet_index(2)]
    pub type TokenAllocation = pallet_token_allocation;
}

#[derive_impl(frame_system::config_preludes::TestDefaultConfig)]
impl frame_system::Config for Test {
    type Block = Block;
    type AccountId = u128; // u64 is not enough to hold bytes used to generate accounts
    type Lookup = IdentityLookup<Self::AccountId>;
    type AccountData = pallet_balances::AccountData<u64>;
}

#[derive_impl(pallet_balances::config_preludes::TestDefaultConfig)]
impl pallet_balances::Config for Test {
    type AccountStore = frame_system::Pallet<Test>;
}

parameter_types! {
    pub TokenAllocPalletId: PalletId = PalletId(*b"tkalloc8");
}


impl pallet_token_allocation::Config for Test {
    type Currency = Balances;

    type AdminOrigin = EnsureRoot<Self::AccountId>;
    type PalletId = TokenAllocPalletId;
}

pub(crate) fn new_test_ext() -> sp_io::TestExternalities {
    let mut t = frame_system::GenesisConfig::<Test>::default()
        .build_storage()
        .unwrap();

    pallet_balances::GenesisConfig::<Test> {
        balances: vec![(1, 1_000_000), (2, 1_000_000), (3, 1_000_000)],
        ..Default::default()
    }
    .assimilate_storage(&mut t)
    .unwrap();

    let mut ext = sp_io::TestExternalities::new(t);
    ext.execute_with(|| System::set_block_number(1));
    ext
}
