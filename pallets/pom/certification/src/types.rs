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

use frame_support::pallet_prelude::Member;
use frame_support::sp_runtime::RuntimeDebug;
use frame_support::Parameter;
use frame_support::{pallet_prelude::Zero, traits::DefensiveSaturating};
use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;

pub use allfeat_support::types::CertifStatus;

#[derive(Encode, Default, Decode, Clone, PartialEq, Eq, RuntimeDebug, MaxEncodedLen, TypeInfo)]
pub struct CertifState<Balance: Parameter + Member + Zero + DefensiveSaturating> {
	pub(crate) status: CertifStatus<Balance>,
}

impl<Balance: Parameter + Member + DefensiveSaturating + Zero + Clone> CertifState<Balance> {
	pub(crate) fn new() -> Self {
		CertifState { status: Default::default() }
	}
}
