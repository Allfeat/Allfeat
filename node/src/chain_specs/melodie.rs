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

use super::{ChainSpec, build_chain_spec};
use melodie_runtime::WASM_BINARY;
use sc_service::ChainType;

const TOKEN_SYMBOL: &str = "MEL";

pub fn development_chain_spec() -> Result<ChainSpec, String> {
    build_chain_spec(WASM_BINARY, "Melodie Testnet V2 Development", "melodie_2_dev", ChainType::Development, TOKEN_SYMBOL, "development")
}

pub fn local_chain_spec() -> Result<ChainSpec, String> {
    build_chain_spec(WASM_BINARY, "Melodie Testnet V2 Local", "melodie_2_local", ChainType::Local, TOKEN_SYMBOL, "local_testnet")
}

pub fn live_chain_spec() -> Result<ChainSpec, String> {
    build_chain_spec(WASM_BINARY, "Melodie Testnet V2 Live", "melodie_2_staging", ChainType::Live, TOKEN_SYMBOL, "staging")
}
