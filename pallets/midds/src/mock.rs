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

#![cfg(test)]

use crate::{self as pallet_midds};
use allfeat_support::traits::Midds;
use frame_support::{derive_impl, parameter_types, PalletId};
use frame_system::EnsureSigned;
use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use sp_core::RuntimeDebug;
use sp_runtime::BuildStorage;

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
	pub provider: <Test as frame_system::Config>::AccountId,
	pub value: u64,
}

impl Midds<<Test as frame_system::Config>::Hashing, <Test as frame_system::Config>::AccountId>
	for MockMiddsStruct
{
	type EditableFields = MockMiddsStructEdFields;

	fn provider(&self) -> <Test as frame_system::Config>::AccountId {
		self.provider
	}

	fn set_provider(&mut self, provider: <Test as frame_system::Config>::AccountId) {
		self.provider = provider
	}

	fn is_complete(&self) -> bool {
		true
	}

	fn hash(
		&self,
	) -> <<Test as frame_system::Config>::Hashing as sp_runtime::traits::Hash>::Output {
		let mut bytes = Vec::new();

		bytes.extend_from_slice(&self.value.encode());

		<<Test as frame_system::Config>::Hashing as sp_runtime::traits::Hash>::hash(&bytes)
	}

	fn total_bytes(&self) -> u32 {
		self.encoded_size() as u32
	}

	fn update_field(&mut self, data: Self::EditableFields) {
		match data {
			MockMiddsStructEdFields::Value(x) => {
				self.value = x;
			},
		}
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
	pub type Balances = pallet_balances;

	#[runtime::pallet_index(2)]
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

#[derive_impl(pallet_midds::config_preludes::TestDefaultConfig)]
impl pallet_midds::Config for Test {
	type PalletId = MiddsPalletId;
	type Currency = Balances;
	type MIDDS = MockMiddsStruct;
	type MIDDSEditableFields = MockMiddsStructEdFields;
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
