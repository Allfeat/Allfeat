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

#[cfg(feature = "runtime-benchmarks")]
frame_benchmarking::define_benchmarks!(
	[frame_benchmarking, BaselineBench::<Runtime>]
	[pallet_babe, Babe]
	[pallet_balances, Balances]
	[pallet_grandpa, Grandpa]
	[pallet_identity, Identity]
	[pallet_im_online, ImOnline]
	[pallet_midds_stakeholders, Stakeholders]
	[pallet_midds_musical_works, MusicalWorks]
	[pallet_mmr, Mmr]
	[pallet_multisig, Multisig]
	[pallet_preimage, Preimage]
	[pallet_proxy, Proxy]
	[pallet_scheduler, Scheduler]
	[pallet_sudo, Sudo]
	[frame_system, SystemBench::<Runtime>]
	[pallet_timestamp, Timestamp]
	[pallet_utility, Utility]
	[pallet_safe_mode, SafeMode]
);
