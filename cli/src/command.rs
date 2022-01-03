use crate::cli::{Cli, Subcommand};
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
    "https://github.com/tide-labs/tidechain".into()
  }

  fn copyright_start_year() -> i32 {
    2021
  }

  fn load_spec(&self, id: &str) -> Result<Box<dyn sc_service::ChainSpec>, String> {
    let id = if id.is_empty() {
      let n = get_exec_name().unwrap_or_default();
      ["tidechain", "hertel"]
        .iter()
        .cloned()
        .find(|&chain| n.starts_with(chain))
        .unwrap_or("tidechain")
    } else {
      id
    };

    Ok(match id {
      #[cfg(feature = "hertel-native")]
      "hertel" => Box::new(chain_spec::hertel_config()?),
      #[cfg(feature = "hertel-native")]
      "hertel-dev" => Box::new(chain_spec::hertel_development_config()?),
      #[cfg(feature = "hertel-native")]
      "hertel-local" => Box::new(chain_spec::hertel_local_testnet_config()?),
      #[cfg(feature = "hertel-native")]
      "hertel-staging" => Box::new(chain_spec::hertel_staging_testnet_config()?),

      #[cfg(feature = "tidechain-native")]
      "tidechain" => Box::new(chain_spec::tidechain_config()?),
      #[cfg(feature = "tidechain-native")]
      "tidechain-dev" | "dev" => Box::new(chain_spec::tidechain_development_config()?),
      #[cfg(feature = "tidechain-native")]
      "tidechain-local" => Box::new(chain_spec::tidechain_local_testnet_config()?),
      #[cfg(feature = "tidechain-native")]
      "tidechain-staging" => Box::new(chain_spec::tidechain_staging_testnet_config()?),
      _path => return Err("Custom chain spec is not supported".into()),
    })
  }

  fn native_runtime_version(spec: &Box<dyn ChainSpec>) -> &'static RuntimeVersion {
    #[cfg(feature = "tidechain-native")]
    if spec.is_tidechain() {
      return &tidechain_service::tidechain_runtime::VERSION;
    }

    #[cfg(feature = "hertel-native")]
    if spec.is_hertel() {
      return &tidechain_service::hertel_runtime::VERSION;
    }

    panic!("No runtime feature (tidechain, hertel) is enabled")
  }
}

#[allow(clippy::borrowed_box)]
fn set_default_ss58_version(_spec: &Box<dyn sc_service::ChainSpec>) {
  use sp_core::crypto::Ss58AddressFormatRegistry;

  /*
  let ss58_version = if spec.is_tidechain() {
    Ss58AddressFormatRegistry::SubstrateAccount
  } else if spec.is_hertel() {
    Ss58AddressFormatRegistry::SubstrateAccount
  } else {
    Ss58AddressFormatRegistry::SubstrateAccount
  }
  .into();
  */

  sp_core::crypto::set_default_ss58_version(Ss58AddressFormatRegistry::SubstrateAccount.into());
}

/// Parses Tidechain specific CLI arguments and run the service.
pub fn run() -> Result<(), Error> {
  let cli = Cli::from_args();

  match &cli.subcommand {
    None => {
      let runner = cli.create_runner(&cli.run).map_err(Error::from)?;
      let chain_spec = &runner.config().chain_spec;

      set_default_ss58_version(chain_spec);

      runner.run_node_until_exit(move |config| async move {
        let role = config.role.clone();

        let task_manager = match role {
          //Role::Light => tidechain_service::build_light(config).map(|light| light.task_manager),
          _ => tidechain_service::build_full(config).map(|full| full.task_manager),
        }?;
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
            .run(Arc::new(ops.client), ops.backend)
            .map_err(Error::SubstrateCli),
          ops.task_manager,
        ))
      })?)
    }
    Some(Subcommand::Benchmark(cmd)) => {
      let runner = cli.create_runner(cmd)?;
      let chain_spec = &runner.config().chain_spec;

      set_default_ss58_version(chain_spec);

      Ok(runner.sync_run(|config| {
				cmd.run::<tidechain_service::tidechain_runtime::Block, tidechain_service::TidechainExecutorDispatch>(config)
					.map_err(Error::SubstrateCli)
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

      #[cfg(feature = "hertel-native")]
      {
        let wasm_binary_bloaty = tidechain_service::hertel_runtime::WASM_BINARY_BLOATY
          .ok_or(Error::UnavailableWasmBinary)?;
        let wasm_binary =
          tidechain_service::hertel_runtime::WASM_BINARY.ok_or(Error::UnavailableWasmBinary)?;

        info!(
          "Exporting hertel builtin wasm binary to folder: {}",
          cmd.folder
        );

        let folder = PathBuf::from(cmd.folder.clone());
        {
          let mut path = folder.clone();
          path.push("hertel_runtime.compact.wasm");
          let mut file = File::create(path)?;
          file.write_all(wasm_binary)?;
          file.flush()?;
        }

        {
          let mut path = folder;
          path.push("hertel_runtime.wasm");
          let mut file = File::create(path)?;
          file.write_all(wasm_binary_bloaty)?;
          file.flush()?;
        }
      }

      Ok(())
    }
    Some(Subcommand::Key(cmd)) => Ok(cmd.run(&cli)?),
    #[cfg(feature = "try-runtime")]
    Some(Subcommand::TryRuntime(cmd)) => {
      let runner = cli.create_runner(cmd)?;
      let chain_spec = &runner.config().chain_spec;
      set_default_ss58_version(chain_spec);

      runner.async_run(|config| {
        use sc_service::TaskManager;
        let registry = config.prometheus_config.as_ref().map(|cfg| &cfg.registry);
        let task_manager = TaskManager::new(config.task_executor.clone(), registry)
          .map_err(|e| Error::SubstrateService(sc_service::Error::Prometheus(e)))?;

        Ok((
          cmd
            .run::<service::kusama_runtime::Block, service::KusamaExecutor>(config)
            .map_err(Error::SubstrateCli),
          task_manager,
        ))
        // NOTE: we fetch only the block number from the block type, the chance of disparity
        // between kusama's and polkadot's block number is small enough to overlook this.
      })
    }
  }?;
  Ok(())
}
