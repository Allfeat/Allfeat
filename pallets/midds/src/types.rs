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

use allfeat_support::traits::Midds;
use frame_support::Parameter;
use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use sp_runtime::traits::Member;

#[derive(Encode, MaxEncodedLen, Decode, Clone, PartialEq, Eq, TypeInfo)]
pub struct MiddsWrapper<AccountId, BlockNumber, Inner>
where
	AccountId: Parameter + Member,
	BlockNumber: Parameter + Member,
	Inner: Midds + Parameter + Member,
{
	pub(crate) base: BaseInfos<AccountId, BlockNumber>,
	pub midds: Inner,
}

impl<AccountId, BlockNumber, Inner> MiddsWrapper<AccountId, BlockNumber, Inner>
where
	AccountId: Parameter + Member,
	BlockNumber: Parameter + Member,
	Inner: Midds + Parameter + Member,
{
	pub(crate) fn new(provider: AccountId, timestamp: BlockNumber, inner_midds: Inner) -> Self {
		Self { base: BaseInfos { provider, registered_at: timestamp }, midds: inner_midds }
	}

	pub fn provider(&self) -> AccountId {
		self.base.provider.clone()
	}

	pub fn registered_at(&self) -> BlockNumber {
		self.base.registered_at.clone()
	}
}

#[derive(Encode, MaxEncodedLen, Decode, Clone, PartialEq, Eq, TypeInfo)]
/// Basic informations on the MIDDS entity used to manage the MIDDS pallet database
pub struct BaseInfos<AccountId, BlockNumber>
where
	AccountId: Parameter + Member,
{
	provider: AccountId,
	registered_at: BlockNumber,
}
