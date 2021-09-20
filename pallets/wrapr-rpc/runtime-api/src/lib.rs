#![cfg_attr(not(feature = "std"), no_std)]

use codec::Codec;
use sp_runtime::DispatchError;
use tidefi_primitives::{AssetId, Balance};

sp_api::decl_runtime_apis! {
    pub trait WraprApi<AccountId> where
        AccountId: Codec, {
        fn get_account_balance(asset_id: AssetId, account_id: AccountId) -> Result<Balance, DispatchError>;
    }
}
