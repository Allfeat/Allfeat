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
use alloc::vec;
use sp_keyring::{AccountKeyring, Ed25519Keyring, Sr25519Keyring};

/// Return the development genesis config.

pub fn local_config_genesis() -> serde_json::Value {
	genesis(
		vec![
			(
				AccountKeyring::Alice.to_account_id(),
				Ed25519Keyring::Alice.public().into(), // Grandpa
				Sr25519Keyring::Alice.public().into(), // Babe
				Sr25519Keyring::Alice.public().into(), // ImOnline
				Sr25519Keyring::Alice.public().into(), // AuthorityDiscovery
			),
			(
				AccountKeyring::Bob.to_account_id(),
				Ed25519Keyring::Bob.public().into(),
				Sr25519Keyring::Bob.public().into(),
				Sr25519Keyring::Bob.public().into(),
				Sr25519Keyring::Bob.public().into(),
			),
		],
		AccountKeyring::Alice.to_account_id(),
		vec![
			AccountKeyring::Alice.to_account_id(),
			AccountKeyring::Bob.to_account_id(),
			AccountKeyring::AliceStash.to_account_id(),
			AccountKeyring::BobStash.to_account_id(),
		],
	)
}
