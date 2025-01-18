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
use allfeat_support::traits::Midds;
use frame_support::{
	self, derive_impl,
	sp_runtime::{traits::Hash as HashT, BuildStorage, DispatchResult, RuntimeDebug},
	testing_prelude::*,
	PalletId,
};
use frame_system::EnsureSigned;
use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;

type Block = frame_system::mocking::MockBlock<Test>;

#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo)]
pub enum MockMiddsStructEdFields {
	Value(u64),
}

impl Default for MockMiddsStructEdFields {
	fn default() -> Self {
		Self::Value(0)
	}
}

#[derive(Encode, Default, Decode, Clone, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub struct MockMiddsStruct {
	pub value: u64,
}

impl Midds for MockMiddsStruct {
	type Hash = <Test as frame_system::Config>::Hashing;
	type EditableFields = MockMiddsStructEdFields;

	fn is_complete(&self) -> bool {
		true
	}

	fn is_valid(&self) -> bool {
		true // TODO write test for validity
	}

	fn hash(&self) -> <<Test as frame_system::Config>::Hashing as HashT>::Output {
		let mut bytes = Vec::new();

		bytes.extend_from_slice(&self.value.encode());

		<<Test as frame_system::Config>::Hashing as HashT>::hash(&bytes)
	}

	fn total_bytes(&self) -> u32 {
		self.encoded_size() as u32
	}

	fn update_field(&mut self, data: Self::EditableFields) -> DispatchResult {
		match data {
			MockMiddsStructEdFields::Value(x) => {
				self.value = x;
			},
		}
		Ok(())
	}

	#[cfg(feature = "runtime-benchmarks")]
	fn create_midds() -> Self {
		Self { value: 0 }
	}
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
	let mut t = frame_system::GenesisConfig::<Test>::default().build_storage().unwrap();

	pallet_balances::GenesisConfig::<Test> {
		balances: vec![(1, 10000), (2, 20000), (3, 30000), (4, 40000)],
	}
	.assimilate_storage(&mut t)
	.unwrap();

	let mut ext = sp_io::TestExternalities::new(t);
	ext.execute_with(|| System::set_block_number(1));
	ext
}
