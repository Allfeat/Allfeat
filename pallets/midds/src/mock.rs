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

#![cfg(test)]

use crate::{self as pallet_midds};
use frame_support::{
    self, PalletId, derive_impl,
    sp_runtime::{BuildStorage, RuntimeDebug},
    testing_prelude::*,
};
use frame_system::EnsureSigned;

#[cfg(feature = "runtime-benchmarks")]
use midds::pallet_prelude::BenchmarkHelperT;

use midds::Midds;
use parity_scale_codec::{Decode, DecodeWithMemTracking, Encode, MaxEncodedLen};
use scale_info::TypeInfo;

type Block = frame_system::mocking::MockBlock<Test>;

#[derive(
    Encode,
    Default,
    Decode,
    DecodeWithMemTracking,
    Clone,
    Eq,
    PartialEq,
    RuntimeDebug,
    TypeInfo,
    MaxEncodedLen,
)]
pub struct MockMiddsStruct {
    pub value: u64,
}

#[cfg(feature = "runtime-benchmarks")]
pub struct BenchmarkHelperMock;
#[cfg(feature = "runtime-benchmarks")]
impl BenchmarkHelperT<MockMiddsStruct> for BenchmarkHelperMock {
    const FIELD_MAX_SIZE: u32 = 0;

    fn build_mock() -> MockMiddsStruct {
        MockMiddsStruct { value: 0 }
    }

    fn build_sized_mock(_size: u32) -> MockMiddsStruct {
        MockMiddsStruct { value: 0 }
    }
}

impl Midds for MockMiddsStruct {
    const NAME: &'static str = "MOCK";

    #[cfg(feature = "runtime-benchmarks")]
    type BenchmarkHelper = BenchmarkHelperMock;
}

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
    pub type Time = pallet_timestamp;

    #[runtime::pallet_index(2)]
    pub type Balances = pallet_balances;

    #[runtime::pallet_index(3)]
    pub type MockMidds = pallet_midds;
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
    pub MiddsPalletId: PalletId = PalletId(*b"mckmidds");
}

#[derive_impl(pallet_timestamp::config_preludes::TestDefaultConfig)]
impl pallet_timestamp::Config for Test {}

#[derive_impl(pallet_midds::config_preludes::TestDefaultConfig)]
impl pallet_midds::Config for Test {
    type PalletId = MiddsPalletId;
    type Timestamp = Time;
    type Currency = Balances;
    type MIDDS = MockMiddsStruct;
    type ProviderOrigin = EnsureSigned<Self::AccountId>;
}

pub(crate) fn new_test_ext() -> sp_io::TestExternalities {
    let mut t = frame_system::GenesisConfig::<Test>::default()
        .build_storage()
        .unwrap();

    pallet_balances::GenesisConfig::<Test> {
        balances: vec![(1, 10000), (2, 20000), (3, 30000), (4, 40000)],
        ..Default::default()
    }
    .assimilate_storage(&mut t)
    .unwrap();

    let mut ext = sp_io::TestExternalities::new(t);
    ext.execute_with(|| System::set_block_number(1));
    ext
}
