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

use allfeat_primitives::{AccountId, Signature};
use sp_std::{vec, vec::Vec};
use sp_core::Public;
use sp_runtime::traits::{IdentifyAccount, Verify};
use sp_core::crypto::Pair;
use sp_runtime::format;

type AccountPublic = <Signature as Verify>::Signer;

/// Helper function to generate a crypto pair from seed
pub fn get_from_seed<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
	TPublic::Pair::from_string(&format!("//{}", seed), None)
		.expect("static values are valid; qed")
		.public()
}

/// Helper function to generate an account ID from seed
pub fn get_account_id_from_seed<TPublic: Public>(seed: &str) -> AccountId
where
	AccountPublic: From<<TPublic::Pair as Pair>::Public>,
{
	AccountPublic::from(get_from_seed::<TPublic>(seed)).into_account()
}

pub fn development_account() -> Vec<AccountId> {
    vec![
        // Alith
        array_bytes::hex_n_into_unchecked::<_, AccountId, 20>(
            "0xf24FF3a9CF04c71Dbc94D0b566f7A27B94566cac",
        ),
        // Baltathar
        array_bytes::hex_n_into_unchecked::<_, AccountId, 20>(
            "0x3Cd0A705a2DC65e5b1E1205896BaA2be8A07c6e0",
        ),
        // Charleth
        array_bytes::hex_n_into_unchecked::<_, AccountId, 20>(
            "0x798d4Ba9baf0064Ec19eB4F0a1a45785ae9D6DFc",
        ),
        // Dorothy
        array_bytes::hex_n_into_unchecked::<_, AccountId, 20>(
            "0x773539d4Ac0e786233D90A233654ccEE26a613D9",
        ),
        // Ethan
        array_bytes::hex_n_into_unchecked::<_, AccountId, 20>(
            "0xFf64d3F6efE2317EE2807d223a0Bdc4c0c49dfDB",
        ),
        // Faith
        array_bytes::hex_n_into_unchecked::<_, AccountId, 20>(
            "0xC0F0f4ab324C46e55D02D0033343B4Be8A55532d",
        ),
    ]
}