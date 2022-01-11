use sc_client_api::{Backend as BackendT, BlockchainEvents, KeyIterator};
pub use sc_executor::NativeElseWasmExecutor;
use sp_api::{CallApiAt, NumberFor, ProvideRuntimeApi};
use sp_blockchain::HeaderBackend;
use sp_runtime::{
  generic::{BlockId, SignedBlock},
  traits::{BlakeTwo256, Block as BlockT},
  Justifications,
};
use sp_storage::{ChildInfo, StorageData, StorageKey};
use std::sync::Arc;
use tidefi_primitives::{AccountId, Balance, Block, BlockNumber, Hash, Header};

#[cfg(not(any(feature = "tidechain", feature = "hertel",)))]
compile_error!("at least one runtime feature must be enabled");

#[cfg(feature = "tidechain")]
pub struct TidechainExecutorDispatch;

#[cfg(feature = "tidechain")]
impl sc_executor::NativeExecutionDispatch for TidechainExecutorDispatch {
  type ExtendHostFunctions = frame_benchmarking::benchmarking::HostFunctions;

  fn dispatch(method: &str, data: &[u8]) -> Option<Vec<u8>> {
    tidechain_runtime::dispatch(method, data)
  }

  fn native_version() -> sc_executor::NativeVersion {
    tidechain_runtime::native_version()
  }
}

#[cfg(feature = "hertel")]
pub struct HertelExecutorDispatch;

#[cfg(feature = "hertel")]
impl sc_executor::NativeExecutionDispatch for HertelExecutorDispatch {
  type ExtendHostFunctions = frame_benchmarking::benchmarking::HostFunctions;

  fn dispatch(method: &str, data: &[u8]) -> Option<Vec<u8>> {
    hertel_runtime::dispatch(method, data)
  }

  fn native_version() -> sc_executor::NativeVersion {
    hertel_runtime::native_version()
  }
}

pub type FullBackend = sc_service::TFullBackend<Block>;
pub type FullClient<RuntimeApi, ExecutorDispatch> =
  sc_service::TFullClient<Block, RuntimeApi, NativeElseWasmExecutor<ExecutorDispatch>>;

/// A set of APIs that Tidechain-like runtimes must implement.
pub trait RuntimeApiCollection:
  sp_transaction_pool::runtime_api::TaggedTransactionQueue<Block>
  + sp_api::ApiExt<Block>
  + sp_consensus_babe::BabeApi<Block>
  + sp_finality_grandpa::GrandpaApi<Block>
  + sp_block_builder::BlockBuilder<Block>
  + frame_system_rpc_runtime_api::AccountNonceApi<Block, AccountId, u32>
  + pallet_transaction_payment_rpc_runtime_api::TransactionPaymentApi<Block, Balance>
  + sp_api::Metadata<Block>
  + sp_offchain::OffchainWorkerApi<Block>
  + sp_session::SessionKeys<Block>
  + sp_authority_discovery::AuthorityDiscoveryApi<Block>
  + pallet_tidefi_rpc::TidefiRuntimeApi<Block, AccountId>
where
  <Self as sp_api::ApiExt<Block>>::StateBackend: sp_api::StateBackend<BlakeTwo256>,
{
}

impl<Api> RuntimeApiCollection for Api
where
  Api: sp_transaction_pool::runtime_api::TaggedTransactionQueue<Block>
    + sp_api::ApiExt<Block>
    + sp_consensus_babe::BabeApi<Block>
    + sp_finality_grandpa::GrandpaApi<Block>
    + sp_block_builder::BlockBuilder<Block>
    + frame_system_rpc_runtime_api::AccountNonceApi<Block, AccountId, u32>
    + pallet_transaction_payment_rpc_runtime_api::TransactionPaymentApi<Block, Balance>
    + sp_api::Metadata<Block>
    + sp_offchain::OffchainWorkerApi<Block>
    + sp_session::SessionKeys<Block>
    + sp_authority_discovery::AuthorityDiscoveryApi<Block>
    + pallet_tidefi_rpc::TidefiRuntimeApi<Block, AccountId>,
  <Self as sp_api::ApiExt<Block>>::StateBackend: sp_api::StateBackend<BlakeTwo256>,
{
}

/// Trait that abstracts over all available client implementations.
///
/// For a concrete type there exists [`Client`].
pub trait AbstractClient<Block, Backend>:
  BlockchainEvents<Block>
  + Sized
  + Send
  + Sync
  + ProvideRuntimeApi<Block>
  + HeaderBackend<Block>
  + CallApiAt<Block, StateBackend = Backend::State>
where
  Block: BlockT,
  Backend: BackendT<Block>,
  Backend::State: sp_api::StateBackend<BlakeTwo256>,
  Self::Api: RuntimeApiCollection<StateBackend = Backend::State>,
{
}

impl<Block, Backend, Client> AbstractClient<Block, Backend> for Client
where
  Block: BlockT,
  Backend: BackendT<Block>,
  Backend::State: sp_api::StateBackend<BlakeTwo256>,
  Client: BlockchainEvents<Block>
    + ProvideRuntimeApi<Block>
    + HeaderBackend<Block>
    + Sized
    + Send
    + Sync
    + CallApiAt<Block, StateBackend = Backend::State>,
  Client::Api: RuntimeApiCollection<StateBackend = Backend::State>,
{
}

/// Execute something with the client instance.
///
/// As there exist multiple chains inside Polkadot, like Polkadot itself, Kusama, Westend etc,
/// there can exist different kinds of client types. As these client types differ in the generics
/// that are being used, we can not easily return them from a function. For returning them from a
/// function there exists [`Client`]. However, the problem on how to use this client instance still
/// exists. This trait "solves" it in a dirty way. It requires a type to implement this trait and
/// than the [`execute_with_client`](ExecuteWithClient::execute_with_client) function can be called
/// with any possible client instance.
///
/// In a perfect world, we could make a closure work in this way.
pub trait ExecuteWithClient {
  /// The return type when calling this instance.
  type Output;

  /// Execute whatever should be executed with the given client instance.
  fn execute_with_client<Client, Api, Backend>(self, client: Arc<Client>) -> Self::Output
  where
    <Api as sp_api::ApiExt<Block>>::StateBackend: sp_api::StateBackend<BlakeTwo256>,
    Backend: sc_client_api::Backend<Block> + 'static,
    Backend::State: sp_api::StateBackend<BlakeTwo256>,
    Api: RuntimeApiCollection<StateBackend = Backend::State>,
    Client: AbstractClient<Block, Backend, Api = Api> + 'static;
}

/// A handle to a Polkadot client instance.
///
/// The Polkadot service supports multiple different runtimes (Westend, Polkadot itself, etc). As each runtime has a
/// specialized client, we need to hide them behind a trait. This is this trait.
///
/// When wanting to work with the inner client, you need to use `execute_with`.
///
/// See [`ExecuteWithClient`](trait.ExecuteWithClient.html) for more information.
pub trait ClientHandle {
  /// Execute the given something with the client.
  fn execute_with<T: ExecuteWithClient>(&self, t: T) -> T::Output;
}

/// A client instance of Tidechain.
///
/// See [`ExecuteWithClient`] for more information.
#[derive(Clone)]
pub enum Client {
  #[cfg(feature = "tidechain")]
  Tidechain(Arc<FullClient<tidechain_runtime::RuntimeApi, crate::TidechainExecutorDispatch>>),
  #[cfg(feature = "hertel")]
  Hertel(Arc<FullClient<hertel_runtime::RuntimeApi, crate::HertelExecutorDispatch>>),
}

macro_rules! with_client {
	{
		$self:ident,
		$client:ident,
		{
			$( $code:tt )*
		}
	} => {
		match $self {
			#[cfg(feature = "tidechain")]
			Self::Tidechain($client) => { $( $code )* },
			#[cfg(feature = "hertel")]
			Self::Hertel($client) => { $( $code )* },
		}
	}
}

impl ClientHandle for Client {
  fn execute_with<T: ExecuteWithClient>(&self, t: T) -> T::Output {
    with_client! {
      self,
      client,
      {
        T::execute_with_client::<_, _, FullBackend>(t, client.clone())
      }
    }
  }
}

impl sc_client_api::UsageProvider<Block> for Client {
  fn usage_info(&self) -> sc_client_api::ClientInfo<Block> {
    with_client! {
      self,
      client,
      {
        client.usage_info()
      }
    }
  }
}

impl sc_client_api::BlockBackend<Block> for Client {
  fn block_body(
    &self,
    id: &BlockId<Block>,
  ) -> sp_blockchain::Result<Option<Vec<<Block as BlockT>::Extrinsic>>> {
    with_client! {
      self,
      client,
      {
        client.block_body(id)
      }
    }
  }

  fn block(&self, id: &BlockId<Block>) -> sp_blockchain::Result<Option<SignedBlock<Block>>> {
    with_client! {
      self,
      client,
      {
        client.block(id)
      }
    }
  }

  fn block_status(&self, id: &BlockId<Block>) -> sp_blockchain::Result<sp_consensus::BlockStatus> {
    with_client! {
      self,
      client,
      {
        client.block_status(id)
      }
    }
  }

  fn justifications(&self, id: &BlockId<Block>) -> sp_blockchain::Result<Option<Justifications>> {
    with_client! {
      self,
      client,
      {
        client.justifications(id)
      }
    }
  }

  fn block_hash(
    &self,
    number: NumberFor<Block>,
  ) -> sp_blockchain::Result<Option<<Block as BlockT>::Hash>> {
    with_client! {
      self,
      client,
      {
        client.block_hash(number)
      }
    }
  }

  fn indexed_transaction(
    &self,
    id: &<Block as BlockT>::Hash,
  ) -> sp_blockchain::Result<Option<Vec<u8>>> {
    with_client! {
      self,
      client,
      {
        client.indexed_transaction(id)
      }
    }
  }

  fn block_indexed_body(&self, id: &BlockId<Block>) -> sp_blockchain::Result<Option<Vec<Vec<u8>>>> {
    with_client! {
      self,
      client,
      {
        client.block_indexed_body(id)
      }
    }
  }
}

impl sc_client_api::StorageProvider<Block, FullBackend> for Client {
  fn storage(
    &self,
    id: &BlockId<Block>,
    key: &StorageKey,
  ) -> sp_blockchain::Result<Option<StorageData>> {
    with_client! {
      self,
      client,
      {
        client.storage(id, key)
      }
    }
  }

  fn storage_keys(
    &self,
    id: &BlockId<Block>,
    key_prefix: &StorageKey,
  ) -> sp_blockchain::Result<Vec<StorageKey>> {
    with_client! {
      self,
      client,
      {
        client.storage_keys(id, key_prefix)
      }
    }
  }

  fn storage_hash(
    &self,
    id: &BlockId<Block>,
    key: &StorageKey,
  ) -> sp_blockchain::Result<Option<<Block as BlockT>::Hash>> {
    with_client! {
      self,
      client,
      {
         client.storage_hash(id, key)
      }
    }
  }

  fn storage_pairs(
    &self,
    id: &BlockId<Block>,
    key_prefix: &StorageKey,
  ) -> sp_blockchain::Result<Vec<(StorageKey, StorageData)>> {
    with_client! {
      self,
      client,
      {
        client.storage_pairs(id, key_prefix)
      }
    }
  }

  fn storage_keys_iter<'a>(
    &self,
    id: &BlockId<Block>,
    prefix: Option<&'a StorageKey>,
    start_key: Option<&StorageKey>,
  ) -> sp_blockchain::Result<
    KeyIterator<'a, <FullBackend as sc_client_api::Backend<Block>>::State, Block>,
  > {
    with_client! {
      self,
      client,
      {
        client.storage_keys_iter(id, prefix, start_key)
      }
    }
  }

  fn child_storage(
    &self,
    id: &BlockId<Block>,
    child_info: &ChildInfo,
    key: &StorageKey,
  ) -> sp_blockchain::Result<Option<StorageData>> {
    with_client! {
      self,
      client,
      {
        client.child_storage(id, child_info, key)
      }
    }
  }

  fn child_storage_keys(
    &self,
    id: &BlockId<Block>,
    child_info: &ChildInfo,
    key_prefix: &StorageKey,
  ) -> sp_blockchain::Result<Vec<StorageKey>> {
    with_client! {
      self,
      client,
      {
        client.child_storage_keys(id, child_info, key_prefix)
      }
    }
  }

  fn child_storage_keys_iter<'a>(
    &self,
    id: &BlockId<Block>,
    child_info: ChildInfo,
    prefix: Option<&'a StorageKey>,
    start_key: Option<&StorageKey>,
  ) -> sp_blockchain::Result<
    KeyIterator<'a, <FullBackend as sc_client_api::Backend<Block>>::State, Block>,
  > {
    with_client! {
      self,
      client,
      {
        client.child_storage_keys_iter(id, child_info, prefix, start_key)
      }
    }
  }

  fn child_storage_hash(
    &self,
    id: &BlockId<Block>,
    child_info: &ChildInfo,
    key: &StorageKey,
  ) -> sp_blockchain::Result<Option<<Block as BlockT>::Hash>> {
    with_client! {
      self,
      client,
      {
        client.child_storage_hash(id, child_info, key)
      }
    }
  }
}

impl sp_blockchain::HeaderBackend<Block> for Client {
  fn header(&self, id: BlockId<Block>) -> sp_blockchain::Result<Option<Header>> {
    with_client! {
      self,
      client,
      {
        client.header(&id)
      }
    }
  }

  fn info(&self) -> sp_blockchain::Info<Block> {
    with_client! {
      self,
      client,
      {
        client.info()
      }
    }
  }

  fn status(&self, id: BlockId<Block>) -> sp_blockchain::Result<sp_blockchain::BlockStatus> {
    with_client! {
      self,
      client,
      {
        client.status(id)
      }
    }
  }

  fn number(&self, hash: Hash) -> sp_blockchain::Result<Option<BlockNumber>> {
    with_client! {
      self,
      client,
      {
        client.number(hash)
      }
    }
  }

  fn hash(&self, number: BlockNumber) -> sp_blockchain::Result<Option<Hash>> {
    with_client! {
      self,
      client,
      {
        client.hash(number)
      }
    }
  }
}
