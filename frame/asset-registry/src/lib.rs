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

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

pub mod weights;
pub use weights::*;

// Re-export pallet items so that they can be accessed from the crate namespace.
pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
  use super::*;
  use frame_support::{
    inherent::Vec,
    pallet_prelude::*,
    traits::fungibles::{Inspect, InspectHold, Mutate, Transfer},
    PalletId,
  };
  use frame_system::{pallet_prelude::*, RawOrigin};
  use sp_runtime::traits::{AccountIdConversion, StaticLookup};
  use sp_std::vec;
  use tidefi_primitives::{
    pallet::AssetRegistryExt, AssetId, Balance, BalanceInfo, CurrencyBalance, CurrencyId,
    CurrencyMetadata,
  };

  /// Asset registry configuration
  #[pallet::config]
  pub trait Config:
    frame_system::Config + pallet_assets::Config<AssetId = AssetId, Balance = Balance>
  {
    /// Events
    type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

    /// Pallet ID
    #[pallet::constant]
    type AssetRegistryPalletId: Get<PalletId>;

    /// Weights
    type WeightInfo: WeightInfo;

    /// Tidechain currency wrapper
    type CurrencyTidefi: Inspect<Self::AccountId, AssetId = CurrencyId, Balance = Balance>
      + Mutate<Self::AccountId, AssetId = CurrencyId, Balance = Balance>
      + Transfer<Self::AccountId, AssetId = CurrencyId, Balance = Balance>
      + InspectHold<Self::AccountId, AssetId = CurrencyId, Balance = Balance>;
  }

  #[pallet::pallet]
  #[pallet::generate_store(pub (super) trait Store)]
  pub struct Pallet<T>(_);

  /// Assets Account ID owner
  #[pallet::storage]
  #[pallet::getter(fn account_id)]
  pub type AssetRegistryAccountId<T: Config> = StorageValue<_, T::AccountId, OptionQuery>;

  /// Genesis configuration
  #[pallet::genesis_config]
  pub struct GenesisConfig<T: Config> {
    /// Assets to create on initialization
    /// \[currency_id, name, symbol, decimals\]
    pub assets: Vec<(
      CurrencyId,
      Vec<u8>,
      Vec<u8>,
      u8,
      Vec<(T::AccountId, T::Balance)>,
    )>,
    /// Assets owner
    /// Only this account can modify storage on this pallet.
    pub account: T::AccountId,
  }

  #[cfg(feature = "std")]
  impl<T: Config> Default for GenesisConfig<T> {
    fn default() -> Self {
      Self {
        // empty assets by default
        assets: Vec::new(),
        // We use pallet account ID by default,
        // but should always be set in the genesis config.
        account: T::AssetRegistryPalletId::get().into_account_truncating(),
      }
    }
  }

  #[pallet::genesis_build]
  impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
    fn build(&self) {
      // 1. Save asset registry account id
      AssetRegistryAccountId::<T>::put(self.account.clone());

      // 2. Loop through all currency defined in our genesis config
      for (currency_id, name, symbol, decimals, pre_filled_account) in self.assets.clone() {
        // If it's a wrapped token, register it with pallet_assets
        if let CurrencyId::Wrapped(asset_id) = currency_id {
          let _ = Pallet::<T>::register_asset(asset_id, name, symbol, decimals, 1);
        }

        for (account_id, mint_amount) in pre_filled_account {
          let _ = T::CurrencyTidefi::mint_into(currency_id, &account_id, mint_amount);
        }
      }
    }
  }

  #[pallet::event]
  #[pallet::generate_deposit(pub(super) fn deposit_event)]
  pub enum Event<T: Config> {
    /// Asset was registered. \[currency_id\]
    Registered(CurrencyId),
    /// Asset was updated. \[currency_id, is_enabled\]
    StatusChanged(CurrencyId, bool),
  }

  // Errors inform users that something went wrong.
  #[pallet::error]
  pub enum Error<T> {
    /// The access to the Asset registry pallet is not allowed for this account ID.
    AccessDenied,
    /// Asset ID is not registered in the asset-registry.
    AssetNotRegistered,
    /// Asset ID status is already the same as requested.
    NoStatusChangeRequested,
    /// Asset is already registered.
    AssetAlreadyRegistered,
    /// Invalid Currency Id
    CurrencyIdNotValid,
  }

  #[pallet::call]
  impl<T: Config> Pallet<T> {
    /// Register new asset on chain.
    ///
    /// - `currency_id`: The currency ID to register
    /// - `name`: Currency name. Ex: `Bitcoin`
    /// - `symbol`: Currency symbol. Ex: `BTC`
    /// - `decimals`: Number of decimals for the asset. Ex: `8`
    /// - `existential_deposit`: Number of token required to keep the balance alive. Ex: `1`
    ///
    /// Emits `Registered` event when successful.
    ///
    /// Weight: `O(1)`
    #[pallet::weight(<T as Config>::WeightInfo::set_status())]
    pub fn register(
      origin: OriginFor<T>,
      currency_id: CurrencyId,
      name: Vec<u8>,
      symbol: Vec<u8>,
      decimals: u8,
      existential_deposit: <T as pallet_assets::Config>::Balance,
    ) -> DispatchResult {
      // 1. Make sure it's signed from the asset-registry owner
      ensure!(
        Some(ensure_signed(origin)?) == Self::account_id(),
        Error::<T>::AccessDenied
      );

      // 2. Make sure the asset isn't already registered
      ensure!(
        !Self::is_currency_exist(currency_id),
        Error::<T>::AssetAlreadyRegistered
      );

      // 3. If it's a wrapped token, let's register it with pallet_assets
      if let CurrencyId::Wrapped(asset_id) = currency_id {
        Self::register_asset(asset_id, name, symbol, decimals, existential_deposit)?;
      }

      // 5. Emit new registered currency
      Self::deposit_event(<Event<T>>::Registered(currency_id));

      Ok(())
    }

    /// Update asset status.
    ///
    /// - `currency_id`: The currency ID to register
    /// - `is_enabled`: Is the currency enabled on chain?
    ///
    /// Emits `StatusChanged` event when successful.
    ///
    /// Weight: `O(1)`
    #[pallet::weight(<T as Config>::WeightInfo::set_status())]
    pub fn set_status(
      origin: OriginFor<T>,
      currency_id: CurrencyId,
      is_enabled: bool,
    ) -> DispatchResult {
      // 1. Make sure it's signed from the asset-registry owner
      ensure!(
        Some(ensure_signed(origin)?) == Self::account_id(),
        Error::<T>::AccessDenied
      );

      // 2. Make sure the currency is already registered
      ensure!(
        Self::is_currency_exist(currency_id),
        Error::<T>::AssetNotRegistered
      );

      // 3. Freeze/unfreeze at the chain level, do nothing if
      // we requested a TDFY freeze
      if let CurrencyId::Wrapped(asset_id) = currency_id {
        match is_enabled {
          true => {
            // unfreeze asset
            pallet_assets::Pallet::<T>::thaw_asset(
              RawOrigin::Signed(T::AssetRegistryPalletId::get().into_account_truncating()).into(),
              asset_id,
            )?;
          }
          false => {
            // freeze asset
            pallet_assets::Pallet::<T>::freeze_asset(
              RawOrigin::Signed(T::AssetRegistryPalletId::get().into_account_truncating()).into(),
              asset_id,
            )?;
          }
        };
      }

      // 4. Emit new registered currency
      Self::deposit_event(<Event<T>>::StatusChanged(currency_id, is_enabled));

      Ok(())
    }
  }

  impl<T: Config> Pallet<T> {
    fn register_asset(
      asset_id: T::AssetId,
      name: Vec<u8>,
      symbol: Vec<u8>,
      decimals: u8,
      existential_deposit: <T as pallet_assets::Config>::Balance,
    ) -> Result<(), DispatchError> {
      // 1. Create asset
      pallet_assets::Pallet::<T>::force_create(
        RawOrigin::Root.into(),
        asset_id,
        // make the pallet account id the owner, so only this pallet can handle the funds.
        T::Lookup::unlookup(T::AssetRegistryPalletId::get().into_account_truncating()),
        true,
        existential_deposit,
      )?;

      // 2. Set metadata
      pallet_assets::Pallet::<T>::force_set_metadata(
        RawOrigin::Signed(T::AssetRegistryPalletId::get().into_account_truncating()).into(),
        asset_id,
        name,
        symbol,
        decimals,
        false,
      )?;

      Ok(())
    }

    pub fn is_currency_exist(currency_id: CurrencyId) -> bool {
      match currency_id {
        // TDFY always exist
        CurrencyId::Tdfy => true,
        CurrencyId::Wrapped(asset_id) => {
          pallet_assets::Pallet::<T>::asset_details(asset_id).is_some()
        }
      }
    }

    pub fn get_account_balance(
      account_id: &T::AccountId,
      asset_id: CurrencyId,
    ) -> Result<CurrencyBalance<BalanceInfo>, DispatchError> {
      // we use `reducible_balance` to return the real available value for the account
      // we also force the `keep_alive`

      // FIXME: Review the `keep_alive` system for TDFY & assets, we should probably have
      // a user settings, where they can enable or force opting-out to keep-alive.
      // that mean if they use all of their funds, the account is deleted from the chain
      // and and will be re-created on next deposit. this could drain all persistent settings
      // of the user as well
      let balance = T::CurrencyTidefi::reducible_balance(asset_id, account_id, true);
      let reserved = T::CurrencyTidefi::balance_on_hold(asset_id, account_id);
      Ok(CurrencyBalance::<BalanceInfo> {
        available: BalanceInfo { amount: balance },
        reserved: BalanceInfo { amount: reserved },
      })
    }

    pub fn get_assets() -> Result<Vec<(CurrencyId, CurrencyMetadata<Vec<u8>>)>, DispatchError> {
      let mut final_assets = vec![(
        CurrencyId::Tdfy,
        CurrencyMetadata {
          name: "Tidefi Token".into(),
          symbol: "TDFY".into(),
          decimals: 12,
          is_frozen: false,
        },
      )];

      let mut asset_metadatas = pallet_assets::Metadata::<T>::iter()
        .map(|(asset_id, asset_metadata)| {
          (
            CurrencyId::Wrapped(asset_id),
            CurrencyMetadata {
              name: asset_metadata.name.into(),
              symbol: asset_metadata.symbol.into(),
              decimals: asset_metadata.decimals,
              is_frozen: asset_metadata.is_frozen,
            },
          )
        })
        .collect();

      final_assets.append(&mut asset_metadatas);

      Ok(final_assets)
    }

    pub fn get_account_balances(
      account_id: &T::AccountId,
    ) -> Result<Vec<(CurrencyId, CurrencyBalance<BalanceInfo>)>, DispatchError> {
      let mut final_balances = vec![(
        CurrencyId::Tdfy,
        Self::get_account_balance(account_id, CurrencyId::Tdfy)?,
      )];
      let mut asset_balances = pallet_assets::Account::<T>::iter_prefix(account_id)
        .map(|(asset_id, balance)| {
          (
            CurrencyId::Wrapped(asset_id),
            CurrencyBalance::<BalanceInfo> {
              available: BalanceInfo {
                amount: balance.balance,
              },
              reserved: BalanceInfo {
                amount: balance.reserved,
              },
            },
          )
        })
        .collect();
      final_balances.append(&mut asset_balances);
      Ok(final_balances)
    }
  }

  impl<T: Config> AssetRegistryExt for Pallet<T> {
    fn is_currency_enabled(currency_id: CurrencyId) -> bool {
      match currency_id {
        // we can't disable TDFY
        CurrencyId::Tdfy => true,
        CurrencyId::Wrapped(asset_id) => pallet_assets::Pallet::<T>::asset_details(asset_id)
          .map(|detail| !detail.is_frozen)
          .unwrap_or(false),
      }
    }
  }
}
