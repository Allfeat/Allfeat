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
//!
//! This module provides chain specification builders for all supported networks.
//! Each network is defined via the [`define_chain_spec!`] macro, which generates
//! a module with `development_chain_spec()`, `local_chain_spec()`, and
//! `live_chain_spec()` functions.

use sc_service::{ChainType, Properties};

/// Specialized `ChainSpec`. This is a specialization of the general Substrate ChainSpec type.
pub type ChainSpec = sc_service::GenericChainSpec;

const WASM_BINARY_NOT_AVAILABLE: &str =
    "WASM binary not available. Build the runtime with `cargo build --release`.";
const TOKEN_DECIMALS: u32 = 12;

// ============================================================================
// Chain Spec Generation Macro
// ============================================================================

/// Generates a chain specification module for a runtime.
///
/// This macro creates a module containing:
/// - `network_config()` - Returns the network configuration
/// - `development_chain_spec()` - Creates a development chain spec
/// - `local_chain_spec()` - Creates a local testnet chain spec
/// - `live_chain_spec()` - Creates a live/staging chain spec
///
/// # Parameters
/// - `$feature`: The cargo feature name (e.g., `"allfeat-runtime"`)
/// - `$mod_name`: The module name to create (e.g., `mainnet`)
/// - `$runtime`: The runtime crate path (e.g., `allfeat_runtime`)
/// - `$symbol`: Token symbol (e.g., `"AFT"`)
/// - `$prefix`: Chain ID prefix (e.g., `"allfeat"`)
/// - `$name`: Human-readable chain name (e.g., `"Allfeat"`)
macro_rules! define_chain_spec {
    (
        feature = $feature:literal,
        module = $mod_name:ident,
        runtime = $runtime:ident,
        symbol = $symbol:literal,
        prefix = $prefix:literal,
        name = $name:literal
    ) => {
        #[cfg(feature = $feature)]
        pub mod $mod_name {
            //! Chain specifications for the network.

            use super::{build_chain_spec_for, ChainSpec, NetworkConfig};
            use sc_service::ChainType;

            /// Returns the network configuration for this runtime.
            pub fn network_config() -> NetworkConfig {
                NetworkConfig {
                    token_symbol: $symbol,
                    id_prefix: $prefix,
                    wasm_binary: $runtime::WASM_BINARY,
                    chain_name: $name,
                }
            }

            /// Creates a development chain spec (single validator, fast blocks).
            pub fn development_chain_spec() -> Result<ChainSpec, String> {
                build_chain_spec_for(&network_config(), ChainType::Development)
            }

            /// Creates a local testnet chain spec (multiple validators).
            pub fn local_chain_spec() -> Result<ChainSpec, String> {
                build_chain_spec_for(&network_config(), ChainType::Local)
            }

            /// Creates a live/staging chain spec.
            pub fn live_chain_spec() -> Result<ChainSpec, String> {
                build_chain_spec_for(&network_config(), ChainType::Live)
            }
        }
    };
}

// ============================================================================
// Network Definitions
// ============================================================================

define_chain_spec!(
    feature = "melodie-runtime",
    module = melodie,
    runtime = melodie_runtime,
    symbol = "MEL",
    prefix = "melodie_2",
    name = "Melodie Testnet V2"
);

define_chain_spec!(
    feature = "allfeat-runtime",
    module = mainnet,
    runtime = allfeat_runtime,
    symbol = "AFT",
    prefix = "allfeat",
    name = "Allfeat"
);

// Re-exports for convenience
#[cfg(feature = "allfeat-runtime")]
pub use mainnet as allfeat_chain_spec;
#[cfg(feature = "melodie-runtime")]
pub use melodie as melodie_chain_spec;

// Fallback type aliases when features are disabled (prevents compilation errors)
#[cfg(not(feature = "melodie-runtime"))]
#[allow(dead_code)]
pub type MelodieChainSpec = ChainSpec;
#[cfg(not(feature = "allfeat-runtime"))]
#[allow(dead_code)]
pub type AllfeatChainSpec = ChainSpec;

// ============================================================================
// Network Variant Identification
// ============================================================================

/// Identifies which network variant a chain spec belongs to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NetworkVariant {
    Melodie,
    Allfeat,
}

impl NetworkVariant {
    /// Try to determine the variant from a chain spec id string.
    ///
    /// Returns `Some(variant)` if the id starts with a known prefix,
    /// `None` otherwise.
    pub fn from_id(id: &str) -> Option<Self> {
        if id.starts_with("melodie") {
            Some(Self::Melodie)
        } else if id.starts_with("allfeat") {
            Some(Self::Allfeat)
        } else {
            None
        }
    }
}

/// Configuration parameters for a network variant's chain specs.
pub struct NetworkConfig {
    pub token_symbol: &'static str,
    pub id_prefix: &'static str,
    pub wasm_binary: Option<&'static [u8]>,
    pub chain_name: &'static str,
}

// ============================================================================
// Chain Spec Builder
// ============================================================================

fn token_properties(symbol: &str) -> Result<Properties, String> {
    serde_json::json!({
        "tokenDecimals": TOKEN_DECIMALS,
        "tokenSymbol": symbol,
    })
    .as_object()
    .cloned()
    .ok_or_else(|| "Failed to build token properties map".to_string())
}

/// Build a chain spec from a [`NetworkConfig`] and a [`ChainType`].
pub fn build_chain_spec_for(
    config: &NetworkConfig,
    chain_type: ChainType,
) -> Result<ChainSpec, String> {
    let wasm = config
        .wasm_binary
        .ok_or_else(|| WASM_BINARY_NOT_AVAILABLE.to_string())?;
    let properties = token_properties(config.token_symbol)?;

    let suffix = match chain_type {
        ChainType::Development => "dev",
        ChainType::Local => "local",
        ChainType::Live => "staging",
        ChainType::Custom(ref s) => return Err(format!("Unsupported chain type: {s}")),
    };
    let id = format!("{}_{suffix}", config.id_prefix);

    let preset = match chain_type {
        ChainType::Development => "development",
        ChainType::Local => "local_testnet",
        ChainType::Live => "staging",
        ChainType::Custom(_) => unreachable!(),
    };

    let name = match chain_type {
        ChainType::Development => format!("{} Development", config.chain_name),
        ChainType::Local => format!("{} Local", config.chain_name),
        ChainType::Live => format!("{} Live", config.chain_name),
        ChainType::Custom(_) => unreachable!(),
    };

    Ok(ChainSpec::builder(wasm, Default::default())
        .with_name(&name)
        .with_id(&id)
        .with_chain_type(chain_type)
        .with_properties(properties)
        .with_genesis_config_preset_name(preset)
        .build())
}

// ============================================================================
// IdentifyVariant Trait
// ============================================================================

/// Trait for identifying which network a chain specification belongs to.
///
/// This trait can be implemented for `Configuration` or `ChainSpec` types
/// to enable runtime dispatch based on the chain being used.
pub trait IdentifyVariant {
    /// Get the chain spec id string.
    fn id(&self) -> &str;

    /// Returns the [`NetworkVariant`] for this chain spec, if recognized.
    fn variant(&self) -> Option<NetworkVariant> {
        NetworkVariant::from_id(self.id())
    }

    /// Returns `true` if this is a configuration for the Melodie testnet.
    fn is_melodie(&self) -> bool {
        self.variant() == Some(NetworkVariant::Melodie)
    }

    /// Returns `true` if this is a configuration for the Allfeat mainnet.
    fn is_allfeat(&self) -> bool {
        self.variant() == Some(NetworkVariant::Allfeat)
    }
}

impl IdentifyVariant for Box<dyn sc_service::ChainSpec> {
    fn id(&self) -> &str {
        sc_service::ChainSpec::id(&**self)
    }
}
