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

use std::{env, path::PathBuf};
// You should have received a copy of the GNU General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.
use crate::{
	chain_specs::{melodie_chain_spec, ChainSpec},
	cli::{Cli, Subcommand},
	service::{self, *},
};
use sc_cli::{ChainSpec as ChainSpecT, SubstrateCli};
use sp_core::crypto::Ss58AddressFormatRegistry;

impl SubstrateCli for Cli {
	fn impl_name() -> String {
		"Allfeat Node".into()
	}

	fn impl_version() -> String {
		env!("SUBSTRATE_CLI_IMPL_VERSION").into()
	}

	fn description() -> String {
		env!("CARGO_PKG_DESCRIPTION").into()
	}

	fn author() -> String {
		env!("CARGO_PKG_AUTHORS").into()
	}

	fn support_url() -> String {
		"https://github.com/allfeat/allfeat/issues/new".into()
	}

	fn copyright_start_year() -> i32 {
		2022
	}

	fn load_spec(&self, id: &str) -> std::result::Result<Box<dyn ChainSpecT>, String> {
		load_spec(id)
	}
}

/// Parse command line arguments into service configuration.
/// Parse and run command line arguments
pub fn run() -> sc_cli::Result<()> {
	#[cfg(feature = "runtime-benchmarks")]
	/// Creates partial components for the runtimes that are supported by the benchmarks.
	macro_rules! construct_benchmark_partials {
		($config:expr, $cli:ident, |$partials:ident| $code:expr) => {{
			#[cfg(feature = "melodie-runtime")]
			if $config.chain_spec.is_melodie() {
				let $partials = service::new_partial::<MelodieRuntimeApi>(&$config)?;

				return $code;
			}

			panic!("No feature(melodie-runtime) is enabled!");
		}};
	}

	macro_rules! construct_async_run {
		(|$components:ident, $cli:ident, $cmd:ident, $config:ident| $( $code:tt )* ) => {{
			let runner = $cli.create_runner($cmd)?;
			let chain_spec = &runner.config().chain_spec;

			set_default_ss58_version(chain_spec);

			#[cfg(feature = "melodie-runtime")]
			if chain_spec.is_melodie() {
				return runner.async_run(|$config| {
					let $components = service::new_partial::<MelodieRuntimeApi>(
						&$config,
					)?;
					let task_manager = $components.task_manager;

					{ $( $code )* }.map(|v| (v, task_manager))
				});
			}

			panic!("No feature(melodie-runtime) is enabled!");
		}}
	}

	let cli = Cli::from_args();

	match &cli.subcommand {
		Some(Subcommand::Key(cmd)) => cmd.run(&cli),
		Some(Subcommand::BuildSpec(cmd)) => {
			let runner = cli.create_runner(cmd)?;
			let chain_spec = &runner.config().chain_spec;

			set_default_ss58_version(chain_spec);
			runner.sync_run(|config| cmd.run(config.chain_spec, config.network))
		},
		Some(Subcommand::CheckBlock(cmd)) => {
			construct_async_run!(|components, cli, cmd, config| {
				Ok(cmd.run(components.client, components.import_queue))
			})
		},
		Some(Subcommand::ExportBlocks(cmd)) => {
			construct_async_run!(|components, cli, cmd, config| {
				Ok(cmd.run(components.client, config.database))
			})
		},
		Some(Subcommand::ExportState(cmd)) => {
			construct_async_run!(|components, cli, cmd, config| {
				Ok(cmd.run(components.client, config.chain_spec))
			})
		},
		Some(Subcommand::ImportBlocks(cmd)) => {
			construct_async_run!(|components, cli, cmd, config| {
				Ok(cmd.run(components.client, components.import_queue))
			})
		},
		Some(Subcommand::Revert(cmd)) => {
			construct_async_run!(|components, cli, cmd, config| {
				Ok(cmd.run(components.client, components.backend, None))
			})
		},
		Some(Subcommand::PurgeChain(cmd)) => {
			let runner = cli.create_runner(cmd)?;
			let chain_spec = &runner.config().chain_spec;

			set_default_ss58_version(chain_spec);
			runner.sync_run(|config| {
				cmd.run(config.database)
			})
		},
		#[cfg(feature = "runtime-benchmarks")]
		Some(Subcommand::Benchmark(cmd)) => {
			// polkadot-sdk
			use frame_benchmarking_cli::{BenchmarkCmd, SUBSTRATE_REFERENCE_HARDWARE};

			let runner = cli.create_runner(cmd)?;

			set_default_ss58_version(&runner.config().chain_spec);

			match cmd {
				BenchmarkCmd::Pallet(_) =>
					Err("Pallet benchmarking has migrated to his own CLI tool, please read https://github.com/paritytech/polkadot-sdk/pull/3512.".into()),
				BenchmarkCmd::Storage(cmd) => runner.sync_run(|config| {
					construct_benchmark_partials!(config, cli, |partials| {
						let db = partials.backend.expose_db();
						let storage = partials.backend.expose_storage();

						cmd.run(config, partials.client.clone(), db, storage)
					})
				}),
				BenchmarkCmd::Overhead(_) => Err("Unsupported benchmarking command".into()),
				BenchmarkCmd::Extrinsic(_) => Err("Unsupported benchmarking command".into()),
				BenchmarkCmd::Block(cmd) => runner.sync_run(|config| {
					construct_benchmark_partials!(config, cli, |partials| cmd.run(partials.client))
				}),
				BenchmarkCmd::Machine(cmd) =>
					runner.sync_run(|config| cmd.run(&config, SUBSTRATE_REFERENCE_HARDWARE.clone())),
			}
		},
		#[cfg(not(feature = "runtime-benchmarks"))]
		Some(Subcommand::Benchmark(_)) => Err(
			"Benchmarking was not enabled when building the node. You can enable it with `--features runtime-benchmarks`.".into()
		),
		None => {
			let runner = cli.create_runner(&cli.run)?;

			runner.run_node_until_exit(|config| async move {
				let chain_spec = &config.chain_spec;

				set_default_ss58_version(chain_spec);

				log::info!(
					"Is validating: {}",
					if config.role.is_authority() { "yes" } else { "no" }
				);

				#[cfg(feature = "melodie-runtime")]
				if chain_spec.is_melodie() {
					return service::new_full_from_network_cfg::<MelodieRuntimeApi>(
						config,
					)
					.map_err(Into::into);
				}

				panic!("No feature(melodie-runtime) is enabled!");
			})
		},
	}
}

fn load_spec(id: &str) -> std::result::Result<Box<dyn ChainSpecT>, String> {
	let id = if id.is_empty() {
		let n = get_exec_name().unwrap_or_default();
		["melodie"]
			.iter()
			.cloned()
			.find(|&chain| n.starts_with(chain))
			.unwrap_or("melodie")
	} else {
		id
	};
	let chain_spec = match id.to_lowercase().as_str() {
		#[cfg(feature = "melodie-runtime")]
		"" | "melodie" =>
			Box::new(ChainSpec::from_json_bytes(&include_bytes!("../specs/melodie_raw.json")[..])?),
		#[cfg(feature = "melodie-runtime")]
		"melodie-staging" => Box::new(melodie_chain_spec::live_chain_spec().unwrap()),
		#[cfg(feature = "melodie-runtime")]
		"melodie-local" => Box::new(melodie_chain_spec::local_chain_spec().unwrap()),
		#[cfg(feature = "melodie-runtime")]
		"dev" | "melodie-dev" => Box::new(melodie_chain_spec::development_chain_spec().unwrap()),
		_ => Box::new(ChainSpec::from_json_file(PathBuf::from(id))?),
	};

	Ok(chain_spec)
}

fn get_exec_name() -> Option<String> {
	env::current_exe()
		.ok()
		.and_then(|pb| pb.file_name().map(|s| s.to_os_string()))
		.and_then(|s| s.into_string().ok())
}

fn set_default_ss58_version(chain_spec: &dyn IdentifyVariant) {
	let ss58_version = if chain_spec.is_melodie() {
		Ss58AddressFormatRegistry::SubstrateAccount
	} else {
		Ss58AddressFormatRegistry::AllfeatNetworkAccount
	}
	.into();

	sp_core::crypto::set_default_ss58_version(ss58_version);
}
