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

use core::time::Duration;

use frame_support::sp_runtime::RuntimeDebug;
use frame_support::{pallet_prelude::Zero, traits::DefensiveSaturating};
use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;

#[derive(Encode, Default, Decode, Clone, PartialEq, Eq, RuntimeDebug, MaxEncodedLen, TypeInfo)]
pub struct CertifState<Balance: Clone + Zero + DefensiveSaturating> {
	pub(crate) status: CertifStatus<Balance>,
}

impl<Balance: DefensiveSaturating + Zero + Clone> CertifState<Balance> {
	pub(crate) fn new() -> Self {
		CertifState { status: Default::default() }
	}
}

#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, MaxEncodedLen, TypeInfo)]
pub enum CertifStatus<Balance> {
	VOTING(VotingInfos<Balance>), // The MIDDS just got sealed by the provider and is awaiting votes.
	PRECERTIF(PrecertifInfos), // The MIDDS got the certification threshold and enter the pre-certification period.
	CERTIF, // The pre-certification period ended without conflict and the MIDDS is certified.
}

impl<Balance: Zero> Default for CertifStatus<Balance> {
	fn default() -> Self {
		CertifStatus::VOTING(VotingInfos { total_staked: Zero::zero() })
	}
}

#[derive(Encode, Default, Decode, Clone, PartialEq, Eq, RuntimeDebug, MaxEncodedLen, TypeInfo)]
pub struct VotingInfos<Balance> {
	total_staked: Balance,
}

impl<Balance: Clone + DefensiveSaturating> VotingInfos<Balance> {
	pub fn total_staked(&self) -> Balance {
		self.total_staked.clone()
	}

	pub(crate) fn add_staked(&mut self, amount: Balance) {
		self.total_staked = self.total_staked.clone().defensive_saturating_add(amount)
	}
}

#[derive(Encode, Default, Decode, Clone, PartialEq, Eq, RuntimeDebug, MaxEncodedLen, TypeInfo)]
pub struct PrecertifInfos {
	pub(crate) precertif_timestamp: Duration,
}

impl PrecertifInfos {
	pub fn precertif_timestamp(&self) -> Duration {
		self.precertif_timestamp
	}
}
