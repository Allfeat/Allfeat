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

use allfeat_support::traits::Midds;
use frame_support::{sp_runtime::traits::Member, Parameter};
use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;

#[derive(Encode, MaxEncodedLen, Decode, Clone, PartialEq, Eq, TypeInfo)]
pub struct MiddsWrapper<AccountId, Moment, Inner>
where
	AccountId: Parameter + Member,
	Moment: Parameter + Member,
	Inner: Midds + Parameter + Member,
{
	pub(crate) base: BaseInfos<AccountId, Moment>,
	pub midds: Inner,
}

impl<AccountId, Moment, Inner> MiddsWrapper<AccountId, Moment, Inner>
where
	AccountId: Parameter + Member,
	Moment: Parameter + Member,
	Inner: Midds + Parameter + Member,
{
	pub(crate) fn new(provider: AccountId, timestamp: Moment, inner_midds: Inner) -> Self {
		Self { base: BaseInfos { provider, registered_at: timestamp }, midds: inner_midds }
	}

	pub fn provider(&self) -> AccountId {
		self.base.provider.clone()
	}

	pub fn registered_at(&self) -> Moment {
		self.base.registered_at.clone()
	}
}

#[derive(Encode, MaxEncodedLen, Decode, Clone, PartialEq, Eq, TypeInfo)]
/// Basic informations on the MIDDS entity used to manage the MIDDS pallet database
pub struct BaseInfos<AccountId, Moment>
where
	AccountId: Parameter + Member,
{
	provider: AccountId,
	registered_at: Moment,
}
