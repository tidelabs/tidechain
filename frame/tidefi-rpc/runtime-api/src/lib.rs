#![cfg_attr(not(feature = "std"), no_std)]
use codec::Codec;
use frame_support::inherent::Vec;
use sp_runtime::DispatchError;
use tidefi_primitives::{
  BalanceInfo, BlockNumber, CurrencyBalance, CurrencyId, CurrencyMetadata, Stake,
};

sp_api::decl_runtime_apis! {
    pub trait TidefiApi<AccountId> where AccountId: Codec,
        {
            fn get_account_balance(account_id: AccountId, asset_id: CurrencyId) -> Result<CurrencyBalance<BalanceInfo>, DispatchError>;
            fn get_account_balances(account_id: AccountId) -> Result<Vec<(CurrencyId, CurrencyBalance<BalanceInfo>)>, DispatchError>;
            fn get_account_stakes(account_id: AccountId) -> Result<Vec<(CurrencyId, Stake<BalanceInfo, BlockNumber>)>, DispatchError>;
            fn get_assets() -> Result<Vec<(CurrencyId, CurrencyMetadata)>, DispatchError>;
        }
}
