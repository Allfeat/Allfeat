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

#[macro_export]
macro_rules! declare_subtype {
    ($enum_name:ident { $($variant:ident),* $(,)? }) => {
        #[derive(
            Clone, Copy, PartialEq, Eq, PartialOrd, Ord,
            RuntimeDebug, Encode, Decode, TypeInfo, MaxEncodedLen
        )]
        pub enum $enum_name {
            $($variant),*
        }
    }
}

#[macro_export]
macro_rules! declare_music_genre {
    ($($genre:ident($subtype:ident { $($variant:ident),* $(,)? }),)*) => {
        $(declare_subtype!($subtype { $($variant),* });)*

        #[derive(
            Clone, Copy, PartialEq, Eq, PartialOrd, Ord,
            RuntimeDebug, Encode, Decode, TypeInfo, MaxEncodedLen
        )]
        pub enum MusicGenre {
            $($genre(Option<$subtype>)),*
        }
    }
}
