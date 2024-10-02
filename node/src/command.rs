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

use std::{env, path::PathBuf};
// You should have received a copy of the GNU General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.
use crate::{
	chain_specs::ChainSpec,
	cli::{Cli, FrontierBackendType, Subcommand},
	service::{self, *},
};
use sc_cli::{ChainSpec as ChainSpecT, SubstrateCli};
use sc_service::DatabaseSource;
use sp_core::crypto::Ss58AddressFormatRegistry;

use crate::chain_specs::harmonie_chain_spec;

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
	macro_rules! construct_async_run {
		(|$components:ident, $cli:ident, $cmd:ident, $config:ident| $( $code:tt )* ) => {{
			let runner = $cli.create_runner($cmd)?;
			let chain_spec = &runner.config().chain_spec;

			set_default_ss58_version(chain_spec);

			#[cfg(feature = "harmonie-runtime")]
			if chain_spec.is_harmonie() {
				return runner.async_run(|$config| {
					let $components = service::new_partial::<HarmonieRuntimeApi>(
						&$config,
						&$cli.eth.build_eth_rpc_config()
					)?;
					let task_manager = $components.task_manager;

					{ $( $code )* }.map(|v| (v, task_manager))
				});
			}

			panic!("No feature(harmonie-runtime) is enabled!");
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
				// Remove Frontier off-chain db
				let db_config_dir = frontier::db_config_dir(&config);

				match cli.eth.frontier_backend_type {
					FrontierBackendType::KeyValue => {
						let frontier_database_config = match config.database {
							DatabaseSource::RocksDb { .. } => DatabaseSource::RocksDb {
								path: fc_db::kv::frontier_database_dir(&db_config_dir, "db"),
								cache_size: 0,
							},
							DatabaseSource::ParityDb { .. } => DatabaseSource::ParityDb {
								path: fc_db::kv::frontier_database_dir(&db_config_dir, "paritydb"),
							},
							_ => {
								return Err(format!(
									"Cannot purge `{:?}` database",
									config.database
								)
								.into())
							}
						};

						cmd.run(frontier_database_config)?;
					}
					FrontierBackendType::Sql => {
						let db_path = db_config_dir.join("sql");

						match std::fs::remove_dir_all(&db_path) {
							Ok(_) => {
								println!("{:?} removed.", &db_path);
							}
							Err(ref err) if err.kind() == std::io::ErrorKind::NotFound => {
								eprintln!("{:?} did not exist.", &db_path);
							}
							Err(err) => {
								return Err(format!(
									"Cannot purge `{:?}` database: {:?}",
									db_path, err,
								)
								.into())
							}
						};
					}
				};

				cmd.run(config.database)
			})
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

				let no_hardware_benchmarks = cli.no_hardware_benchmarks;
				let storage_monitor = cli.storage_monitor;
				let eth_rpc_config = cli.eth.build_eth_rpc_config();

				log::info!(
					"Is validating: {}",
					if config.role.is_authority() { "yes" } else { "no" }
				);

				#[cfg(feature = "harmonie-runtime")]
				if chain_spec.is_harmonie() {
					return service::start_node::<HarmonieRuntimeApi>(
						config,
						no_hardware_benchmarks,
						storage_monitor,
						&eth_rpc_config,
					)
					.await
					.map(|r| r.0)
					.map_err(Into::into);
				}

				panic!("No feature(harmonie-runtime) is enabled!");
			})
		},
	}
}

fn load_spec(id: &str) -> std::result::Result<Box<dyn ChainSpecT>, String> {
	let id = if id.is_empty() {
		let n = get_exec_name().unwrap_or_default();
		["harmonie"]
			.iter()
			.cloned()
			.find(|&chain| n.starts_with(chain))
			.unwrap_or("harmonie")
	} else {
		id
	};
	let chain_spec = match id.to_lowercase().as_str() {
		#[cfg(feature = "harmonie-runtime")]
		"harmonie" => Box::new(ChainSpec::from_json_bytes(
			&include_bytes!("../genesis/harmonie-raw.json")[..],
		)?),
		#[cfg(feature = "harmonie-runtime")]
		"harmonie-local" => Box::new(harmonie_chain_spec::get_chain_spec()),
		#[cfg(feature = "harmonie-runtime")]
		"dev" | "harmonie-dev" => Box::new(harmonie_chain_spec::development_chain_spec(None, None)),
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
	let ss58_version = if chain_spec.is_harmonie() {
		Ss58AddressFormatRegistry::SubstrateAccount
	} else {
		Ss58AddressFormatRegistry::AllfeatNetworkAccount
	}
	.into();

	sp_core::crypto::set_default_ss58_version(ss58_version);
}