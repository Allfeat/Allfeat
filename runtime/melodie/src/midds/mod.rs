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

pub mod musical_works;
pub mod party_identifier;
pub mod release;
pub mod track;

pub type PartyIdentifiers = pallet_midds::Instance1;
pub type MusicalWorks = pallet_midds::Instance2;
pub type Tracks = pallet_midds::Instance3;
pub type Releases = pallet_midds::Instance4;
