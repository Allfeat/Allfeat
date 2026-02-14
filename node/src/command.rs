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

use crate::{
    chain_specs::{ChainSpec, IdentifyVariant},
    cli::{Cli, Subcommand},
    service,
};

#[cfg(feature = "allfeat-runtime")]
use crate::chain_specs::allfeat_chain_spec;
#[cfg(feature = "melodie-runtime")]
use crate::chain_specs::melodie_chain_spec;
use sc_cli::{ChainSpec as ChainSpecT, SubstrateCli};
use sc_storage_monitor::StorageMonitorService;
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
#[allow(clippy::result_large_err)]
pub fn run() -> sc_cli::Result<()> {
    let cli = Cli::from_args();

    match &cli.subcommand {
        Some(Subcommand::Key(cmd)) => cmd.run(&cli),
        Some(Subcommand::BuildSpec(cmd)) => {
            let runner = cli.create_runner(cmd)?;
            set_default_ss58_version(&runner.config().chain_spec);
            runner.sync_run(|config| cmd.run(config.chain_spec, config.network))
        }
        Some(Subcommand::CheckBlock(cmd)) => {
            let runner = cli.create_runner(cmd)?;
            set_default_ss58_version(&runner.config().chain_spec);
            dispatch_async_run!(runner, &runner.config().chain_spec, config => |components| {
                Ok(cmd.run(components.client, components.import_queue))
            })
        }
        Some(Subcommand::ExportBlocks(cmd)) => {
            let runner = cli.create_runner(cmd)?;
            set_default_ss58_version(&runner.config().chain_spec);
            dispatch_async_run!(runner, &runner.config().chain_spec, config => |components| {
                Ok(cmd.run(components.client, config.database))
            })
        }
        Some(Subcommand::ExportState(cmd)) => {
            let runner = cli.create_runner(cmd)?;
            set_default_ss58_version(&runner.config().chain_spec);
            dispatch_async_run!(runner, &runner.config().chain_spec, config => |components| {
                Ok(cmd.run(components.client, config.chain_spec))
            })
        }
        Some(Subcommand::ImportBlocks(cmd)) => {
            let runner = cli.create_runner(cmd)?;
            set_default_ss58_version(&runner.config().chain_spec);
            dispatch_async_run!(runner, &runner.config().chain_spec, config => |components| {
                Ok(cmd.run(components.client, components.import_queue))
            })
        }
        Some(Subcommand::Revert(cmd)) => {
            let runner = cli.create_runner(cmd)?;
            set_default_ss58_version(&runner.config().chain_spec);
            dispatch_async_run!(runner, &runner.config().chain_spec, config => |components| {
                Ok(cmd.run(components.client, components.backend, None))
            })
        }
        Some(Subcommand::PurgeChain(cmd)) => {
            let runner = cli.create_runner(cmd)?;
            set_default_ss58_version(&runner.config().chain_spec);
            runner.sync_run(|config| cmd.run(config.database))
        }
        #[cfg(feature = "runtime-benchmarks")]
        Some(Subcommand::Benchmark(cmd)) => {
            use frame_benchmarking_cli::{BenchmarkCmd, SUBSTRATE_REFERENCE_HARDWARE};

            let runner = cli.create_runner(cmd)?;
            set_default_ss58_version(&runner.config().chain_spec);

            match cmd {
                BenchmarkCmd::Pallet(_) => {
                    Err("Pallet benchmarking has migrated to its own CLI tool, \
                    please read https://github.com/paritytech/polkadot-sdk/pull/3512."
                        .into())
                }
                BenchmarkCmd::Storage(cmd) => runner.sync_run(|config| {
                    dispatch_benchmark_partials!(config => |partials| {
                        let db = partials.backend.expose_db();
                        let storage = partials.backend.expose_storage();
                        let shared_trie_cache = partials.backend.expose_shared_trie_cache();
                        cmd.run(config, partials.client.clone(), db, storage, shared_trie_cache)
                    })
                }),
                BenchmarkCmd::Overhead(_) => Err("Unsupported benchmarking command".into()),
                BenchmarkCmd::Extrinsic(_) => Err("Unsupported benchmarking command".into()),
                BenchmarkCmd::Block(cmd) => runner.sync_run(|config| {
                    dispatch_benchmark_partials!(config => |partials| {
                        cmd.run(partials.client)
                    })
                }),
                BenchmarkCmd::Machine(cmd) => {
                    runner.sync_run(|config| cmd.run(&config, SUBSTRATE_REFERENCE_HARDWARE.clone()))
                }
            }
        }
        #[cfg(not(feature = "runtime-benchmarks"))]
        Some(Subcommand::Benchmark(_)) => {
            Err("Benchmarking was not enabled when building the node. \
            You can enable it with `--features runtime-benchmarks`."
                .into())
        }
        None => {
            let runner = cli.create_runner(&cli.run)?;
            let no_hardware_benchmarks = cli.no_hardware_benchmarks;
            let storage_monitor = cli.storage_monitor.clone();

            runner.run_node_until_exit(move |config| async move {
                let hwbench = (!no_hardware_benchmarks)
                    .then(|| {
                        config.database.path().map(|database_path| {
                            let _ = std::fs::create_dir_all(&database_path);
                            sc_sysinfo::gather_hwbench(
                                Some(database_path),
                                &frame_benchmarking_cli::SUBSTRATE_REFERENCE_HARDWARE,
                            )
                        })
                    })
                    .flatten();

                if let Some(ref hwbench) = hwbench {
                    sc_sysinfo::print_hwbench(hwbench);
                    if let Err(err) = frame_benchmarking_cli::SUBSTRATE_REFERENCE_HARDWARE
                        .check_hardware(hwbench, config.role.is_authority())
                    {
                        log::warn!("Hardware does not meet reference requirements: {err}");
                    }
                }

                let database_source = config.database.clone();
                let chain_spec = &config.chain_spec;
                set_default_ss58_version(chain_spec);

                log::info!(
                    "Is validating: {}",
                    if config.role.is_authority() {
                        "yes"
                    } else {
                        "no"
                    }
                );

                let task_manager: sc_service::TaskManager =
                    dispatch_on_runtime!(chain_spec => |RuntimeApi| {
                        service::new_full_from_network_cfg::<RuntimeApi>(config)
                            .map_err(|e| sc_cli::Error::from(*e))
                    })?;

                if let Some(path) = database_source.path() {
                    StorageMonitorService::try_spawn(
                        storage_monitor,
                        path.to_path_buf(),
                        &task_manager.spawn_essential_handle(),
                    )
                    .map_err(|e| sc_cli::Error::Application(Box::new(e)))?;
                }

                Ok(task_manager)
            })
        }
    }
}

fn load_spec(id: &str) -> std::result::Result<Box<dyn ChainSpecT>, String> {
    let id = if id.is_empty() {
        let n = get_exec_name().unwrap_or_else(|| {
            log::warn!("Failed to detect executable name, falling back to \"allfeat\"");
            String::new()
        });
        ["melodie", "allfeat"]
            .iter()
            .cloned()
            .find(|&chain| n.starts_with(chain))
            .unwrap_or("allfeat")
    } else {
        id
    };
    let chain_spec = match id.to_lowercase().as_str() {
        #[cfg(feature = "allfeat-runtime")]
        "" | "allfeat" => Box::new(ChainSpec::from_json_bytes(
            &include_bytes!("../specs/mainnet/allfeat_raw.json")[..],
        )?),
        #[cfg(feature = "melodie-runtime")]
        "melodie" => Box::new(ChainSpec::from_json_bytes(
            &include_bytes!("../specs/testnets/melodie/v2/melodie_raw.json")[..],
        )?),
        #[cfg(feature = "melodie-runtime")]
        "melodie-staging" => Box::new(melodie_chain_spec::live_chain_spec()?),
        #[cfg(feature = "melodie-runtime")]
        "melodie-local" => Box::new(melodie_chain_spec::local_chain_spec()?),

        #[cfg(feature = "allfeat-runtime")]
        "allfeat-staging" => Box::new(allfeat_chain_spec::live_chain_spec()?),
        #[cfg(feature = "allfeat-runtime")]
        "allfeat-local" => Box::new(allfeat_chain_spec::local_chain_spec()?),

        #[cfg(feature = "melodie-runtime")]
        "dev" | "melodie-dev" => Box::new(melodie_chain_spec::development_chain_spec()?),
        #[cfg(feature = "allfeat-runtime")]
        "allfeat-dev" => Box::new(allfeat_chain_spec::development_chain_spec()?),
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
