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

use frame_support::{sp_runtime::Percent, traits::fungible::Inspect};
use parity_scale_codec::{Decode, DecodeWithMemTracking, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use serde::{Deserialize, Serialize};

use crate::Config;

pub type BalanceOf<T> =
    <<T as Config>::Currency as Inspect<<T as frame_system::Config>::AccountId>>::Balance;
pub type AccountIdOf<T> = <T as frame_system::Config>::AccountId;

#[derive(
    Debug,
    Encode,
    Decode,
    DecodeWithMemTracking,
    Clone,
    PartialEq,
    Eq,
    TypeInfo,
    MaxEncodedLen,
    Serialize,
    Deserialize,
)]
pub enum EnvelopeType {
    Founders,
    KoL,
    Private1,
    Private2,
    Ico1,
    Seed,
    Ico2,
    SerieA,
    Airdrop,
    CommunityReward,
}

#[derive(
    Debug,
    Encode,
    Decode,
    DecodeWithMemTracking,
    Clone,
    PartialEq,
    Eq,
    TypeInfo,
    MaxEncodedLen,
    Serialize,
    Deserialize,
)]
pub enum AllocationStatus<BlockNumber> {
    ActiveSinceGenesis,
    ActivatedAt(BlockNumber),
    Completed,
    Revoked,
}

#[derive(
    Debug,
    Encode,
    Decode,
    DecodeWithMemTracking,
    Clone,
    PartialEq,
    Eq,
    TypeInfo,
    MaxEncodedLen,
    Serialize,
    Deserialize,
)]
pub struct TokenAllocation<Balance, BlockNumber> {
    pub total_allocation: Balance,
    pub envelope_type: EnvelopeType,
    pub status: AllocationStatus<BlockNumber>,
    pub claimed_amount: Balance,
}

#[derive(
    Debug,
    Encode,
    Decode,
    DecodeWithMemTracking,
    Clone,
    PartialEq,
    Eq,
    TypeInfo,
    MaxEncodedLen,
    Serialize,
    Deserialize,
)]
pub struct EnvelopeWallet<Balance> {
    pub distributed_amount: Balance,
}

#[derive(
    Debug, Encode, Decode, DecodeWithMemTracking, Clone, PartialEq, Eq, TypeInfo, MaxEncodedLen, Serialize, Deserialize,
)]
pub struct EnvelopeConfig<BlockNumber> {
    pub immediate_unlock_percentage: Percent,
    pub cliff_duration: BlockNumber,
    pub vesting_duration: BlockNumber,
}

impl EnvelopeType {
    /// Returns the deterministic address for this envelope type
    pub fn address<T: Config>(&self) -> T::AccountId {
        crate::Pallet::<T>::envelope_account_id(self)
    }
}
