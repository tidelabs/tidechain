#![cfg_attr(not(feature = "std"), no_std)]

use codec::Codec;
use frame_support::inherent::Vec;
use sp_runtime::DispatchError;

use tidefi_primitives::{BalanceInfo, CurrencyId, Stake};

sp_api::decl_runtime_apis! {
    pub trait WraprApi<AccountId> where AccountId: Codec,
        {
            fn get_account_balance(asset_id: CurrencyId, account_id: AccountId) -> Result<BalanceInfo, DispatchError>;
            fn get_account_stakes(account_id: AccountId) -> Result<Vec<(CurrencyId, Stake<BalanceInfo>)>, DispatchError>;
        }
}
