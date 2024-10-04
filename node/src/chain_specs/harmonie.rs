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

use super::ChainSpec;
use harmonie_runtime::WASM_BINARY;
use sc_chain_spec::ChainType;

/// Generate a chain spec for use with the development service.
pub fn development_chain_spec() -> Result<ChainSpec, String> {
	Ok(ChainSpec::builder(
		WASM_BINARY.ok_or_else(|| "Development wasm not available".to_string())?,
		Default::default(),
	)
	.with_name("Harmonie Testnet Development")
	.with_id("harmonie_live")
	.with_chain_type(ChainType::Development)
	.with_protocol_id("aft")
	.with_properties(
		serde_json::json!({
			"isEthereum": true,
			"tokenDecimals": 18,
			"tokenSymbol": "HMY",
		})
		.as_object()
		.expect("Map given; qed")
		.clone(),
	)
	.with_genesis_config_preset_name("development")
	.build())
}

/// Generate a default spec for the parachain service. Use this as a starting point when launching
/// a custom chain.
pub fn local_chain_spec() -> Result<ChainSpec, String> {
	Ok(ChainSpec::builder(
		WASM_BINARY.ok_or_else(|| "Development wasm not available".to_string())?,
		Default::default(),
	)
	.with_name("Harmonie Testnet Local")
	.with_id("harmonie_live")
	.with_chain_type(ChainType::Local)
	.with_protocol_id("aft")
	.with_properties(
		serde_json::json!({
			"isEthereum": true,
			"tokenDecimals": 18,
			"tokenSymbol": "HMY",
		})
		.as_object()
		.expect("Map given; qed")
		.clone(),
	)
	.with_genesis_config_preset_name("local_testnet")
	.build())
}
