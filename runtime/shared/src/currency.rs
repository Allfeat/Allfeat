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

use allfeat_primitives::Balance;

pub const MICROALFT: Balance = 1_000_000;
pub const MILLIALFT: Balance = 1_000_000_000;
pub const ALFT: Balance = 1_000_000_000_000;

pub const fn deposit(items: u32, bytes: u32) -> Balance {
	items as Balance * 10 * ALFT + (bytes as Balance) * 100 * MICROALFT
}
