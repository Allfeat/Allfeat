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

use frame_support::traits::{Time, fungible::Inspect};
use parity_scale_codec::{Decode, Encode, MaxEncodedLen};

use crate::Config;

pub type BalanceOf<T> =
    <<T as Config>::Currency as Inspect<<T as frame_system::Config>::AccountId>>::Balance;
pub type MomentOf<T> = <<T as Config>::Timestamp as Time>::Moment;
pub type AccountIdOf<T> = <T as frame_system::Config>::AccountId;

/// Basic informations on the MIDDS entity.
#[derive(Clone, Encode, Decode, scale_info::TypeInfo, MaxEncodedLen)]
#[codec(mel_bound())]
#[scale_info(skip_type_params(T, I))]
pub struct MiddsInfo<T: Config> {
    pub(crate) provider: AccountIdOf<T>,
    pub(crate) registered_at: MomentOf<T>,
    pub(crate) hash: [u8; 32],
    pub(crate) encoded_size: u32,
    pub(crate) data_cost: BalanceOf<T>,
}
