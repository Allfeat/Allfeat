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
				<[u8; 32]>::dehexify(
					"8c5f12e6c3725b4f71e8658c0ec5f9d4c0e424439bf279662f828a63561b38ec",
				)
				.unwrap()
				.unchecked_into(),
				//5DZiVsshwJPM2PpWitSPJ2oKFPZa8fWAMV3yUmm7SaKaE7yy
				<[u8; 32]>::dehexify(
					"425c8281892ec5210904a9288c3000a0d56c4f9aefeb489d13f2d64e4164e81b",
				)
				.unwrap()
				.unchecked_into(),
				//5Ek7XKVN6fX7KRrWKUfKjU4UpSegGUUYFzuDDYxnqLhkz1DH
				<[u8; 32]>::dehexify(
					"7686b4b332c4cca82dfce0174aabfb46c908f26ec7b3022993e0e8bcccc3b628",
				)
				.unwrap()
				.unchecked_into(),
				//5CQ85GydZ5SnxLUmVKPmMigxMSKszVhpnpvWKvdsJygp3Bku
				<[u8; 32]>::dehexify(
					"0ecf2f84d838c29a1ea9f79f1d68979e2e5548d48235c890449a68f0244d3d01",
				)
				.unwrap()
				.unchecked_into(),
			),
			(
				AccountId::from_ss58check("5GZALNe7abDksJTWbysRLcs4yg4b3ZeBhzK3N322DsVp3qa3")
					.unwrap(), //5HYAvJYxgxCC3mhDaQbnwTjsVLjVQGRV4cTHj6ypFQdMokjt
				<[u8; 32]>::dehexify(
					"f21fe4ece38c66c00045ff8ffdc9f937fe71222b36785fd734f1fd3e97c16ed5",
				)
				.unwrap()
				.unchecked_into(),
				//5CJyzogirY6DPaDT4KfegwbryBVMUBNx12NJQ3kGHGqbgGu5
				<[u8; 32]>::dehexify(
					"0ae3c4cb761c86f670fa69ef137e699052987a83eef367b529390ffe49956e32",
				)
				.unwrap()
				.unchecked_into(),
				//5E9hvtZe9c85dg9EPZGrEWuGuG9KY2robE4BQMHcwRHt4RsD
				<[u8; 32]>::dehexify(
					"5c48f05ce8fb089d088d7ff34a8bbde45950a71fa1968330ebf86ada14603e22",
				)
				.unwrap()
				.unchecked_into(),
				//5F44ZrMZZ2PTgCGkhBxNWQPjkY9QKBYowHQjpaQUQtW3iKFe
				<[u8; 32]>::dehexify(
					"84372bd3fe85bce586fa6990eb76b4300166200c83372c95b342548dc9322a15",
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
