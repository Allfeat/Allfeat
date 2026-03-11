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

use allfeat_primitives::Balance;
use frame_support::parameter_types;
use shared_runtime::currency::AFT;

parameter_types! {
    pub const BaseDeposit: Balance = 10 * AFT;
    pub const VersionDeposit: Balance = 1 * AFT;
    pub const MaxVersionsPerAts: u32 = 100;
    pub const MaxAtsPerAccount: u32 = 1000;
}

impl pallet_ats::Config for Runtime {
    type RuntimeHoldReason = RuntimeHoldReason;
    type Currency = Balances;
    type BaseDeposit = BaseDeposit;
    type VersionDeposit = VersionDeposit;
    type MaxVersionsPerAts = MaxVersionsPerAts;
    type MaxAtsPerAccount = MaxAtsPerAccount;
    type WeightInfo = ();
}
