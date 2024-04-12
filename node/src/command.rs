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

use std::{env, path::PathBuf, sync::Arc};
// You should have received a copy of the GNU General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.
use crate::{
	chain_specs::DummyChainSpec,
	cli::{Cli, Subcommand},
	client::Client,
	eth::db_config_dir,
	service,
};
use fc_db::kv::frontier_database_dir;
use harmonie_runtime::Block;
use sc_cli::{ChainSpec, SubstrateCli};
use sc_service::DatabaseSource;

#[cfg(feature = "runtime-benchmarks")]
use crate::chain_specs::get_account_id_from_seed;
use crate::{
	chain_specs::{
		allfeat_chain_spec, harmonie_chain_spec, AllfeatChainSpec, HarmonieChainSpec,
		IdentifyVariant,
	},
	client::{AllfeatRuntimeExecutor, HarmonieRuntimeExecutor},
};

impl SubstrateCli for Cli {
	fn impl_name() -> String {
		r"
    _     _  _   __               _
   / \   | || | / _|  ___   __ _ | |_
  / _ \  | || || |_  / _ \ / _` || __|
 / ___ \ | || ||  _||  __/| (_| || |_
/_/   \_\|_||_||_|   \___| \__,_| \__|

       ♪♫ Music Blockchain ♫♪
		"
		.into()
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

	fn load_spec(&self, id: &str) -> Result<Box<dyn ChainSpec>, String> {
		load_spec(id)
	}
}

/// Parse command line arguments into service configuration.
/// Parse and run command line arguments
pub fn run() -> sc_cli::Result<()> {
	let cli = Cli::from_args();

	match &cli.subcommand {
		Some(Subcommand::Key(cmd)) => cmd.run(&cli),
		Some(Subcommand::BuildSpec(cmd)) => {
			let runner = cli.create_runner(cmd)?;
			runner.sync_run(|config| cmd.run(config.chain_spec, config.network))
		},
		Some(Subcommand::CheckBlock(cmd)) => {
			let runner = cli.create_runner(cmd)?;
			runner.async_run(|mut config| {
				let (client, _, import_queue, task_manager, _) =
					service::new_chain_ops(&mut config, &cli.eth)?;
				Ok((cmd.run(client, import_queue), task_manager))
			})
		},
		Some(Subcommand::ExportBlocks(cmd)) => {
			let runner = cli.create_runner(cmd)?;
			runner.async_run(|mut config| {
				let (client, _, _, task_manager, _) =
					service::new_chain_ops(&mut config, &cli.eth)?;
				Ok((cmd.run(client, config.database), task_manager))
			})
		},
		Some(Subcommand::ExportState(cmd)) => {
			let runner = cli.create_runner(cmd)?;
			runner.async_run(|mut config| {
				let (client, _, _, task_manager, _) =
					service::new_chain_ops(&mut config, &cli.eth)?;
				Ok((cmd.run(client, config.chain_spec), task_manager))
			})
		},
		Some(Subcommand::ImportBlocks(cmd)) => {
			let runner = cli.create_runner(cmd)?;
			runner.async_run(|mut config| {
				let (client, _, import_queue, task_manager, _) =
					service::new_chain_ops(&mut config, &cli.eth)?;
				Ok((cmd.run(client, import_queue), task_manager))
			})
		},
		Some(Subcommand::PurgeChain(cmd)) => {
			let runner = cli.create_runner(cmd)?;
			runner.sync_run(|config| {
				// Remove Frontier offchain db
				let db_config_dir = db_config_dir(&config);
				match cli.eth.frontier_backend_type {
					crate::eth::FrontierBackendType::KeyValue => {
						let frontier_database_config = match config.database {
							DatabaseSource::RocksDb { .. } => DatabaseSource::RocksDb {
								path: frontier_database_dir(&db_config_dir, "db"),
								cache_size: 0,
							},
							DatabaseSource::ParityDb { .. } => DatabaseSource::ParityDb {
								path: frontier_database_dir(&db_config_dir, "paritydb"),
							},
							_ =>
								return Err(
									format!("Cannot purge `{:?}` database", config.database).into()
								),
						};
						cmd.run(frontier_database_config)?;
					},
					crate::eth::FrontierBackendType::Sql => {
						let db_path = db_config_dir.join("sql");
						match std::fs::remove_dir_all(&db_path) {
							Ok(_) => {
								println!("{:?} removed.", &db_path);
							},
							Err(ref err) if err.kind() == std::io::ErrorKind::NotFound => {
								eprintln!("{:?} did not exist.", &db_path);
							},
							Err(err) =>
								return Err(format!(
									"Cannot purge `{:?}` database: {:?}",
									db_path, err,
								)
								.into()),
						};
					},
				};
				cmd.run(config.database)
			})
		},
		Some(Subcommand::Revert(cmd)) => {
			let runner = cli.create_runner(cmd)?;
			runner.async_run(|mut config| {
				let (client, backend, _, task_manager, _) =
					service::new_chain_ops(&mut config, &cli.eth)?;
				let aux_revert = Box::new(move |client: Arc<Client>, backend, blocks| {
					sc_consensus_babe::revert(client.clone(), backend, blocks)?;
					grandpa::revert(client, blocks)?;
					Ok(())
				});
				Ok((cmd.run(client, backend, Some(aux_revert)), task_manager))
			})
		},
		#[cfg(feature = "runtime-benchmarks")]
		Some(Subcommand::Benchmark(cmd)) => {
			use crate::benchmarking::{
				inherent_benchmark_data, RemarkBuilder, TransferKeepAliveBuilder,
			};
			use allfeat_primitives::Hashing;
			use frame_benchmarking_cli::{
				BenchmarkCmd, ExtrinsicFactory, SUBSTRATE_REFERENCE_HARDWARE,
			};
			use harmonie_runtime::ExistentialDeposit;

			let runner = cli.create_runner(cmd)?;
			match cmd {
				BenchmarkCmd::Pallet(cmd) =>
					runner.sync_run(|config| cmd.run::<Hashing, ()>(config)),
				BenchmarkCmd::Block(cmd) => runner.sync_run(|mut config| {
					let (client, _, _, _, _) = service::new_chain_ops(&mut config, &cli.eth)?;
					cmd.run(client)
				}),
				BenchmarkCmd::Storage(cmd) => runner.sync_run(|mut config| {
					let (client, backend, _, _, _) = service::new_chain_ops(&mut config, &cli.eth)?;
					let db = backend.expose_db();
					let storage = backend.expose_storage();
					cmd.run(config, client, db, storage)
				}),
				BenchmarkCmd::Overhead(cmd) => runner.sync_run(|mut config| {
					let (client, _, _, _, _) = service::new_chain_ops(&mut config, &cli.eth)?;
					let ext_builder = RemarkBuilder::new(client.clone());
					cmd.run(config, client, inherent_benchmark_data()?, Vec::new(), &ext_builder)
				}),
				BenchmarkCmd::Extrinsic(cmd) => runner.sync_run(|mut config| {
					let (client, _, _, _, _) = service::new_chain_ops(&mut config, &cli.eth)?;
					// Register the *Remark* and *TKA* builders.
					let ext_factory = ExtrinsicFactory(vec![
						Box::new(RemarkBuilder::new(client.clone())),
						Box::new(TransferKeepAliveBuilder::new(
							client.clone(),
							get_account_id_from_seed::<sp_core::ecdsa::Public>("Alice"),
							ExistentialDeposit::get(),
						)),
					]);

					cmd.run(client, inherent_benchmark_data()?, Vec::new(), &ext_factory)
				}),
				BenchmarkCmd::Machine(cmd) =>
					runner.sync_run(|config| cmd.run(&config, SUBSTRATE_REFERENCE_HARDWARE.clone())),
			}
		},
		#[cfg(not(feature = "runtime-benchmarks"))]
		Some(Subcommand::Benchmark(_)) => Err("Benchmarking wasn't enabled when building the node. \
			You can enable it with `--features runtime-benchmarks`."
			.into()),
		#[cfg(feature = "try-runtime")]
		Some(Subcommand::TryRuntime(_)) => Err(try_runtime_cli::DEPRECATION_NOTICE.into()),
		#[cfg(not(feature = "try-runtime"))]
		Some(Subcommand::TryRuntime) => Err("TryRuntime wasn't enabled when building the node. \
				You can enable it with `--features try-runtime`."
			.into()),
		Some(Subcommand::ChainInfo(cmd)) => {
			let runner = cli.create_runner(cmd)?;
			runner.sync_run(|config| cmd.run::<Block>(&config))
		},
		Some(Subcommand::FrontierDb(cmd)) => {
			let runner = cli.create_runner(cmd)?;
			runner.sync_run(|mut config| {
				let (client, _, _, _, frontier_backend) =
					service::new_chain_ops(&mut config, &cli.eth)?;
				let frontier_backend = match frontier_backend {
					fc_db::Backend::KeyValue(kv) => std::sync::Arc::new(kv),
					_ => panic!("Only fc_db::Backend::KeyValue supported"),
				};
				cmd.run(client, frontier_backend)
			})
		},
		None => {
			let runner = cli.create_runner(&cli.run)?;

			runner.run_node_until_exit(|config| async move {
				let chain_spec = &config.chain_spec;

				#[cfg(feature = "allfeat-native")]
				if chain_spec.is_allfeat() {
					return service::new_full::<allfeat_runtime::RuntimeApi, AllfeatRuntimeExecutor>(
						config,
						cli.eth,
						cli.no_hardware_benchmarks,
						|_, _| (),
					)
						.await
						.map(|r| r)
						.map_err(Into::into);
				}

				#[cfg(feature = "harmonie-native")]
				if chain_spec.is_harmonie() {
					return service::new_full::<harmonie_runtime::RuntimeApi, HarmonieRuntimeExecutor>(
						config,
						cli.eth,
						cli.no_hardware_benchmarks,
						|_, _| (),
					)
						.await
						.map(|r| r)
						.map_err(Into::into);
				}

				panic!("No feature(harmonie-native, allfeat-native) is enabled!");
			})
		},
	}
}

fn load_spec(id: &str) -> Result<Box<dyn ChainSpec>, String> {
	let id = if id.is_empty() { "harmonie" } else { id };

	Ok(match id.to_lowercase().as_str() {
		#[cfg(feature = "allfeat-native")]
		"allfeat-dev" | "dev" => Box::new(allfeat_chain_spec::development_chain_spec(None, None)),
		#[cfg(feature = "harmonie-native")]
		"harmonie" => Box::new(HarmonieChainSpec::from_json_bytes(
			&include_bytes!("../genesis/harmonie-raw.json")[..],
		)?),
		#[cfg(feature = "harmonie-native")]
		"harmonie-dev" => Box::new(harmonie_chain_spec::development_chain_spec(None, None)),
		#[cfg(feature = "harmonie-native")]
		"harmonie-local" => Box::new(harmonie_chain_spec::get_chain_spec()),
		_ => {
			let path = PathBuf::from(id);
			let chain_spec =
				Box::new(DummyChainSpec::from_json_file(path.clone())?) as Box<dyn ChainSpec>;

			if chain_spec.is_harmonie() {
				return Ok(Box::new(HarmonieChainSpec::from_json_file(path)?));
			}

			if chain_spec.is_allfeat() {
				return Ok(Box::new(AllfeatChainSpec::from_json_file(path)?));
			}

			panic!("No feature(allfeat-native, harmonie-native) is enabled!")
		},
	})
}
