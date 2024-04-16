// This file is part of Allfeat.

// Copyright (C) 2022-2024 Allfeat.
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

//! # Artists test environment.

use super::*;
use crate as pallet_artists;
use frame_support::{
	derive_impl, parameter_types,
	traits::{ConstU128, ConstU16, ConstU32, ConstU64},
};
use frame_system::EnsureRoot;
use sp_runtime::{
	testing::H256,
	traits::{BlakeTwo256, IdentityLookup},
	BuildStorage,
};

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
	type WeightInfo = ();
}

pub fn new_test_ext() -> sp_io::TestExternalities {
	let mut t = frame_system::GenesisConfig::<Test>::default().build_storage().unwrap();
	let balances = pallet_balances::GenesisConfig::<Test> {
		balances: vec![(1, 500), (2, 500), (3, 500), (4, 500), (5, 500)],
	};
	balances.assimilate_storage(&mut t).unwrap();
	t.into()
}
