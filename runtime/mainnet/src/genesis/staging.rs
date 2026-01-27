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
            // Validator 1
            (
                AccountId::from_ss58check("qSuo1LcUoi7JFNQbar8r8N7JN9JSMggsFEUKcPWc6sEyPjiFa")
                    .unwrap(),
                <[u8; 32]>::dehexify(
                    "ae2b4a12c4f94171a897c1546f85b64adc1883409ef6ad2c6bcb3c0668e3896c",
                )
                .unwrap()
                .unchecked_into(),
                <[u8; 32]>::dehexify(
                    "2edd6141c37e37a90b7bb8398346d6689e7ccda12c6b8bf9bba124549a7e626f",
                )
                .unwrap()
                .unchecked_into(),
            ),
            // Validator 2
            (
                AccountId::from_ss58check("qSv3xY3t1rFkhxvpSBdqChhwjTCqJ1qJjNoq5ZKFo8vvTgms4")
                    .unwrap(),
                <[u8; 32]>::dehexify(
                    "211155996ba312dd1dcba2080934c38178d6dea4bc1f4d63a0a2445f81b75cc1",
                )
                .unwrap()
                .unchecked_into(),
                <[u8; 32]>::dehexify(
                    "e050792174140b0d17097c7cf837ab6e07a79f9a8c3682574bccf30ffe7c1b2f",
                )
                .unwrap()
                .unchecked_into(),
            ),
        ],
        vec![],
        AccountId::from_ss58check("qSysBTZC3yQRKNroife4djUTQwnfVxHQ19PpxgHKcRFJszHRA").unwrap(),
    )
}
