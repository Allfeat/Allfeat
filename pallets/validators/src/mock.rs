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

use frame_support::{derive_impl, parameter_types};
use pallet_session::TestSessionHandler;
use sp_runtime::{BuildStorage, testing::UintAuthorityId, traits::ConvertInto};

use crate as pallet_validators;

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
    pub type Validators = pallet_validators;

    #[runtime::pallet_index(3)]
    pub type Session = pallet_session;

    #[runtime::pallet_index(4)]
    pub type Historical = pallet_session::historical;
}

#[derive_impl(frame_system::config_preludes::TestDefaultConfig)]
impl frame_system::Config for Test {
    type Block = Block;

    type AccountData = pallet_balances::AccountData<u64>;
}

#[derive_impl(pallet_balances::config_preludes::TestDefaultConfig)]
impl pallet_balances::Config for Test {
    type AccountStore = frame_system::Pallet<Test>;
}

parameter_types! {
    pub const MaxValidators: u32 = 5;
    pub const Period: u64 = 3; // 3 blocks per session
    pub const Offset: u64 = 0;
}

impl pallet_session::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type Currency = Balances;
    type ShouldEndSession = pallet_session::PeriodicSessions<Period, Offset>;
    type SessionManager = pallet_session::historical::NoteHistoricalRoot<Test, Validators>;
    type SessionHandler = TestSessionHandler;
    type ValidatorId = u64;
    type ValidatorIdOf = ConvertInto;
    type Keys = UintAuthorityId;
    type NextSessionRotation = pallet_session::PeriodicSessions<Period, Offset>;
    type DisablingStrategy = ();
    type KeyDeposit = ();
    type WeightInfo = ();
}

impl pallet_session::historical::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type FullIdentification = u64;
    type FullIdentificationOf = sp_runtime::traits::ConvertInto;
}

impl pallet_validators::Config for Test {
    type MaxValidators = MaxValidators;
    type WeightInfo = ();
}

pub fn new_test_ext() -> sp_io::TestExternalities {
    let mut storage = frame_system::GenesisConfig::<Test>::default()
        .build_storage()
        .unwrap();

    pallet_validators::GenesisConfig::<Test> {
        initial_validators: vec![1, 2, 3],
    }
    .assimilate_storage(&mut storage)
    .unwrap();

    pallet_session::GenesisConfig::<Test> {
        keys: vec![
            (1, 1, UintAuthorityId::from(1)),
            (2, 2, UintAuthorityId::from(2)),
            (3, 3, UintAuthorityId::from(3)),
            (4, 4, UintAuthorityId::from(4)),
        ],
        non_authority_keys: Default::default(),
    }
    .assimilate_storage(&mut storage)
    .unwrap();

    sp_io::TestExternalities::new(storage)
}
