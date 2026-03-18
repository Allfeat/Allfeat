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
                AccountId::from_ss58check("5HB52SvFJEgXj4TnQyv9ghMvQZLzDsZKJPZY2AhEYdUyW8Ns")
                    .unwrap(), //5FEkoaPqZnNbHb8H9NqSrBsELhf27Fpp3YEhxKqEAWbCxF7N
                //5FEkoaPqZnNbHb8H9NqSrBsELhf27Fpp3YEhxKqEAWbCxF7N
                <[u8; 32]>::dehexify(
                    "527d4a33043535d6f2e03117b9d6f66e60ca84962c740799b4ce4e4bdb52c101",
                )
                .unwrap()
                .unchecked_into(),
                //5CUonrGNpUrwERBJmHGo1GSHsVYsaB17AnW2SFoG1YLbcbEW
                <[u8; 32]>::dehexify(
                    "f6353bfc949d1d268740e70697514d327b454a2b44976d6b2d19925ceb7c1004",
                )
                .unwrap()
                .unchecked_into(),
            ),
            (
                AccountId::from_ss58check("5CVU8MfV5P6oKXW4Hx5j2i7FBwX9RtDhXfkRTSq4uPrDZsuj")
                    .unwrap(), //5HYAvJYxgxCC3mhDaQbnwTjsVLjVQGRV4cTHj6ypFQdMokjt
                //5HYAvJYxgxCC3mhDaQbnwTjsVLjVQGRV4cTHj6ypFQdMokjt
                <[u8; 32]>::dehexify(
                    "6285743d7f481e338cd52f888f4926caf28ba49bd8e21217358baf2292ddeac9",
                )
                .unwrap()
                .unchecked_into(),
                //5FRZGxnVAjYNT3xCDWRKLTbrdYwt1P2NoK9RvdYPTUXrQamy
                <[u8; 32]>::dehexify(
                    "e4af3da951c73adce00a8258dc8676ce6d30fce78561f82e75ebd6e8cdbc1730",
                )
                .unwrap()
                .unchecked_into(),
            ),
        ],
        AccountId::from_ss58check("5HDq69cbUxRMHwCDzpFefSeBaLAQmnLKp795zcWNmgGqAix6").unwrap(),
        vec![
            // Sudo account
            AccountId::from_ss58check("5HDq69cbUxRMHwCDzpFefSeBaLAQmnLKp795zcWNmgGqAix6").unwrap(),
        ],
    )
}
