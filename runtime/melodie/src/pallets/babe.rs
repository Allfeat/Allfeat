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

use crate::*;
use frame_support::parameter_types;
use pallets::grandpa::MaxNominatorRewardedPerValidator;

/// The BABE epoch configuration at genesis.
pub const BABE_GENESIS_EPOCH_CONFIG: sp_consensus_babe::BabeEpochConfiguration =
	sp_consensus_babe::BabeEpochConfiguration {
		c: PRIMARY_PROBABILITY,
		allowed_slots: sp_consensus_babe::AllowedSlots::PrimaryAndSecondaryPlainSlots,
	};

parameter_types! {
	// NOTE: Currently it is not possible to change the epoch duration after the chain has started.
	//       Attempting to do so will brick block production.
	pub EpochDuration: u64 = shared_runtime::prod_or_fast!(
		EPOCH_DURATION_IN_SLOTS as u64,
		2 * MINUTES as u64,
		"AFT_EPOCH_DURATION"
	);
	pub const ExpectedBlockTime: Moment = MILLISECS_PER_BLOCK;
}

impl pallet_babe::Config for Runtime {
	type EpochDuration = EpochDuration;
	type ExpectedBlockTime = ExpectedBlockTime;
	type EpochChangeTrigger = pallet_babe::ExternalTrigger;
	type DisabledValidators = Session;

	type KeyOwnerProof = <Historical as frame_support::traits::KeyOwnerProofSystem<(
		sp_runtime::KeyTypeId,
		pallet_babe::AuthorityId,
	)>>::Proof;

	type EquivocationReportSystem = ();

	type WeightInfo = ();
	type MaxAuthorities = MaxAuthorities;
	type MaxNominators = MaxNominatorRewardedPerValidator;
}
