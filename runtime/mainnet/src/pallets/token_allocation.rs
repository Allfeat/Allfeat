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

use frame_support::{PalletId, parameter_types};
use frame_system::EnsureRoot;

use crate::*;

parameter_types! {
    pub const TokenAllocPalletId: PalletId = PalletId(*b"m/tknalc");
    pub const EpochDuration: BlockNumber = DAYS;
    pub const MaxPayoutsPerBlock: u32 = 256;
    pub const MaxAllocations: u32 = 10;
}

impl pallet_token_allocation::Config for Runtime {
    type Currency = Balances;
    type AdminOrigin = EnsureRoot<Self::AccountId>;
    type PalletId = TokenAllocPalletId;
    type EpochDuration = EpochDuration;
    type MaxPayoutsPerBlock = MaxPayoutsPerBlock;
    type MaxAllocations = MaxAllocations;
    type RuntimeHoldReason = RuntimeHoldReason;
}
