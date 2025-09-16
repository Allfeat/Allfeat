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

use super::genesis;
use allfeat_primitives::AccountId;
use alloc::vec;
use sp_application_crypto::Ss58Codec;

pub fn staging_config_genesis() -> serde_json::Value {
    genesis(
        vec![],
        AccountId::from_ss58check("5EARX89jfEp9DjBitYW55CtSQ2xW2gJRvB69nLNwbHNf9TY8").unwrap(),
        vec![
            // Sudo account
            AccountId::from_ss58check("5EARX89jfEp9DjBitYW55CtSQ2xW2gJRvB69nLNwbHNf9TY8").unwrap(),
        ],
    )
}
