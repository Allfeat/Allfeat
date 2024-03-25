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

// You should have received a copy of the GNU General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.
use super::command_helper::{
	inherent_benchmark_data, BenchmarkExtrinsicBuilder, TransferKeepAliveBuilder,
};
use crate::{
	chain_specs,
	chain_specs::{get_account_id_from_seed, DummyChainSpec},
	cli::{Cli, Subcommand},
	service,
	service::{new_partial, FullClient},
};
use allfeat_primitives::Hashing;
use frame_benchmarking_cli::*;
use harmonie_runtime::Block;
use sc_cli::{ChainSpec, Result, SubstrateCli};
use sc_service::PartialComponents;
use std::sync::Arc;

impl SubstrateCli for Cli {
	fn impl_name() -> String {
		"🎶 Allfeat Node".into()
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
		"https://github.com/allfeat/Allfeat/issues/new".into()
	}

	fn copyright_start_year() -> i32 {
		2023
	}

	fn load_spec(&self, id: &str) -> std::result::Result<Box<dyn ChainSpec>, String> {
		Ok(match id {
			#[cfg(feature = "allfeat-native")]
			"dev" | "development" | "allfeat-dev" =>
				Box::new(chain_specs::allfeat::development_chain_spec(None, None)),

			"" | "harmonie-live" | "testnet" => Box::new(DummyChainSpec::from_json_bytes(
				&include_bytes!("../genesis/harmonie-raw.json")[..],
			)?),
			#[cfg(feature = "harmonie-native")]
			"testnet-dev" | "testnet-development" | "harmonie-dev" =>
				Box::new(chain_specs::harmonie::development_chain_spec(None, None)),
			#[cfg(feature = "harmonie-native")]
			"harmonie-local" => Box::new(chain_specs::harmonie::get_chain_spec()),

			path => Box::new(chain_specs::DummyChainSpec::from_json_file(
				std::path::PathBuf::from(path),
			)?),
		})
	}
}

/// Parse command line arguments into service configuration.
pub fn run() -> Result<()> {
	let cli = Cli::from_args();

	match &cli.subcommand {
		None => {
			let runner = cli.create_runner(&cli.run)?;
			runner.run_node_until_exit(|config| async move {
				service::new_full(config, cli.eth, cli.no_hardware_benchmarks)
					.await
					.map_err(sc_cli::Error::Service)
			})
		},
		Some(Subcommand::Benchmark(cmd)) => {
			let runner = cli.create_runner(cmd)?;

			runner.sync_run(|config| {
				// This switch needs to be in the client, since the client decides
				// which sub-commands it wants to support.
				match cmd {
					BenchmarkCmd::Pallet(cmd) => {
						if !cfg!(feature = "runtime-benchmarks") {
							return Err(
								"Runtime benchmarking wasn't enabled when building the node. \
							You can enable it with `--features runtime-benchmarks`."
									.into(),
							);
						}

						cmd.run::<Hashing, sp_statement_store::runtime_api::HostFunctions>(config)
					},
					BenchmarkCmd::Block(cmd) => {
						// ensure that we keep the task manager alive
						let partial = new_partial(&config, &cli.eth)?;
						cmd.run(partial.client)
					},
					#[cfg(not(feature = "runtime-benchmarks"))]
					BenchmarkCmd::Storage(_) => Err(
						"Storage benchmarking can be enabled with `--features runtime-benchmarks`."
							.into(),
					),
					#[cfg(feature = "runtime-benchmarks")]
					BenchmarkCmd::Storage(cmd) => {
						let partial = new_partial(&config, &cli.eth)?;
						let db = partial.backend.expose_db();
						let storage = partial.backend.expose_storage();

						cmd.run(config, partial.client, db, storage)
					},
					BenchmarkCmd::Overhead(cmd) => {
						let partial = new_partial(&config, &cli.eth)?;
						let ext_builder = BenchmarkExtrinsicBuilder::new(partial.client.clone());
						let inherent_data = inherent_benchmark_data()?;

						cmd.run(config, partial.client, inherent_data, Vec::new(), &ext_builder)
					},
					BenchmarkCmd::Extrinsic(cmd) => {
						let PartialComponents { client, .. } =
							service::new_partial(&config, &cli.eth)?;
						// Register the *Remark* and *TKA* builders.
						let ext_factory = ExtrinsicFactory(vec![
							Box::new(BenchmarkExtrinsicBuilder::new(client.clone())),
							Box::new(TransferKeepAliveBuilder::new(
								client.clone(),
								get_account_id_from_seed::<sp_core::ecdsa::Public>("Alice"),
								harmonie_runtime::ExistentialDeposit::get(),
							)),
						]);

						cmd.run(client, inherent_benchmark_data()?, Vec::new(), &ext_factory)
					},
					BenchmarkCmd::Machine(cmd) =>
						cmd.run(&config, SUBSTRATE_REFERENCE_HARDWARE.clone()),
				}
			})
		},
		Some(Subcommand::Key(cmd)) => cmd.run(&cli),
		Some(Subcommand::BuildSpec(cmd)) => {
			let runner = cli.create_runner(cmd)?;
			runner.sync_run(|config| cmd.run(config.chain_spec, config.network))
		},
		Some(Subcommand::CheckBlock(cmd)) => {
			let runner = cli.create_runner(cmd)?;
			runner.async_run(|config| {
				let PartialComponents { client, task_manager, import_queue, .. } =
					new_partial(&config, &cli.eth)?;
				Ok((cmd.run(client, import_queue), task_manager))
			})
		},
		Some(Subcommand::ExportBlocks(cmd)) => {
			let runner = cli.create_runner(cmd)?;
			runner.async_run(|config| {
				let PartialComponents { client, task_manager, .. } =
					new_partial(&config, &cli.eth)?;
				Ok((cmd.run(client, config.database), task_manager))
			})
		},
		Some(Subcommand::ExportState(cmd)) => {
			let runner = cli.create_runner(cmd)?;
			runner.async_run(|config| {
				let PartialComponents { client, task_manager, .. } =
					new_partial(&config, &cli.eth)?;
				Ok((cmd.run(client, config.chain_spec), task_manager))
			})
		},
		Some(Subcommand::ImportBlocks(cmd)) => {
			let runner = cli.create_runner(cmd)?;
			runner.async_run(|config| {
				let PartialComponents { client, task_manager, import_queue, .. } =
					new_partial(&config, &cli.eth)?;
				Ok((cmd.run(client, import_queue), task_manager))
			})
		},
		Some(Subcommand::PurgeChain(cmd)) => {
			let runner = cli.create_runner(cmd)?;
			runner.sync_run(|config| cmd.run(config.database))
		},
		Some(Subcommand::Revert(cmd)) => {
			let runner = cli.create_runner(cmd)?;
			runner.async_run(|config| {
				let PartialComponents { client, task_manager, backend, .. } =
					new_partial(&config, &cli.eth)?;
				let aux_revert = Box::new(|client: Arc<FullClient>, backend, blocks| {
					sc_consensus_babe::revert(client.clone(), backend, blocks)?;
					grandpa::revert(client, blocks)?;
					Ok(())
				});
				Ok((cmd.run(client, backend, Some(aux_revert)), task_manager))
			})
		},
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
				let partial = new_partial(&mut config, &cli.eth)?;
				let frontier_backend = match partial.other.2 {
					fc_db::Backend::KeyValue(kv) => Arc::new(kv),
					_ => panic!("Only fc_db::Backend::KeyValue supported"),
				};
				cmd.run(partial.client, frontier_backend)
			})
		},
	}
}
