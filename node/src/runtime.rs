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

//! Runtime dispatch utilities for multi-runtime support.
//!
//! This module provides a unified macro for dispatching operations to the
//! appropriate runtime based on chain specification, eliminating code
//! duplication across command handlers.

/// Error message when no runtime feature is enabled.
pub const NO_RUNTIME_ERR: &str = "No feature (melodie-runtime, allfeat-runtime) is enabled! \
    Compile with --features melodie-runtime or --features allfeat-runtime.";

/// Dispatches to the appropriate runtime based on chain spec identification.
///
/// This macro generates feature-gated branches for each supported runtime,
/// binding the `$RuntimeApi` identifier to the concrete runtime API type
/// within the provided expression body.
///
/// # Usage
///
/// ```ignore
/// dispatch_on_runtime!(chain_spec => |RuntimeApi| {
///     service::new_full::<RuntimeApi>(config)
/// })
/// ```
///
/// # Expansion
///
/// The macro expands to a series of `#[cfg(feature = "...-runtime")]` gated
/// if-blocks that check `chain_spec.is_melodie()` / `chain_spec.is_allfeat()`
/// and execute the body with the appropriate `RuntimeApi` type alias.
///
/// # Adding a New Runtime
///
/// To add support for a new runtime (e.g., `foo-runtime`):
/// 1. Add the feature and dependency in `Cargo.toml`
/// 2. Add a new `#[cfg(feature = "foo-runtime")]` block in this macro
/// 3. Add `is_foo()` method to `IdentifyVariant` trait
/// 4. Create the chain spec module in `chain_specs/`
#[macro_export]
macro_rules! dispatch_on_runtime {
    ($chain_spec:expr => |$RuntimeApi:ident| $body:expr) => {{
        use $crate::chain_specs::IdentifyVariant;

        #[cfg(feature = "melodie-runtime")]
        if $chain_spec.is_melodie() {
            type $RuntimeApi = $crate::service::MelodieRuntimeApi;
            return $body;
        }

        #[cfg(feature = "allfeat-runtime")]
        if $chain_spec.is_allfeat() {
            type $RuntimeApi = $crate::service::AllfeatRuntimeApi;
            return $body;
        }

        // If a single runtime feature is enabled, use it as a safe fallback for custom specs.
        #[cfg(all(feature = "melodie-runtime", not(feature = "allfeat-runtime")))]
        {
            type $RuntimeApi = $crate::service::MelodieRuntimeApi;
            return $body;
        }

        #[cfg(all(feature = "allfeat-runtime", not(feature = "melodie-runtime")))]
        {
            type $RuntimeApi = $crate::service::AllfeatRuntimeApi;
            return $body;
        }

        Err(sc_cli::Error::from($crate::runtime::NO_RUNTIME_ERR))
    }};
}

/// Variant of [`dispatch_on_runtime!`] for async runner contexts.
///
/// This macro is designed for use with `runner.async_run()` pattern where
/// we need to create partial components and return a tuple of (result, task_manager).
///
/// # Usage
///
/// ```ignore
/// dispatch_async_run!(runner, chain_spec, config => |components| {
///     Ok(cmd.run(components.client, components.import_queue))
/// })
/// ```
#[macro_export]
macro_rules! dispatch_async_run {
    ($runner:expr, $chain_spec:expr, $config:ident => |$components:ident| $body:expr) => {{
        use $crate::chain_specs::IdentifyVariant;

        #[cfg(feature = "melodie-runtime")]
        if $chain_spec.is_melodie() {
            return $runner.async_run(|$config| {
                let $components =
                    $crate::service::new_partial::<$crate::service::MelodieRuntimeApi>(&$config)
                        .map_err(|e| sc_cli::Error::from(*e))?;
                let task_manager = $components.task_manager;
                { $body }.map(|v| (v, task_manager))
            });
        }

        #[cfg(feature = "allfeat-runtime")]
        if $chain_spec.is_allfeat() {
            return $runner.async_run(|$config| {
                let $components =
                    $crate::service::new_partial::<$crate::service::AllfeatRuntimeApi>(&$config)
                        .map_err(|e| sc_cli::Error::from(*e))?;
                let task_manager = $components.task_manager;
                { $body }.map(|v| (v, task_manager))
            });
        }

        // If a single runtime feature is enabled, use it as a safe fallback for custom specs.
        #[cfg(all(feature = "melodie-runtime", not(feature = "allfeat-runtime")))]
        {
            return $runner.async_run(|$config| {
                let $components =
                    $crate::service::new_partial::<$crate::service::MelodieRuntimeApi>(&$config)
                        .map_err(|e| sc_cli::Error::from(*e))?;
                let task_manager = $components.task_manager;
                { $body }.map(|v| (v, task_manager))
            });
        }

        #[cfg(all(feature = "allfeat-runtime", not(feature = "melodie-runtime")))]
        {
            return $runner.async_run(|$config| {
                let $components =
                    $crate::service::new_partial::<$crate::service::AllfeatRuntimeApi>(&$config)
                        .map_err(|e| sc_cli::Error::from(*e))?;
                let task_manager = $components.task_manager;
                { $body }.map(|v| (v, task_manager))
            });
        }

        Err(sc_cli::Error::from($crate::runtime::NO_RUNTIME_ERR))
    }};
}

/// Variant of [`dispatch_on_runtime!`] for benchmark partial components.
///
/// This macro creates partial components for storage benchmarking operations.
///
/// # Usage
///
/// ```ignore
/// dispatch_benchmark_partials!(config => |partials| {
///     cmd.run(config, partials.client.clone(), db, storage)
/// })
/// ```
#[cfg(feature = "runtime-benchmarks")]
#[macro_export]
macro_rules! dispatch_benchmark_partials {
    ($config:expr => |$partials:ident| $body:expr) => {{
        use $crate::chain_specs::IdentifyVariant;

        #[cfg(feature = "melodie-runtime")]
        if $config.chain_spec.is_melodie() {
            let $partials =
                $crate::service::new_partial::<$crate::service::MelodieRuntimeApi>(&$config)
                    .map_err(|e| sc_cli::Error::from(*e))?;
            return $body;
        }

        #[cfg(feature = "allfeat-runtime")]
        if $config.chain_spec.is_allfeat() {
            let $partials =
                $crate::service::new_partial::<$crate::service::AllfeatRuntimeApi>(&$config)
                    .map_err(|e| sc_cli::Error::from(*e))?;
            return $body;
        }

        // If a single runtime feature is enabled, use it as a safe fallback for custom specs.
        #[cfg(all(feature = "melodie-runtime", not(feature = "allfeat-runtime")))]
        {
            let $partials =
                $crate::service::new_partial::<$crate::service::MelodieRuntimeApi>(&$config)
                    .map_err(|e| sc_cli::Error::from(*e))?;
            return $body;
        }

        #[cfg(all(feature = "allfeat-runtime", not(feature = "melodie-runtime")))]
        {
            let $partials =
                $crate::service::new_partial::<$crate::service::AllfeatRuntimeApi>(&$config)
                    .map_err(|e| sc_cli::Error::from(*e))?;
            return $body;
        }

        return Err(sc_cli::Error::from($crate::runtime::NO_RUNTIME_ERR));
    }};
}
