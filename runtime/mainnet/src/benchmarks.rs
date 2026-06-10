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

polkadot_sdk::frame_benchmarking::define_benchmarks!(
	[frame_system, SystemBench::<Runtime>]
	[cumulus_pallet_parachain_system, ParachainSystem]
	[pallet_timestamp, Timestamp]
	[pallet_balances, Balances]
	[pallet_sudo, Sudo]
	[pallet_collator_selection, CollatorSelection]
	[pallet_session, SessionBench::<Runtime>]
	[cumulus_pallet_xcmp_queue, XcmpQueue]
	[pallet_message_queue, MessageQueue]
	[cumulus_pallet_weight_reclaim, WeightReclaim]
	[pallet_utility, Utility]
	[pallet_multisig, Multisig]
	[pallet_proxy, Proxy]
	[pallet_scheduler, Scheduler]
	[pallet_preimage, Preimage]
	[pallet_treasury, Treasury]
	[pallet_token_allocation, TokenAllocation]
	[pallet_meta_tx, MetaTx]
	[pallet_verify_signature, VerifySignature]
	[pallet_ats, Ats]
);
