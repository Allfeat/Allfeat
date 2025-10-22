// This file is part of Allfeat.

// Copyright (C) 2022-2025 Allfeat.
// SPDX-License-Identifier: GPL-3.0-or-later

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.

use crate::{self as pallet_token_allocation, EnvConfigOf, EnvelopeId, InitialAllocation};
use frame_support::{
    PalletId, derive_impl, parameter_types, sp_runtime::BuildStorage, traits::Hooks,
};
use frame_system::{EnsureRoot, pallet_prelude::BlockNumberFor};
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
    pub const EpochDuration: u64 = 5;
    pub const MaxPayoutPerBlock: u32 = 5;
    pub const MaxAllocations: u32 = 5;
}

impl pallet_token_allocation::Config for Test {
    type Currency = Balances;
    type AdminOrigin = EnsureRoot<Self::AccountId>;
    type PalletId = TokenAllocPalletId;
    type EpochDuration = EpochDuration;
    type MaxPayoutsPerBlock = MaxPayoutPerBlock;
    type MaxAllocations = MaxAllocations;
    type RuntimeHoldReason = RuntimeHoldReason;
}

pub(crate) fn run_to_block(n: BlockNumberFor<Test>) {
    while System::block_number() < n {
        let b = System::block_number() + 1;
        System::set_block_number(b);
        let _ = TokenAllocation::on_initialize(b);
    }
}

pub(crate) fn new_test_ext(
    envelopes: Vec<(EnvelopeId, EnvConfigOf<Test>)>,
    initial_allocations: Vec<InitialAllocation<Test>>,
) -> sp_io::TestExternalities {
    let mut t = frame_system::GenesisConfig::<Test>::default()
        .build_storage()
        .unwrap();

    pallet_token_allocation::pallet::GenesisConfig::<Test> {
        envelopes,
        initial_allocations,
    }
    .assimilate_storage(&mut t)
    .unwrap();

    let mut ext = sp_io::TestExternalities::new(t);
    ext.execute_with(|| System::set_block_number(1));
    ext
}
