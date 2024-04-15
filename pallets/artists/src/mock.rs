// This file is part of Allfeat.

// Copyright (C) Allfeat (FR) Ltd.
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! # Artists test environment.

use super::*;
use crate as pallet_artists;
use frame_support::derive_impl;
use frame_support::traits::{ConstU128, ConstU16, ConstU32, ConstU64};
use frame_support::{parameter_types, PalletId};
use frame_system::EnsureRoot;
use sp_runtime::testing::H256;
use sp_runtime::traits::{BlakeTwo256, IdentityLookup};
use sp_runtime::BuildStorage;

type Block = frame_system::mocking::MockBlock<Test>;

frame_support::construct_runtime!(
    pub enum Test
    {
        System: frame_system,
        Balances: pallet_balances,
        Artists: pallet_artists,
    }
);

#[derive_impl(frame_system::config_preludes::TestDefaultConfig as frame_system::DefaultConfig)]
impl frame_system::Config for Test {
    type BaseCallFilter = frame_support::traits::Everything;
    type BlockWeights = ();
    type BlockLength = ();
    type DbWeight = ();
    type RuntimeOrigin = RuntimeOrigin;
    type RuntimeCall = RuntimeCall;
    type Nonce = u64;
    type Hash = H256;
    type Hashing = BlakeTwo256;
    type AccountId = u64;
    type Lookup = IdentityLookup<Self::AccountId>;
    type Block = Block;
    type RuntimeEvent = RuntimeEvent;
    type BlockHashCount = ConstU64<250>;
    type Version = ();
    type PalletInfo = PalletInfo;
    type AccountData = pallet_balances::AccountData<u128>;
    type OnNewAccount = ();
    type OnKilledAccount = ();
    type SystemWeightInfo = ();
    type SS58Prefix = ConstU16<42>;
    type OnSetCode = ();
    type MaxConsumers = frame_support::traits::ConstU32<16>;
}

impl pallet_balances::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type WeightInfo = ();
    type Balance = u128;
    type DustRemoval = ();
    type ExistentialDeposit = ConstU128<1>;
    type AccountStore = System;
    type ReserveIdentifier = [u8; 8];
    type RuntimeHoldReason = RuntimeHoldReason;
    type RuntimeFreezeReason = RuntimeFreezeReason;
    type FreezeIdentifier = ();
    type MaxLocks = ();
    type MaxReserves = ();
    type MaxFreezes = ();
}

parameter_types! {
    pub const ArtistsPalletId: PalletId = PalletId(*b"py/artst");
}

impl Config for Test {
    type PalletId = ArtistsPalletId;
    type RuntimeEvent = RuntimeEvent;
    type Currency = Balances;
    type BaseDeposit = ConstU128<5>;
    type ByteDeposit = ConstU128<1>;
    type RuntimeHoldReason = RuntimeHoldReason;
    type RootOrigin = EnsureRoot<Self::AccountId>;
    type Slash = ();
    type UnregisterPeriod = ConstU32<10>;
    type MaxNameLen = ConstU32<64>;
    type MaxGenres = ConstU32<5>;
    type MaxAssets = ConstU32<32>;
    type MaxContracts = ConstU32<2048>;
    type WeightInfo = ();
}

pub fn new_test_ext() -> sp_io::TestExternalities {
    let mut t = frame_system::GenesisConfig::<Test>::default()
        .build_storage()
        .unwrap();
    let balances = pallet_balances::GenesisConfig::<Test> {
        balances: vec![(1, 500), (2, 500), (3, 500), (4, 500), (5, 500)],
    };
    balances.assimilate_storage(&mut t).unwrap();
    t.into()
}
