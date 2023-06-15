// Copyright 2021-2022 Semantic Network Ltd.
// This file is part of Tidechain.

// Tidechain is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Tidechain is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Tidechain.  If not, see <http://www.gnu.org/licenses/>.

use crate::cli::{Cli, Subcommand};
#[cfg(feature = "runtime-benchmarks")]
use frame_benchmarking_cli::{BenchmarkCmd, SUBSTRATE_REFERENCE_HARDWARE};
use futures::future::TryFutureExt;
use log::info;
use sc_cli::{ChainSpec, RuntimeVersion, SubstrateCli};
use std::{fs::File, io::Write, path::PathBuf, sync::Arc};
use tidechain_service::{chain_spec, IdentifyVariant};

#[derive(thiserror::Error, Debug)]
pub enum Error {
  #[error(transparent)]
  TidechainService(#[from] tidechain_service::Error),

  #[error(transparent)]
  SubstrateCli(#[from] sc_cli::Error),

  #[error(transparent)]
  SubstrateService(#[from] sc_service::Error),

  #[error(transparent)]
  Io(#[from] std::io::Error),

  #[error("Wasm binary is not available")]
  UnavailableWasmBinary,

  #[error("Command is not implemented")]
  CommandNotImplemented,

  #[error("Other: {0}")]
  Other(String),
}

fn get_exec_name() -> Option<String> {
  std::env::current_exe()
    .ok()
    .and_then(|pb| pb.file_name().map(|s| s.to_os_string()))
    .and_then(|s| s.into_string().ok())
}

impl SubstrateCli for Cli {
  fn impl_name() -> String {
    "Tidechain".into()
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
    "https://github.com/tidelabs/tidechain".into()
  }

  fn copyright_start_year() -> i32 {
    2021
  }

  fn load_spec(&self, id: &str) -> Result<Box<dyn sc_service::ChainSpec>, String> {
    let id = if id.is_empty() {
      let n = get_exec_name().unwrap_or_default();
      ["tidechain", "lagoon"]
        .iter()
        .cloned()
        .find(|&chain| n.starts_with(chain))
        .unwrap_or("tidechain")
    } else {
      id
    };

    Ok(match id {
      #[cfg(feature = "lagoon-native")]
      "lagoon" => Box::new(chain_spec::lagoon_config()?),
      #[cfg(feature = "lagoon-native")]
      "lagoon-dev" => Box::new(chain_spec::lagoon_development_config()?),
      #[cfg(feature = "lagoon-native")]
      "lagoon-local" => Box::new(chain_spec::lagoon_local_testnet_config()?),
      #[cfg(feature = "lagoon-native")]
      "lagoon-staging" => Box::new(chain_spec::lagoon_staging_testnet_config()?),

      #[cfg(feature = "tidechain-native")]
      "tidechain" => Box::new(chain_spec::tidechain_config()?),
      #[cfg(feature = "tidechain-native")]
      "tidechain-dev" | "dev" => Box::new(chain_spec::tidechain_development_config()?),
      #[cfg(feature = "tidechain-native")]
      "tidechain-local" => Box::new(chain_spec::tidechain_local_testnet_config()?),
      #[cfg(feature = "tidechain-native")]
      "tidechain-staging" => Box::new(chain_spec::tidechain_staging_testnet_config()?),
      path => {
        let path = std::path::PathBuf::from(path);

        let chain_spec = Box::new(tidechain_service::TidechainChainSpec::from_json_file(
          path.clone(),
        )?) as Box<dyn tidechain_service::ChainSpec>;

        // When `force_*` is given or the file name starts with the name of one of the known chains,
        // we use the chain spec for the specific chain.
        if self.run.force_lagoon || chain_spec.is_lagoon() {
          Box::new(tidechain_service::LagoonChainSpec::from_json_file(path)?)
        } else {
          chain_spec
        }
      }
    })
  }

  fn native_runtime_version(spec: &Box<dyn ChainSpec>) -> &'static RuntimeVersion {
    #[cfg(feature = "tidechain-native")]
    if spec.is_tidechain() {
      return &tidechain_service::tidechain_runtime::VERSION;
    }

    #[cfg(feature = "lagoon-native")]
    if spec.is_lagoon() {
      return &tidechain_service::lagoon_runtime::VERSION;
    }

    panic!("No runtime feature (tidechain, lagoon) is enabled")
  }
}

#[cfg(any(feature = "runtime-benchmarks", feature = "try-runtime"))]
const DEV_ONLY_ERROR_PATTERN: &'static str =
  "can only use subcommand with --chain [tidechain-dev, lagoon-dev], got ";

#[cfg(any(feature = "runtime-benchmarks", feature = "try-runtime"))]
fn ensure_dev(spec: &Box<dyn tidechain_service::ChainSpec>) -> std::result::Result<(), String> {
  if spec.is_dev() {
    Ok(())
  } else {
    Err(format!("{}{}", DEV_ONLY_ERROR_PATTERN, spec.id()))
  }
}

#[cfg(feature = "runtime-benchmarks")]
macro_rules! unwrap_client {
  (
		$client:ident,
		$code:expr
	) => {
    match $client {
      #[cfg(feature = "lagoon-native")]
      tidechain_client::Client::Lagoon($client) => $code,
      #[cfg(feature = "tidechain-native")]
      tidechain_client::Client::Tidechain($client) => $code,
      #[allow(unreachable_patterns)]
      _ => Err(Error::CommandNotImplemented),
    }
  };
}

#[allow(clippy::borrowed_box)]
fn set_default_ss58_version(spec: &Box<dyn sc_service::ChainSpec>) {
  use sp_core::crypto::Ss58AddressFormatRegistry;

  let ss58_version = if spec.is_tidechain() {
    Ss58AddressFormatRegistry::TidefiAccount
  } else {
    Ss58AddressFormatRegistry::SubstrateAccount
  }
  .into();

  sp_core::crypto::set_default_ss58_version(ss58_version);
}

/// Parses Tidechain specific CLI arguments and run the service.
pub fn run() -> Result<(), Error> {
  let cli = Cli::from_args();

  match &cli.subcommand {
    None => {
      let runner = cli.create_runner(&cli.run.base).map_err(Error::from)?;
      let chain_spec = &runner.config().chain_spec;

      set_default_ss58_version(chain_spec);

      runner.run_node_until_exit(move |config| async move {
        let hwbench = (!cli.run.no_hardware_benchmarks)
          .then_some(config.database.path().map(|database_path| {
            let _ = std::fs::create_dir_all(database_path);
            sc_sysinfo::gather_hwbench(Some(database_path))
          }))
          .flatten();

        let task_manager =
          tidechain_service::build_full(config, hwbench).map(|full| full.task_manager)?;
        Ok::<_, Error>(task_manager)
      })
    }
    Some(Subcommand::BuildSpec(cmd)) => {
      let runner = cli.create_runner(cmd)?;
      Ok(runner.sync_run(|config| cmd.run(config.chain_spec, config.network))?)
    }
    Some(Subcommand::CheckBlock(cmd)) => {
      let runner = cli.create_runner(cmd).map_err(Error::SubstrateCli)?;
      let chain_spec = &runner.config().chain_spec;

      set_default_ss58_version(chain_spec);

      runner.async_run(|mut config| {
        let ops = tidechain_service::new_chain_ops(&mut config)?;
        Ok((
          cmd
            .run(Arc::new(ops.client), ops.import_queue)
            .map_err(Error::SubstrateCli),
          ops.task_manager,
        ))
      })
    }
    Some(Subcommand::ExportBlocks(cmd)) => {
      let runner = cli.create_runner(cmd)?;
      let chain_spec = &runner.config().chain_spec;

      set_default_ss58_version(chain_spec);

      Ok(runner.async_run(|mut config| {
        let ops = tidechain_service::new_chain_ops(&mut config)?;
        Ok((
          cmd
            .run(Arc::new(ops.client), config.database)
            .map_err(Error::SubstrateCli),
          ops.task_manager,
        ))
      })?)
    }
    Some(Subcommand::ExportState(cmd)) => {
      let runner = cli.create_runner(cmd)?;
      let chain_spec = &runner.config().chain_spec;

      set_default_ss58_version(chain_spec);

      Ok(runner.async_run(|mut config| {
        let ops = tidechain_service::new_chain_ops(&mut config)?;
        Ok((
          cmd
            .run(Arc::new(ops.client), config.chain_spec)
            .map_err(Error::SubstrateCli),
          ops.task_manager,
        ))
      })?)
    }
    Some(Subcommand::ImportBlocks(cmd)) => {
      let runner = cli.create_runner(cmd)?;
      let chain_spec = &runner.config().chain_spec;

      set_default_ss58_version(chain_spec);

      Ok(runner.async_run(|mut config| {
        let ops = tidechain_service::new_chain_ops(&mut config)?;
        Ok((
          cmd
            .run(Arc::new(ops.client), ops.import_queue)
            .map_err(Error::SubstrateCli),
          ops.task_manager,
        ))
      })?)
    }
    Some(Subcommand::PurgeChain(cmd)) => {
      let runner = cli.create_runner(cmd)?;
      Ok(runner.sync_run(|config| cmd.run(config.database))?)
    }
    Some(Subcommand::Revert(cmd)) => {
      let runner = cli.create_runner(cmd)?;
      let chain_spec = &runner.config().chain_spec;

      set_default_ss58_version(chain_spec);

      Ok(runner.async_run(|mut config| {
        let ops = tidechain_service::new_chain_ops(&mut config)?;
        Ok((
          cmd
            .run(Arc::new(ops.client), ops.backend, None)
            .map_err(Error::SubstrateCli),
          ops.task_manager,
        ))
      })?)
    }
    Some(Subcommand::ExportBuiltinWasm(cmd)) => {
      #[cfg(feature = "tidechain-native")]
      {
        let wasm_binary_bloaty = tidechain_service::tidechain_runtime::WASM_BINARY_BLOATY
          .ok_or(Error::UnavailableWasmBinary)?;
        let wasm_binary =
          tidechain_service::tidechain_runtime::WASM_BINARY.ok_or(Error::UnavailableWasmBinary)?;

        info!(
          "Exporting tidechain builtin wasm binary to folder: {}",
          cmd.folder
        );

        let folder = PathBuf::from(cmd.folder.clone());
        {
          let mut path = folder.clone();
          path.push("tidechain_runtime.compact.wasm");
          let mut file = File::create(path)?;
          file.write_all(wasm_binary)?;
          file.flush()?;
        }

        {
          let mut path = folder;
          path.push("tidechain_runtime.wasm");
          let mut file = File::create(path)?;
          file.write_all(wasm_binary_bloaty)?;
          file.flush()?;
        }
      }

      #[cfg(feature = "lagoon-native")]
      {
        let wasm_binary_bloaty = tidechain_service::lagoon_runtime::WASM_BINARY_BLOATY
          .ok_or(Error::UnavailableWasmBinary)?;
        let wasm_binary =
          tidechain_service::lagoon_runtime::WASM_BINARY.ok_or(Error::UnavailableWasmBinary)?;

        info!(
          "Exporting lagoon builtin wasm binary to folder: {}",
          cmd.folder
        );

        let folder = PathBuf::from(cmd.folder.clone());
        {
          let mut path = folder.clone();
          path.push("lagoon_runtime.compact.wasm");
          let mut file = File::create(path)?;
          file.write_all(wasm_binary)?;
          file.flush()?;
        }

        {
          let mut path = folder;
          path.push("lagoon_runtime.wasm");
          let mut file = File::create(path)?;
          file.write_all(wasm_binary_bloaty)?;
          file.flush()?;
        }
      }

      Ok(())
    }
    Some(Subcommand::Key(cmd)) => Ok(cmd.run(&cli)?),
    #[cfg(feature = "runtime-benchmarks")]
    Some(Subcommand::Benchmark(cmd)) => {
      let runner = cli.create_runner(cmd)?;
      let chain_spec = &runner.config().chain_spec;

      match cmd {
        BenchmarkCmd::Storage(cmd) => runner.sync_run(|mut config| {
          let ops = tidechain_service::new_chain_ops(&mut config)?;
          let db = ops.backend.expose_db();
          let storage = ops.backend.expose_storage();
          let client = ops.client.clone();

          unwrap_client!(
            client,
            cmd
              .run(config, client.clone(), db, storage)
              .map_err(Error::SubstrateCli)
          )
        }),
        BenchmarkCmd::Block(cmd) => runner.sync_run(|mut config| {
          let client = tidechain_service::new_chain_ops(&mut config)?.client;
          unwrap_client!(client, cmd.run(client.clone()).map_err(Error::SubstrateCli))
        }),

        BenchmarkCmd::Pallet(cmd) => {
          set_default_ss58_version(chain_spec);
          ensure_dev(chain_spec).map_err(Error::Other)?;

          #[cfg(feature = "lagoon-native")]
          if chain_spec.is_lagoon() {
            return runner.sync_run(|config| {
              cmd
                .run::<tidechain_service::lagoon_runtime::Block, tidechain_service::LagoonExecutorDispatch>(
                  config,
                )
                .map_err(Error::SubstrateCli)
            });
          }

          // else we assume it is tidechain
          #[cfg(feature = "tidechain-native")]
          if chain_spec.is_tidechain() {
            return runner.sync_run(|config| {
              cmd
                .run::<tidechain_service::tidechain_runtime::Block, tidechain_service::TidechainExecutorDispatch>(
                  config,
                )
                .map_err(Error::SubstrateCli)
            });
          }

          #[allow(unreachable_code)]
          Err(tidechain_service::Error::NoRuntime.into())
        }

        BenchmarkCmd::Machine(cmd) => runner.sync_run(|config| {
          cmd
            .run(&config, SUBSTRATE_REFERENCE_HARDWARE.clone())
            .map_err(Error::SubstrateCli)
        }),
        // NOTE: this allows the Tidechain client to leniently implement
        // new benchmark commands.
        #[allow(unreachable_patterns)]
        _ => Err(Error::CommandNotImplemented),
      }
    }
    #[cfg(feature = "try-runtime")]
    Some(Subcommand::TryRuntime(cmd)) => {
      use sc_executor::{sp_wasm_interface::ExtendedHostFunctions, NativeExecutionDispatch};
      use sc_service::TaskManager;
      use try_runtime_cli::block_building_info::timestamp_with_babe_info;

      type HostFunctionsOf<E> = ExtendedHostFunctions<
        sp_io::SubstrateHostFunctions,
        <E as NativeExecutionDispatch>::ExtendHostFunctions,
      >;

      let runner = cli.create_runner(cmd)?;
      let chain_spec = &runner.config().chain_spec;
      let registry = &runner
        .config()
        .prometheus_config
        .as_ref()
        .map(|cfg| &cfg.registry);

      let task_manager = TaskManager::new(runner.config().tokio_handle.clone(), *registry)
        .map_err(|e| Error::SubstrateService(sc_service::Error::Prometheus(e)))?;

      ensure_dev(chain_spec).map_err(Error::Other)?;

      #[cfg(feature = "lagoon-native")]
      if chain_spec.is_lagoon() {
        return runner.async_run(|_| {
          Ok((
            cmd.run::<tidechain_service::lagoon_runtime::Block, HostFunctionsOf<tidechain_service::LagoonExecutorDispatch>, _>(
              Some(timestamp_with_babe_info(tidechain_service::lagoon_runtime::constants::time::MILLISECS_PER_BLOCK))
            )
            .map_err(Error::SubstrateCli),
              task_manager,
          ))
        });
      }

      // else we assume it is tidechain
      #[cfg(feature = "tidechain-native")]
      if chain_spec.is_tidechain() {
        return runner.async_run(|_| {
          Ok((
            cmd.run::<tidechain_service::tidechain_runtime::Block, HostFunctionsOf<tidechain_service::TidechainExecutorDispatch>, _>(
              Some(timestamp_with_babe_info(tidechain_service::tidechain_runtime::constants::time::MILLISECS_PER_BLOCK))
            )
            .map_err(Error::SubstrateCli),
              task_manager,
          ))
        });
      }

      #[allow(unreachable_code)]
      Err(tidechain_service::Error::NoRuntime.into())
    }
  }?;

  Ok(())
}
