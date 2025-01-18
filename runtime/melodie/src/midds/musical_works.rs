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

use super::MusicalWorks;
use allfeat_primitives::Balance;
use frame_support::{parameter_types, PalletId};
use frame_system::EnsureSigned;
use shared_runtime::{
	currency::{AFT, MILLIAFT},
	weights,
};

parameter_types! {
	pub const StakeholderPalletId: PalletId = PalletId(*b"m/muwork");
	pub const ByteDepositCost: Balance = 10 * MILLIAFT;
	pub const MaxDepositCost: Balance = 100 * AFT;
}

#[cfg(not(feature = "runtime-benchmarks"))]
parameter_types! {
	pub const UnregisterPeriod: Option<Moment> = Some(7 * DAYS as u64);
}

#[cfg(feature = "runtime-benchmarks")]
parameter_types! {
	pub const UnregisterPeriod: Option<Moment> = None;
}

impl pallet_midds::Config<MusicalWorks> for Runtime {
	type PalletId = StakeholderPalletId;
	type RuntimeEvent = RuntimeEvent;
	type Timestamp = Timestamp;
	type Currency = Balances;
	type RuntimeHoldReason = RuntimeHoldReason;
	type MIDDS = midds_musical_work::MusicalWork<
		Self::Hashing,
		<Self::Hashing as frame_support::sp_runtime::traits::Hash>::Output,
	>;
	type ProviderOrigin = EnsureSigned<Self::AccountId>;
	type ByteDepositCost = ByteDepositCost;
	type MaxDepositCost = MaxDepositCost;
	type UnregisterPeriod = UnregisterPeriod;
	type WeightInfo = weights::midds_musical_works::AllfeatWeight<Runtime>;
}
