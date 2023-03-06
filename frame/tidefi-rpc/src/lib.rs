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

use codec::Codec;
use frame_support::inherent::Vec;
use jsonrpsee::{
  core::{Error as JsonRpseeError, RpcResult},
  proc_macros::rpc,
  types::{error::CallError, ErrorObject},
};
pub use pallet_tidefi_rpc_runtime_api::TidefiApi as TidefiRuntimeApi;
use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;
use sp_runtime::traits::Block as BlockT;
use std::sync::Arc;
use tidefi_primitives::{
  BalanceInfo, BlockNumber, CurrencyBalance, CurrencyId, CurrencyMetadata, Stake,
};

#[rpc(client, server)]
pub trait TidefiApi<BlockHash, AccountId> {
  #[method(name = "tidefi_getAssets")]
  fn get_assets(
    &self,
    at: Option<BlockHash>,
  ) -> RpcResult<Vec<(CurrencyId, CurrencyMetadata<Vec<u8>>)>>;

  #[method(name = "tidefi_getAccountBalance")]
  fn get_account_balance(
    &self,
    account_id: AccountId,
    asset_id: CurrencyId,
    at: Option<BlockHash>,
  ) -> RpcResult<CurrencyBalance<BalanceInfo>>;

  #[method(name = "tidefi_getAccountBalances")]
  fn get_account_balances(
    &self,
    account_id: AccountId,
    at: Option<BlockHash>,
  ) -> RpcResult<Vec<(CurrencyId, CurrencyBalance<BalanceInfo>)>>;

  #[method(name = "tidefi_getAccountStakes")]
  fn get_account_stakes(
    &self,
    account_id: AccountId,
    at: Option<BlockHash>,
  ) -> RpcResult<Vec<(CurrencyId, Stake<BalanceInfo, BlockNumber>)>>;
}

/// A struct that implements the [`TidefiApi`].
pub struct TidefiRpc<C, B> {
  client: Arc<C>,
  _marker: std::marker::PhantomData<B>,
}

impl<C, B> TidefiRpc<C, B> {
  /// Create new `Tidefi` with the given reference to the client.
  pub fn new(client: Arc<C>) -> Self {
    Self {
      client,
      _marker: Default::default(),
    }
  }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
  #[error("{0}")]
  Call(String),
  #[error("{0}")]
  Dispatch(String),
}

impl From<Error> for JsonRpseeError {
  fn from(error: Error) -> Self {
    JsonRpseeError::Call(CallError::Custom(ErrorObject::owned(
      1234,
      error.to_string(),
      None::<()>,
    )))
  }
}

impl<C, Block, AccountId> TidefiApiServer<<Block as BlockT>::Hash, AccountId>
  for TidefiRpc<C, Block>
where
  Block: BlockT,
  C: Send + Sync + 'static,
  C: ProvideRuntimeApi<Block>,
  C: HeaderBackend<Block>,
  C::Api: TidefiRuntimeApi<Block, AccountId>,
  AccountId: Codec,
{
  fn get_account_balance(
    &self,
    account_id: AccountId,
    asset_id: CurrencyId,
    at: Option<<Block as BlockT>::Hash>,
  ) -> RpcResult<CurrencyBalance<BalanceInfo>> {
    let api = self.client.runtime_api();
    let at = at.unwrap_or(
      // If the block hash is not supplied assume the best block.
      self.client.info().best_hash,
    );
    api
      .get_account_balance(at, account_id, asset_id)
      .map_err(|err| Error::Call(format!("{err:?}")))?
      .map_err(|err| Error::Dispatch(format!("{err:?}")).into())
  }

  fn get_account_stakes(
    &self,
    account_id: AccountId,
    at: Option<<Block as BlockT>::Hash>,
  ) -> RpcResult<Vec<(CurrencyId, Stake<BalanceInfo, BlockNumber>)>> {
    let api = self.client.runtime_api();
    let at = at.unwrap_or(
      // If the block hash is not supplied assume the best block.
      self.client.info().best_hash,
    );
    api
      .get_account_stakes(at, account_id)
      .map_err(|err| Error::Call(format!("{err:?}")))?
      .map_err(|err| Error::Dispatch(format!("{err:?}")).into())
  }

  fn get_assets(
    &self,
    at: Option<<Block as BlockT>::Hash>,
  ) -> RpcResult<Vec<(CurrencyId, CurrencyMetadata<Vec<u8>>)>> {
    let api = self.client.runtime_api();
    let at = at.unwrap_or(
      // If the block hash is not supplied assume the best block.
      self.client.info().best_hash,
    );
    api
      .get_assets(at)
      .map_err(|err| Error::Call(format!("{err:?}")))?
      .map_err(|err| Error::Dispatch(format!("{err:?}")).into())
  }

  fn get_account_balances(
    &self,
    account_id: AccountId,
    at: Option<<Block as BlockT>::Hash>,
  ) -> RpcResult<Vec<(CurrencyId, CurrencyBalance<BalanceInfo>)>> {
    let api = self.client.runtime_api();
    let at = at.unwrap_or(
      // If the block hash is not supplied assume the best block.
      self.client.info().best_hash,
    );
    api
      .get_account_balances(at, account_id)
      .map_err(|err| Error::Call(format!("{err:?}")))?
      .map_err(|err| Error::Dispatch(format!("{err:?}")).into())
  }
}
