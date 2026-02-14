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

use super::Recordings;
use allfeat_midds::recording::Recording;
use allfeat_primitives::Balance;
use frame_support::{PalletId, parameter_types};
use frame_system::EnsureSigned;
use shared_runtime::currency::MILLIAFT;

#[cfg(feature = "runtime-benchmarks")]
use allfeat_midds::benchmarking::RecordingBenchmarkHelper;

parameter_types! {
    pub const RecordingPalletId: PalletId = PalletId(*b"m/rcordg");
    pub const ByteDepositCost: Balance = 10 * MILLIAFT; // 0.01 AFT / byte
}

#[cfg(not(feature = "runtime-benchmarks"))]
parameter_types! {
    pub const UnregisterPeriod: Option<Moment> = Some(30 * DAYS as u64);
}

#[cfg(feature = "runtime-benchmarks")]
parameter_types! {
    pub const UnregisterPeriod: Option<Moment> = None;
}

impl pallet_midds::Config<Recordings> for Runtime {
    type PalletId = RecordingPalletId;
    type Timestamp = Timestamp;
    type Currency = Balances;
    type RuntimeHoldReason = RuntimeHoldReason;
    type MIDDS = Recording;
    type ProviderOrigin = EnsureSigned<Self::AccountId>;
    type ByteDepositCost = ByteDepositCost;
    type UnregisterPeriod = UnregisterPeriod;
    type WeightInfo = weights::midds_recordings::AllfeatWeight<Runtime>;

    #[cfg(feature = "runtime-benchmarks")]
    type BenchmarkHelper = RecordingBenchmarkHelper;
}
