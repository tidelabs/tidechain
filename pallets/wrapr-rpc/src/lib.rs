// Copyright 2021 Parallel Finance Developer.
// This file is part of Parallel Finance.

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
// http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::sync::Arc;

use codec::Codec;
use jsonrpc_core::{Error as RpcError, ErrorCode, Result};
use jsonrpc_derive::rpc;
pub use pallet_wrapr_rpc_runtime_api::WraprApi as WraprRuntimeApi;
use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;
use sp_runtime::{generic::BlockId, traits::Block as BlockT};
use tidefi_primitives::{AssetId, Balance};

#[rpc]
pub trait WraprApi<BlockHash, AccountId> {
  #[rpc(name = "wrapr_getAccountBalance")]
  fn get_account_balance(
    &self,
    asset_id: AssetId,
    account_id: AccountId,
    at: Option<BlockHash>,
  ) -> Result<Balance>;
}

/// A struct that implements the [`WraprApi`].
pub struct Wrapr<C, B> {
  client: Arc<C>,
  _marker: std::marker::PhantomData<B>,
}

impl<C, B> Wrapr<C, B> {
  /// Create new `Wrapr` with the given reference to the client.
  pub fn new(client: Arc<C>) -> Self {
    Self {
      client,
      _marker: Default::default(),
    }
  }
}

pub enum Error {
  RuntimeError,
  AccountBalanceError,
}

impl From<Error> for i64 {
  fn from(e: Error) -> i64 {
    match e {
      Error::RuntimeError => 1,
      Error::AccountBalanceError => 2,
    }
  }
}

impl<C, Block, AccountId> WraprApi<<Block as BlockT>::Hash, AccountId> for Wrapr<C, Block>
where
  Block: BlockT,
  C: Send + Sync + 'static,
  C: ProvideRuntimeApi<Block>,
  C: HeaderBackend<Block>,
  C::Api: WraprRuntimeApi<Block, AccountId>,
  AccountId: Codec,
{
  fn get_account_balance(
    &self,
    asset_id: AssetId,
    account_id: AccountId,
    at: Option<<Block as BlockT>::Hash>,
  ) -> Result<Balance> {
    let api = self.client.runtime_api();
    let at = BlockId::hash(at.unwrap_or(
      // If the block hash is not supplied assume the best block.
      self.client.info().best_hash,
    ));
    api
      .get_account_balance(&at, asset_id, account_id)
      .map_err(runtime_error_into_rpc_error)?
      .map_err(account_balance_error_into_rpc_error)
  }
}

/// Converts a runtime trap into an RPC error.
fn runtime_error_into_rpc_error(err: impl std::fmt::Debug) -> RpcError {
  RpcError {
    code: ErrorCode::ServerError(Error::RuntimeError.into()),
    message: "Runtime trapped".into(),
    data: Some(format!("{:?}", err).into()),
  }
}

/// Converts an account balance error into an RPC error.
fn account_balance_error_into_rpc_error(err: impl std::fmt::Debug) -> RpcError {
  RpcError {
    code: ErrorCode::ServerError(Error::AccountBalanceError.into()),
    message: "Not able to get account balance".into(),
    data: Some(format!("{:?}", err).into()),
  }
}
