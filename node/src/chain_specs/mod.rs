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

//! Substrate chain configurations.

#[cfg(feature = "allfeat-runtime")]
pub mod mainnet;
#[cfg(feature = "melodie-runtime")]
pub mod melodie;
#[cfg(feature = "allfeat-runtime")]
pub use mainnet::{self as allfeat_chain_spec};
#[cfg(feature = "melodie-runtime")]
pub use melodie::{self as melodie_chain_spec};

#[cfg(not(feature = "melodie-runtime"))]
pub type MelodieChainSpec = ChainSpec;

#[cfg(not(feature = "allfeat-runtime"))]
pub type AllfeatChainSpec = ChainSpec;

/// Specialized `ChainSpec`. This is a specialization of the general Substrate ChainSpec type.
pub type ChainSpec = sc_service::GenericChainSpec;

use sc_service::{ChainType, Properties};

const WASM_NOT_AVAILABLE: &str = "Development wasm not available";
const TOKEN_DECIMALS: u32 = 12;

fn token_properties(symbol: &str) -> Properties {
    serde_json::json!({
        "tokenDecimals": TOKEN_DECIMALS,
        "tokenSymbol": symbol,
    })
    .as_object()
    .expect("Map given; qed")
    .clone()
}

fn build_chain_spec(
    wasm_binary: Option<&[u8]>,
    name: &str,
    id: &str,
    chain_type: ChainType,
    token_symbol: &str,
    genesis_preset: &str,
) -> Result<ChainSpec, String> {
    Ok(ChainSpec::builder(
        wasm_binary.ok_or_else(|| WASM_NOT_AVAILABLE.to_string())?,
        Default::default(),
    )
    .with_name(name)
    .with_id(id)
    .with_chain_type(chain_type)
    .with_properties(token_properties(token_symbol))
    .with_genesis_config_preset_name(genesis_preset)
    .build())
}

/// Can be called for a `Configuration` to check if it is the specific network.
pub trait IdentifyVariant {
    /// Get spec id.
    fn id(&self) -> &str;

    /// Returns if this is a configuration for the `Melodie` network.
    fn is_melodie(&self) -> bool {
        self.id().starts_with("melodie")
    }

    /// Returns if this is a configuration for the `Allfeat` network.
    fn is_allfeat(&self) -> bool {
        self.id().starts_with("allfeat")
    }
}
impl IdentifyVariant for Box<dyn sc_service::ChainSpec> {
    fn id(&self) -> &str {
        sc_service::ChainSpec::id(&**self)
    }
}
