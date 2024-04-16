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
use frame_system::EnsureRoot;

parameter_types! {
	pub const MaxNameLen: u32 = 128;
	pub const MaxGenres: u32 = 5;
	pub const MaxAssets: u32 = 64;
	pub const ByteDesposit: Balance = deposit(0, 1);
	pub const BaseDeposit: Balance = 1 * AFT;
	pub const UnregisterPeriod: BlockNumber = 7 * DAYS;
	pub const ArtistsPalletId: frame_support::PalletId = frame_support::PalletId(*b"py/artst");
}

impl pallet_artists::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type PalletId = ArtistsPalletId;
	type RootOrigin = EnsureRoot<Self::AccountId>;
	type Slash = ();
	type Currency = Balances;
	type ByteDeposit = ByteDesposit;
	type BaseDeposit = BaseDeposit;
	type UnregisterPeriod = UnregisterPeriod;
	type RuntimeHoldReason = RuntimeHoldReason;
	type MaxNameLen = MaxNameLen;
	type MaxAssets = MaxAssets;
	type MaxGenres = MaxGenres;
	type WeightInfo = pallet_artists::weights::AllfeatWeight<Runtime>;
}
