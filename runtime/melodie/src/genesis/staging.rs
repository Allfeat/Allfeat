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
use array_bytes::Dehexify;
use sp_application_crypto::Ss58Codec;
use sp_core::crypto::UncheckedInto;

pub fn staging_config_genesis() -> serde_json::Value {
    genesis(
        vec![
            (
                AccountId::from_ss58check("5FsEGrMXNXgiA1BLK79mPuncQBCXGAJ9F3kGNWK1eujFZBhe")
                    .unwrap(), //5FEkoaPqZnNbHb8H9NqSrBsELhf27Fpp3YEhxKqEAWbCxF7N
                //5FEkoaPqZnNbHb8H9NqSrBsELhf27Fpp3YEhxKqEAWbCxF7N
                <[u8; 32]>::dehexify(
                    "8c5f12e6c3725b4f71e8658c0ec5f9d4c0e424439bf279662f828a63561b38ec",
                )
                .unwrap()
                .unchecked_into(),
                //5CUonrGNpUrwERBJmHGo1GSHsVYsaB17AnW2SFoG1YLbcbEW
                <[u8; 32]>::dehexify(
                    "1261ddaa37ea3f4291bf8a531ff04f3b1316054e59a8f88237951e1227eae733",
                )
                .unwrap()
                .unchecked_into(),
            ),
            (
                AccountId::from_ss58check("5GZALNe7abDksJTWbysRLcs4yg4b3ZeBhzK3N322DsVp3qa3")
                    .unwrap(), //5HYAvJYxgxCC3mhDaQbnwTjsVLjVQGRV4cTHj6ypFQdMokjt
                //5HYAvJYxgxCC3mhDaQbnwTjsVLjVQGRV4cTHj6ypFQdMokjt
                <[u8; 32]>::dehexify(
                    "f21fe4ece38c66c00045ff8ffdc9f937fe71222b36785fd734f1fd3e97c16ed5",
                )
                .unwrap()
                .unchecked_into(),
                //5FRZGxnVAjYNT3xCDWRKLTbrdYwt1P2NoK9RvdYPTUXrQamy
                <[u8; 32]>::dehexify(
                    "949bf7979e6aa738a85141f90e7e9456babcce7ed933cba2e2107d501602ee6a",
                )
                .unwrap()
                .unchecked_into(),
            ),
        ],
        AccountId::from_ss58check("5EARX89jfEp9DjBitYW55CtSQ2xW2gJRvB69nLNwbHNf9TY8").unwrap(),
        vec![
            // Sudo account
            AccountId::from_ss58check("5EARX89jfEp9DjBitYW55CtSQ2xW2gJRvB69nLNwbHNf9TY8").unwrap(),
        ],
    )
}
