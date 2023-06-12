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

#![cfg_attr(not(feature = "std"), no_std)]
use codec::Codec;
use frame_support::inherent::Vec;
use sp_runtime::DispatchError;
use tidefi_primitives::{
  BalanceInfo, BlockNumber, CurrencyBalance, CurrencyId, CurrencyMetadata, Stake,
};

type CurrenciesMetadata = (CurrencyId, CurrencyMetadata<Vec<u8>>);
type CurrenciesStake = (CurrencyId, Stake<BalanceInfo, BlockNumber>);

sp_api::decl_runtime_apis! {
    pub trait TidefiApi<AccountId> where AccountId: Codec,
        {
            fn get_account_balance(account_id: AccountId, asset_id: CurrencyId) -> Result<CurrencyBalance<BalanceInfo>, DispatchError>;
            fn get_account_balances(account_id: AccountId) -> Result<Vec<(CurrencyId, CurrencyBalance<BalanceInfo>)>, DispatchError>;
            fn get_account_stakes(account_id: AccountId) -> Result<Vec<CurrenciesStake>, DispatchError>;
            fn get_assets() -> Result<Vec<CurrenciesMetadata>, DispatchError>;
        }
}
