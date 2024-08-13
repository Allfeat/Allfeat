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

parameter_types! {
	pub const MaxNominatorRewardedPerValidator: u32 = 0;
}

parameter_types! {
	pub MaxSetIdSessionEntries: u32 = 0;
}

impl pallet_grandpa::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;

	type KeyOwnerProof = <Historical as frame_support::traits::KeyOwnerProofSystem<(
		sp_runtime::KeyTypeId,
		sp_consensus_grandpa::AuthorityId,
	)>>::Proof;

	type EquivocationReportSystem = ();
	type MaxNominators = MaxNominatorRewardedPerValidator;
	type WeightInfo = ();
	type MaxAuthorities = MaxAuthorities;
	type MaxSetIdSessionEntries = MaxSetIdSessionEntries;
}
