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

use parity_scale_codec::Encode;

/// Base definition of a MIDDS (Music Industry Decentralized Data Structure)
pub trait Midds<Identifier, AccountId>: Encode {
	fn depositor(self) -> AccountId;
	fn identifier(self) -> Identifier;
	fn total_bytes(&self) -> u32;
}

/// Base definition that a pallet storing and dealing with a MIDDS should implement.
pub trait MiddsRegistry<Identifier, AccountId, M>
where
	M: Midds<Identifier, AccountId>,
{
}
