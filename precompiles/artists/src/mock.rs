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

//! Test utilities
use super::*;
use frame_support::{
	construct_runtime, derive_impl, parameter_types,
	traits::{Everything, SortedMembers},
	weights::Weight,
	PalletId,
};
use frame_system::EnsureRoot;
use pallet_evm::{EnsureAddressNever, EnsureAddressRoot};
use precompile_utils::{mock_account, precompile_set::*, testing::MockAccount};
use sp_core::{ConstU128, ConstU32};
use sp_runtime::{
	traits::{BlakeTwo256, IdentityLookup},
	BuildStorage, Perbill,
};

pub type AccountId = MockAccount;
pub type Balance = u128;

type Block = frame_system::mocking::MockBlockU32<Runtime>;

construct_runtime!(
	pub enum Runtime	{
		System: frame_system,
		Balances: pallet_balances,
		Evm: pallet_evm,
		Timestamp: pallet_timestamp,
		Artists: pallet_artists,
	}
);

parameter_types! {
	pub const BlockHashCount: u32 = 250;
	pub const MaximumBlockWeight: Weight = Weight::from_parts(1024, 1);
	pub const MaximumBlockLength: u32 = 2 * 1024;
	pub const AvailableBlockRatio: Perbill = Perbill::one();
	pub const SS58Prefix: u8 = 42;
}

#[derive_impl(frame_system::config_preludes::TestDefaultConfig as frame_system::DefaultConfig)]
impl frame_system::Config for Runtime {
	type BaseCallFilter = Everything;
	type DbWeight = ();
	type RuntimeOrigin = RuntimeOrigin;
	type Nonce = u64;
	type Block = Block;
	type RuntimeCall = RuntimeCall;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = AccountId;
	type Lookup = IdentityLookup<AccountId>;
	type RuntimeEvent = RuntimeEvent;
	type BlockHashCount = BlockHashCount;
	type Version = ();
	type PalletInfo = PalletInfo;
	type AccountData = pallet_balances::AccountData<Balance>;
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type BlockWeights = ();
	type BlockLength = ();
	type SS58Prefix = SS58Prefix;
	type OnSetCode = ();
	type MaxConsumers = frame_support::traits::ConstU32<16>;
}
parameter_types! {
	pub const ExistentialDeposit: u128 = 1;
}
impl pallet_balances::Config for Runtime {
	type MaxReserves = ();
	type ReserveIdentifier = [u8; 4];
	type MaxLocks = ();
	type Balance = Balance;
	type RuntimeEvent = RuntimeEvent;
	type DustRemoval = ();
	type ExistentialDeposit = ExistentialDeposit;
	type AccountStore = System;
	type WeightInfo = ();
	type RuntimeHoldReason = RuntimeHoldReason;
	type FreezeIdentifier = ();
	type RuntimeFreezeReason = ();
	type MaxFreezes = ();
}

const MAX_POV_SIZE: u64 = 5 * 1024 * 1024;
/// Block storage limit in bytes. Set to 40 KB.
const BLOCK_STORAGE_LIMIT: u64 = 40 * 1024;

parameter_types! {
	pub BlockGasLimit: U256 = U256::from(u64::MAX);
	pub PrecompilesValue: Precompiles<Runtime> = Precompiles::new();
	pub const WeightPerGas: Weight = Weight::from_parts(1, 0);
	pub GasLimitPovSizeRatio: u64 = {
		let block_gas_limit = BlockGasLimit::get().min(u64::MAX.into()).low_u64();
		block_gas_limit.saturating_div(MAX_POV_SIZE)
	};
	pub GasLimitStorageGrowthRatio: u64 = {
		let block_gas_limit = BlockGasLimit::get().min(u64::MAX.into()).low_u64();
		block_gas_limit.saturating_div(BLOCK_STORAGE_LIMIT)
	};
}

pub type Precompiles<R> =
	PrecompileSetBuilder<R, (PrecompileAt<AddressU64<1>, ArtistsPrecompile<R>>,)>;

pub type PCall = ArtistsPrecompileCall<Runtime>;

impl pallet_evm::Config for Runtime {
	type FeeCalculator = ();
	type GasWeightMapping = pallet_evm::FixedGasWeightMapping<Self>;
	type WeightPerGas = WeightPerGas;
	type CallOrigin = EnsureAddressRoot<AccountId>;
	type WithdrawOrigin = EnsureAddressNever<AccountId>;
	type AddressMapping = AccountId;
	type Currency = Balances;
	type RuntimeEvent = RuntimeEvent;
	type Runner = pallet_evm::runner::stack::Runner<Self>;
	type PrecompilesType = Precompiles<Runtime>;
	type PrecompilesValue = PrecompilesValue;
	type ChainId = ();
	type OnChargeTransaction = ();
	type BlockGasLimit = BlockGasLimit;
	type BlockHashMapping = pallet_evm::SubstrateBlockHashMapping<Self>;
	type FindAuthor = ();
	type OnCreate = ();
	type GasLimitPovSizeRatio = GasLimitPovSizeRatio;
	type Timestamp = Timestamp;
	type SuicideQuickClearLimit = ();
	type WeightInfo = pallet_evm::weights::SubstrateWeight<Runtime>;
}

parameter_types! {
	pub const MinimumPeriod: u64 = 5;
}
impl pallet_timestamp::Config for Runtime {
	type Moment = u64;
	type OnTimestampSet = ();
	type MinimumPeriod = MinimumPeriod;
	type WeightInfo = ();
}

mock_account!(RegistrarAndForceOrigin, |_| H160::repeat_byte(0x1C).into());
impl SortedMembers<MockAccount> for RegistrarAndForceOrigin {
	fn sorted_members() -> Vec<MockAccount> {
		vec![RegistrarAndForceOrigin.into()]
	}
	#[cfg(feature = "runtime-benchmarks")]
	fn add(_m: &MockAccount) {}
}

parameter_types! {
	pub const BasicDeposit: u64 = 10;
	pub const FieldDeposit: u64 = 10;
	pub const SubAccountDeposit: u64 = 10;
	pub const MaxSubAccounts: u32 = 2;
	pub const MaxAdditionalFields: u32 = 2;
	pub const MaxRegistrars: u32 = 20;
}
impl core::fmt::Debug for MaxAdditionalFields {
	fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
		write!(f, "<>")
	}
}

parameter_types! {
	pub const ArtistsPalletId: PalletId = PalletId(*b"py/artst");
}

impl pallet_artists::Config for Runtime {
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

#[derive(Default)]
pub(crate) struct ExtBuilder {
	/// Endowed accounts with balances
	balances: Vec<(AccountId, Balance)>,
}

impl ExtBuilder {
	/// Fund some accounts before starting the test
	pub(crate) fn with_balances(mut self, balances: Vec<(AccountId, Balance)>) -> Self {
		self.balances = balances;
		self
	}

	/// Build the test externalities for use in tests
	pub(crate) fn build(self) -> sp_io::TestExternalities {
		let mut t = frame_system::GenesisConfig::<Runtime>::default()
			.build_storage()
			.expect("Frame system builds valid default genesis config");

		pallet_balances::GenesisConfig::<Runtime> { balances: self.balances.clone() }
			.assimilate_storage(&mut t)
			.expect("Pallet balances storage can be assimilated");

		let mut ext = sp_io::TestExternalities::new(t);
		ext.execute_with(|| {
			System::set_block_number(1);
		});
		ext
	}
}
