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

use crate::{Runtime, genesis_token::tokenomics};
use polkadot_sdk::{frame_system, sp_io, sp_keyring::Sr25519Keyring, sp_runtime::BuildStorage};

pub mod token;

pub fn new_test_ext() -> sp_io::TestExternalities {
	let sudo = Sr25519Keyring::Charlie.to_account_id();
	let token_genesis = tokenomics(sudo, 0);

	let mut t = frame_system::GenesisConfig::<Runtime>::default().build_storage().unwrap();
	token_genesis.balances.assimilate_storage(&mut t).unwrap();
	token_genesis.allocations.assimilate_storage(&mut t).unwrap();

	sp_io::TestExternalities::new(t)
}
