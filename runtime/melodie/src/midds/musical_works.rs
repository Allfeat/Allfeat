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
use allfeat_midds::musical_work::MusicalWork;
use allfeat_primitives::Balance;
use frame_support::{PalletId, parameter_types};
use frame_system::EnsureSigned;
use shared_runtime::{currency::MILLIAFT, weights};

#[cfg(feature = "runtime-benchmarks")]
use allfeat_midds::benchmarking::MusicalWorkBenchmarkHelper;

parameter_types! {
    pub const StakeholderPalletId: PalletId = PalletId(*b"m/muwork");
    pub const ByteDepositCost: Balance = MILLIAFT;
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
    type Timestamp = Timestamp;
    type Currency = Balances;
    type RuntimeHoldReason = RuntimeHoldReason;
    type MIDDS = MusicalWork;
    type ProviderOrigin = EnsureSigned<Self::AccountId>;
    type ByteDepositCost = ByteDepositCost;
    type UnregisterPeriod = UnregisterPeriod;
    type WeightInfo = weights::midds_musical_works::AllfeatWeight<Runtime>;

    #[cfg(feature = "runtime-benchmarks")]
    type BenchmarkHelper = MusicalWorkBenchmarkHelper;
}
