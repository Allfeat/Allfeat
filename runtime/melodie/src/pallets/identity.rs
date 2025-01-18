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

use crate::*;
use frame_support::{parameter_types, sp_runtime::traits::Verify, traits::ConstU32};
use frame_system::EnsureRoot;
use pallet_identity::legacy::IdentityInfo;
use shared_runtime::currency::deposit;

parameter_types! {
	// Minimum 4 MILLIAFT/byte
	pub const ByteDeposit: Balance = deposit(0, 1);
	pub const BasicDeposit: Balance = deposit(1, 258);
	pub const SubAccountDeposit: Balance = deposit(1, 53);
	pub const UsernameDeposit: Balance = deposit(0, 32);
	pub const MaxSubAccounts: u32 = 100;
	pub const MaxAdditionalFields: u32 = 100;
	pub const MaxRegistrars: u32 = 20;
}

impl pallet_identity::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type Currency = Balances;
	type BasicDeposit = BasicDeposit;
	type ByteDeposit = ByteDeposit;
	type SubAccountDeposit = SubAccountDeposit;
	type UsernameDeposit = UsernameDeposit;
	type UsernameGracePeriod = ConstU32<{ 30 * DAYS }>;
	type MaxSubAccounts = MaxSubAccounts;
	type MaxRegistrars = MaxRegistrars;
	type IdentityInformation = IdentityInfo<MaxAdditionalFields>;
	type Slashed = ();
	type ForceOrigin = EnsureRoot<AccountId>;
	type RegistrarOrigin = EnsureRoot<AccountId>;
	type OffchainSignature = Signature;
	type SigningPublicKey = <Signature as Verify>::Signer;
	type UsernameAuthorityOrigin = EnsureRoot<Self::AccountId>;
	type PendingUsernameExpiration = ConstU32<{ 7 * DAYS }>;
	type MaxSuffixLength = ConstU32<7>;
	type MaxUsernameLength = ConstU32<32>;
	type WeightInfo = shared_runtime::weights::identity::AllfeatWeight<Runtime>;
	//type WeightInfo = pallet_identity::weights::SubstrateWeight<Runtime>;
}
