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

use super::Stakeholders;
use crate::*;
use frame_support::{parameter_types, PalletId};
use frame_system::EnsureSigned;
use shared_runtime::currency::ALFT;
use shared_runtime::{currency::MILLIALFT, weights};

parameter_types! {
	pub const StakeholderPalletId: PalletId = PalletId(*b"m/stkhld");
	pub const UnregisterPeriod: u32 = 7 * DAYS;
	pub const ByteDepositCost: Balance = MILLIALFT;
	pub const MaxDepositCost: Balance = 100 * ALFT;
}

impl pallet_midds::Config<Stakeholders> for Runtime {
	type PalletId = StakeholderPalletId;
	type RuntimeEvent = RuntimeEvent;
	type Currency = Balances;
	type RuntimeHoldReason = RuntimeHoldReason;
	type MIDDS = midds_stakeholder::Stakeholder<Self::Hashing>;
	type ProviderOrigin = EnsureSigned<Self::AccountId>;
	type ByteDepositCost = ByteDepositCost;
	type MaxDepositCost = ();
	type UnregisterPeriod = UnregisterPeriod;
	type WeightInfo = weights::midds_stakeholders::AllfeatWeight<Runtime>;
}
