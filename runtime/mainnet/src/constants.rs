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

/// Time.
pub mod time {
    use allfeat_primitives::{BlockNumber, Moment};

    #[cfg(feature = "fast-runtime")]
    pub const MILLISECS_PER_BLOCK: Moment = 3000;
    #[cfg(not(feature = "fast-runtime"))]
    pub const MILLISECS_PER_BLOCK: Moment = 12000;

    pub const SECS_PER_BLOCK: Moment = MILLISECS_PER_BLOCK / 1000;

    // NOTE: Currently it is not possible to change the slot duration after the chain has started.
    //       Attempting to do so will brick block production.
    pub const SLOT_DURATION: Moment = MILLISECS_PER_BLOCK;

    // These time units are defined in number of blocks.
    pub const MINUTES: BlockNumber = 60 / (SECS_PER_BLOCK as BlockNumber);
    pub const HOURS: BlockNumber = MINUTES * 60;
    pub const DAYS: BlockNumber = HOURS * 24;
    pub const WEEKS: BlockNumber = DAYS * 7;
}
