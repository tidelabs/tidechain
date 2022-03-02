#![allow(clippy::type_complexity)]

#[cfg(feature = "full-node")]
pub use tidechain_client::{
  AbstractClient, Client, ClientHandle, ExecuteWithClient, FullBackend, FullClient,
  RuntimeApiCollection,
};

#[cfg(feature = "full-node")]
use {
  sc_client_api::{BlockBackend, ExecutorProvider},
  sc_executor::NativeElseWasmExecutor,
  sc_finality_grandpa::FinalityProofProvider as GrandpaFinalityProofProvider,
  sc_service::{
    config::PrometheusConfig, Configuration, NativeExecutionDispatch, RpcHandlers, TaskManager,
  },
  sc_telemetry::{Telemetry, TelemetryWorker},
  sp_api::ConstructRuntimeApi,
  sp_runtime::traits::Block as BlockT,
  std::{sync::Arc, time::Duration},
  substrate_prometheus_endpoint::Registry,
  tidefi_primitives::Block,
};

use sc_service::ChainSpec;

#[cfg(feature = "tidechain-native")]
pub use tidechain_client::TidechainExecutorDispatch;
#[cfg(feature = "tidechain-native")]
pub use tidechain_runtime;

#[cfg(feature = "hertel-native")]
pub use hertel_runtime;
#[cfg(feature = "hertel-native")]
pub use tidechain_client::HertelExecutorDispatch;

#[cfg(any(feature = "tidechain-native", feature = "hertel-native"))]
pub mod chain_spec;

#[derive(thiserror::Error, Debug)]
pub enum Error {
  #[error(transparent)]
  Io(#[from] std::io::Error),

  #[error(transparent)]
  AddrFormatInvalid(#[from] std::net::AddrParseError),

  #[error(transparent)]
  Sub(#[from] sc_service::Error),

  #[error(transparent)]
  Blockchain(#[from] sp_blockchain::Error),

  #[error(transparent)]
  Consensus(#[from] sp_consensus::Error),

  #[error(transparent)]
  Prometheus(#[from] substrate_prometheus_endpoint::PrometheusError),

  #[error(transparent)]
  Telemetry(#[from] sc_telemetry::Error),

  #[error("Expected at least one of polkadot, kusama, westend or rococo runtime feature")]
  NoRuntime,
}

/// Can be called for a `Configuration` to check if it is a configuration for the `Hertel` network.
pub trait IdentifyVariant {
  /// Returns if this is a configuration for the `Tidechain` network.
  fn is_tidechain(&self) -> bool;

  /// Returns if this is a configuration for the `Hertel` network.
  fn is_hertel(&self) -> bool;
}

impl IdentifyVariant for Box<dyn ChainSpec> {
  fn is_tidechain(&self) -> bool {
    self.id().starts_with("tide")
  }
  fn is_hertel(&self) -> bool {
    self.id().starts_with("hert")
  }
}

#[cfg(feature = "full-node")]
pub type FullSelectChain = sc_consensus::LongestChain<FullBackend, Block>;
#[cfg(feature = "full-node")]
pub type FullGrandpaBlockImport<RuntimeApi, ExecutorDispatch> =
  sc_finality_grandpa::GrandpaBlockImport<
    FullBackend,
    Block,
    FullClient<RuntimeApi, ExecutorDispatch>,
    FullSelectChain,
  >;

// If we're using prometheus, use a registry with a prefix of `tidechain`.
#[cfg(feature = "full-node")]
fn set_prometheus_registry(config: &mut Configuration) -> Result<(), Error> {
  if let Some(PrometheusConfig { registry, .. }) = config.prometheus_config.as_mut() {
    *registry = Registry::new_custom(Some("tidechain".into()), None)?;
  }

  Ok(())
}

#[cfg(feature = "full-node")]
fn new_partial<RuntimeApi, ExecutorDispatch>(
  config: &mut Configuration,
) -> Result<
  sc_service::PartialComponents<
    FullClient<RuntimeApi, ExecutorDispatch>,
    FullBackend,
    FullSelectChain,
    sc_consensus::DefaultImportQueue<Block, FullClient<RuntimeApi, ExecutorDispatch>>,
    sc_transaction_pool::FullPool<Block, FullClient<RuntimeApi, ExecutorDispatch>>,
    (
      impl sc_service::RpcExtensionBuilder,
      (
        sc_consensus_babe::BabeBlockImport<
          Block,
          FullClient<RuntimeApi, ExecutorDispatch>,
          FullGrandpaBlockImport<RuntimeApi, ExecutorDispatch>,
        >,
        sc_finality_grandpa::LinkHalf<
          Block,
          FullClient<RuntimeApi, ExecutorDispatch>,
          FullSelectChain,
        >,
        sc_consensus_babe::BabeLink<Block>,
      ),
      sc_finality_grandpa::SharedVoterState,
      sp_consensus_babe::SlotDuration,
      Option<Telemetry>,
    ),
  >,
  Error,
>
where
  RuntimeApi:
    ConstructRuntimeApi<Block, FullClient<RuntimeApi, ExecutorDispatch>> + Send + Sync + 'static,
  RuntimeApi::RuntimeApi:
    RuntimeApiCollection<StateBackend = sc_client_api::StateBackendFor<FullBackend, Block>>,
  ExecutorDispatch: NativeExecutionDispatch + 'static,
{
  set_prometheus_registry(config)?;

  let telemetry = config
    .telemetry_endpoints
    .clone()
    .filter(|x| !x.is_empty())
    .map(|endpoints| -> Result<_, sc_telemetry::Error> {
      let worker = TelemetryWorker::new(16)?;
      let telemetry = worker.handle().new_telemetry(endpoints);
      Ok((worker, telemetry))
    })
    .transpose()?;

  let executor = NativeElseWasmExecutor::<ExecutorDispatch>::new(
    config.wasm_method,
    config.default_heap_pages,
    config.max_runtime_instances,
    config.runtime_cache_size,
  );

  let (client, backend, keystore_container, task_manager) =
    sc_service::new_full_parts::<Block, RuntimeApi, _>(
      config,
      telemetry.as_ref().map(|(_, telemetry)| telemetry.handle()),
      executor,
    )?;
  let client = Arc::new(client);

  let telemetry = telemetry.map(|(worker, telemetry)| {
    task_manager
      .spawn_handle()
      .spawn("telemetry", Some("telemetry"), Box::pin(worker.run()));
    telemetry
  });

  let select_chain = sc_consensus::LongestChain::new(backend.clone());

  let transaction_pool = sc_transaction_pool::BasicPool::new_full(
    config.transaction_pool.clone(),
    config.role.is_authority().into(),
    config.prometheus_registry(),
    task_manager.spawn_essential_handle(),
    client.clone(),
  );

  let grandpa_hard_forks = Vec::new();

  let (grandpa_block_import, grandpa_link) =
    sc_finality_grandpa::block_import_with_authority_set_hard_forks(
      client.clone(),
      &(client.clone() as Arc<_>),
      select_chain.clone(),
      grandpa_hard_forks,
      telemetry.as_ref().map(|x| x.handle()),
    )?;

  let justification_import = grandpa_block_import.clone();

  let babe_config = sc_consensus_babe::Config::get(&*client)?;
  let (block_import, babe_link) =
    sc_consensus_babe::block_import(babe_config.clone(), grandpa_block_import, client.clone())?;

  let slot_duration = babe_link.config().slot_duration();
  let import_queue = sc_consensus_babe::import_queue(
    babe_link.clone(),
    block_import.clone(),
    Some(Box::new(justification_import)),
    client.clone(),
    select_chain.clone(),
    move |_, ()| async move {
      let timestamp = sp_timestamp::InherentDataProvider::from_system_time();

      let slot =
        sp_consensus_babe::inherents::InherentDataProvider::from_timestamp_and_slot_duration(
          *timestamp,
          slot_duration,
        );

      let uncles =
        sp_authorship::InherentDataProvider::<<Block as BlockT>::Header>::check_inherents();

      Ok((timestamp, slot, uncles))
    },
    &task_manager.spawn_essential_handle(),
    config.prometheus_registry(),
    sp_consensus::CanAuthorWithNativeVersion::new(client.executor().clone()),
    telemetry.as_ref().map(|x| x.handle()),
  )?;

  let justification_stream = grandpa_link.justification_stream();
  let shared_authority_set = grandpa_link.shared_authority_set().clone();
  let shared_voter_state = sc_finality_grandpa::SharedVoterState::empty();
  let finality_proof_provider = GrandpaFinalityProofProvider::new_for_service(
    backend.clone(),
    Some(shared_authority_set.clone()),
  );

  let import_setup = (block_import, grandpa_link, babe_link.clone());
  let rpc_setup = shared_voter_state.clone();

  let shared_epoch_changes = babe_link.epoch_changes().clone();
  let slot_duration = babe_config.slot_duration();

  let rpc_extensions_builder = {
    let client = client.clone();
    let keystore = keystore_container.sync_keystore();
    let transaction_pool = transaction_pool.clone();
    let select_chain = select_chain.clone();
    let chain_spec = config.chain_spec.cloned_box();

    move |deny_unsafe, subscription_executor| -> Result<tidechain_rpc::RpcExtension, _> {
      let deps = tidechain_rpc::FullDeps {
        client: client.clone(),
        pool: transaction_pool.clone(),
        select_chain: select_chain.clone(),
        chain_spec: chain_spec.cloned_box(),
        deny_unsafe,
        babe: tidechain_rpc::BabeDeps {
          babe_config: babe_config.clone(),
          shared_epoch_changes: shared_epoch_changes.clone(),
          keystore: keystore.clone(),
        },
        grandpa: tidechain_rpc::GrandpaDeps {
          shared_voter_state: shared_voter_state.clone(),
          shared_authority_set: shared_authority_set.clone(),
          justification_stream: justification_stream.clone(),
          subscription_executor,
          finality_provider: finality_proof_provider.clone(),
        },
      };

      Ok(tidechain_rpc::create_full(deps)?)
    }
  };

  Ok(sc_service::PartialComponents {
    client,
    backend,
    task_manager,
    keystore_container,
    select_chain,
    import_queue,
    transaction_pool,
    other: (
      rpc_extensions_builder,
      import_setup,
      rpc_setup,
      slot_duration,
      telemetry,
    ),
  })
}

#[cfg(feature = "full-node")]
pub struct NewFull<C> {
  pub task_manager: TaskManager,
  pub client: C,
  pub network: Arc<sc_network::NetworkService<Block, <Block as BlockT>::Hash>>,
  pub rpc_handlers: RpcHandlers,
}

#[cfg(feature = "full-node")]
impl<C> NewFull<C> {
  /// Convert the client type using the given `func`.
  pub fn with_client<NC>(self, func: impl FnOnce(C) -> NC) -> NewFull<NC> {
    NewFull {
      client: func(self.client),
      task_manager: self.task_manager,
      network: self.network,
      rpc_handlers: self.rpc_handlers,
    }
  }
}

#[cfg(feature = "full-node")]
pub fn new_full<RuntimeApi, Executor>(
  mut config: Configuration,
) -> Result<NewFull<Arc<FullClient<RuntimeApi, Executor>>>, Error>
where
  RuntimeApi: ConstructRuntimeApi<Block, FullClient<RuntimeApi, Executor>> + Send + Sync + 'static,
  RuntimeApi::RuntimeApi:
    RuntimeApiCollection<StateBackend = sc_client_api::StateBackendFor<FullBackend, Block>>,
  Executor: NativeExecutionDispatch + 'static,
{
  let role = config.role.clone();
  let force_authoring = config.force_authoring;
  let backoff_authoring_blocks =
    Some(sc_consensus_slots::BackoffAuthoringOnFinalizedHeadLagging::default());

  let disable_grandpa = config.disable_grandpa;
  let name = config.network.node_name.clone();

  let sc_service::PartialComponents {
    client,
    backend,
    mut task_manager,
    keystore_container,
    select_chain,
    import_queue,
    transaction_pool,
    other: (rpc_extensions_builder, import_setup, rpc_setup, _slot_duration, mut telemetry),
  } = new_partial::<RuntimeApi, Executor>(&mut config)?;

  let prometheus_registry = config.prometheus_registry().cloned();

  let shared_voter_state = rpc_setup;

  // Note: GrandPa is pushed before the Tidechain-specific protocols. This doesn't change
  // anything in terms of behaviour, but makes the logs more consistent with the other
  // Substrate nodes.
  let grandpa_protocol_name = sc_finality_grandpa::protocol_standard_name(
    &client
      .block_hash(0)
      .ok()
      .flatten()
      .expect("Genesis block exists; qed"),
    &config.chain_spec,
  );
  config
    .network
    .extra_sets
    .push(sc_finality_grandpa::grandpa_peers_set_config(
      grandpa_protocol_name.clone(),
    ));

  let warp_sync = Arc::new(sc_finality_grandpa::warp_proof::NetworkProvider::new(
    backend.clone(),
    import_setup.1.shared_authority_set().clone(),
    Default::default(),
  ));

  let (network, system_rpc_tx, network_starter) =
    sc_service::build_network(sc_service::BuildNetworkParams {
      config: &config,
      client: client.clone(),
      transaction_pool: transaction_pool.clone(),
      spawn_handle: task_manager.spawn_handle(),
      import_queue,
      block_announce_validator_builder: None,
      warp_sync: Some(warp_sync),
    })?;

  if config.offchain_worker.enabled {
    let _ = sc_service::build_offchain_workers(
      &config,
      task_manager.spawn_handle(),
      client.clone(),
      network.clone(),
    );
  }

  let rpc_handlers = sc_service::spawn_tasks(sc_service::SpawnTasksParams {
    config,
    backend,
    client: client.clone(),
    keystore: keystore_container.sync_keystore(),
    network: network.clone(),
    rpc_extensions_builder: Box::new(rpc_extensions_builder),
    transaction_pool: transaction_pool.clone(),
    task_manager: &mut task_manager,
    system_rpc_tx,
    telemetry: telemetry.as_mut(),
  })?;

  let (block_import, link_half, babe_link) = import_setup;

  if let sc_service::config::Role::Authority { .. } = &role {
    let can_author_with = sp_consensus::CanAuthorWithNativeVersion::new(client.executor().clone());

    let proposer = sc_basic_authorship::ProposerFactory::new(
      task_manager.spawn_handle(),
      client.clone(),
      transaction_pool,
      prometheus_registry.as_ref(),
      telemetry.as_ref().map(|x| x.handle()),
    );

    let client_clone = client.clone();
    let slot_duration = babe_link.config().slot_duration();
    let babe_config = sc_consensus_babe::BabeParams {
      keystore: keystore_container.sync_keystore(),
      client: client.clone(),
      select_chain,
      block_import,
      env: proposer,
      sync_oracle: network.clone(),
      justification_sync_link: network.clone(),
      force_authoring,
      backoff_authoring_blocks,
      babe_link,
      can_author_with,
      create_inherent_data_providers: move |parent, ()| {
        let client_clone = client_clone.clone();
        async move {
          let uncles =
            sc_consensus_uncles::create_uncles_inherent_data_provider(&*client_clone, parent)?;

          let timestamp = sp_timestamp::InherentDataProvider::from_system_time();

          let slot =
            sp_consensus_babe::inherents::InherentDataProvider::from_timestamp_and_slot_duration(
              *timestamp,
              slot_duration,
            );

          Ok((timestamp, slot, uncles))
        }
      },
      block_proposal_slot_portion: sc_consensus_babe::SlotProportion::new(2f32 / 3f32),
      max_block_proposal_slot_portion: None,
      telemetry: telemetry.as_ref().map(|x| x.handle()),
    };

    let babe = sc_consensus_babe::start_babe(babe_config)?;
    task_manager
      .spawn_essential_handle()
      .spawn_blocking("babe-proposer", None, babe);
  }

  if role.is_authority() {
    use futures::StreamExt;
    use sc_network::Event;

    let authority_discovery_role = if role.is_authority() {
      sc_authority_discovery::Role::PublishAndDiscover(keystore_container.keystore())
    } else {
      // don't publish our addresses when we're only a collator
      sc_authority_discovery::Role::Discover
    };
    let dht_event_stream = network
      .event_stream("authority-discovery")
      .filter_map(|e| async move {
        match e {
          Event::Dht(e) => Some(e),
          _ => None,
        }
      });
    let (worker, _service) = sc_authority_discovery::new_worker_and_service(
      client.clone(),
      network.clone(),
      Box::pin(dht_event_stream),
      authority_discovery_role,
      prometheus_registry.clone(),
    );

    task_manager.spawn_handle().spawn(
      "authority-discovery-worker",
      Some("authority-discovery"),
      worker.run(),
    );
  }

  // we'd say let overseer_handler = authority_hertel_service.map(|authority_hertel_service|, ...),
  // but in that case we couldn't use ? to propagate errors
  let local_keystore = keystore_container.local_keystore();
  if local_keystore.is_none() {
    tracing::info!("Cannot run as validator without local keystore.");
  }

  // if the node isn't actively participating in consensus then it doesn't
  // need a keystore, regardless of which protocol we use below.
  let keystore_opt = if role.is_authority() {
    Some(keystore_container.sync_keystore())
  } else {
    None
  };

  let config = sc_finality_grandpa::Config {
    // FIXME substrate#1578 make this available through chainspec
    gossip_duration: Duration::from_millis(1000),
    justification_period: 512,
    name: Some(name),
    observer_enabled: false,
    keystore: keystore_opt,
    local_role: role,
    telemetry: telemetry.as_ref().map(|x| x.handle()),
    protocol_name: grandpa_protocol_name,
  };

  let enable_grandpa = !disable_grandpa;
  if enable_grandpa {
    // start the full GRANDPA voter
    // NOTE: unlike in substrate we are currently running the full
    // GRANDPA voter protocol for all full nodes (regardless of whether
    // they're validators or not). at this point the full voter should
    // provide better guarantees of block and vote data availability than
    // the observer.

    // add a custom voting rule to temporarily stop voting for new blocks
    // after the given pause block is finalized and restarting after the
    // given delay.
    let builder = sc_finality_grandpa::VotingRulesBuilder::default();

    let voting_rule = builder.build();

    let grandpa_config = sc_finality_grandpa::GrandpaParams {
      config,
      link: link_half,
      network: network.clone(),
      voting_rule,
      prometheus_registry,
      shared_voter_state,
      telemetry: telemetry.as_ref().map(|x| x.handle()),
    };

    task_manager.spawn_essential_handle().spawn_blocking(
      "grandpa-voter",
      None,
      sc_finality_grandpa::run_grandpa_voter(grandpa_config)?,
    );
  }

  network_starter.start_network();

  Ok(NewFull {
    task_manager,
    client,
    network,
    rpc_handlers,
  })
}

#[cfg(feature = "full-node")]
pub fn build_full(config: Configuration) -> Result<NewFull<Client>, Error> {
  #[cfg(feature = "tidechain-native")]
  if config.chain_spec.is_tidechain() {
    return new_full::<tidechain_runtime::RuntimeApi, TidechainExecutorDispatch>(config)
      .map(|full| full.with_client(Client::Tidechain));
  }

  #[cfg(feature = "hertel-native")]
  if config.chain_spec.is_hertel() {
    return new_full::<hertel_runtime::RuntimeApi, HertelExecutorDispatch>(config)
      .map(|full| full.with_client(Client::Hertel));
  }

  Err(Error::NoRuntime)
}

#[cfg(feature = "full-node")]
pub struct NewChainOps<C> {
  pub task_manager: TaskManager,
  pub client: C,
  pub import_queue:
    sc_consensus::BasicQueue<Block, sp_trie::PrefixedMemoryDB<sp_runtime::traits::BlakeTwo256>>,
  pub backend: Arc<FullBackend>,
}

/// Builds a new object suitable for chain operations.
#[cfg(feature = "full-node")]
pub fn new_chain_ops(mut config: &mut Configuration) -> Result<NewChainOps<Client>, Error> {
  config.keystore = sc_service::config::KeystoreConfig::InMemory;

  #[cfg(feature = "tidechain-native")]
  if config.chain_spec.is_tidechain() {
    let sc_service::PartialComponents {
      client,
      backend,
      import_queue,
      task_manager,
      ..
    } = new_partial::<tidechain_runtime::RuntimeApi, TidechainExecutorDispatch>(config)?;
    return Ok(NewChainOps {
      client: Client::Tidechain(client),
      backend,
      import_queue,
      task_manager,
    });
  }

  #[cfg(feature = "hertel-native")]
  if config.chain_spec.is_hertel() {
    let sc_service::PartialComponents {
      client,
      backend,
      import_queue,
      task_manager,
      ..
    } = new_partial::<hertel_runtime::RuntimeApi, HertelExecutorDispatch>(config)?;
    return Ok(NewChainOps {
      client: Client::Hertel(client),
      backend,
      import_queue,
      task_manager,
    });
  }

  Err(Error::NoRuntime)
}
