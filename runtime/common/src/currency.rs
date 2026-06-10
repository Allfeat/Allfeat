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

//! Monetary constants, identical by design on every Allfeat network
//! (12 decimals; `UNIT` is one AFT on mainnet, one MEL on Melodie).

use crate::Balance;

pub const UNIT: Balance = 1_000_000_000_000;
pub const CENTIUNIT: Balance = 10_000_000_000;
pub const MILLIUNIT: Balance = 1_000_000_000;
pub const MICROUNIT: Balance = 1_000_000;

/// 0.1 UNIT, aligned across the live networks.
pub const EXISTENTIAL_DEPOSIT: Balance = UNIT / 10;

/// Price of storage: deposit charged per item and per byte for on-chain
/// state created on behalf of an account.
pub const fn deposit(items: u32, bytes: u32) -> Balance {
	items as Balance * 10 * UNIT + (bytes as Balance) * 100 * MICROUNIT
}
